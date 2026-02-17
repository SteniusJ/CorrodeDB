use regex::Regex;
use std::io::{Result, Error, ErrorKind};
use std::option::Option;
use std::fmt;
use crate::{util, meta};
use std::ops::RangeInclusive;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum IndexType {
    Index(u64),
    Wildcard,
}

impl PartialOrd for IndexType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self {
            IndexType::Index(self_v) => {
                if let IndexType::Index(other_v) = other {
                    Some(self_v.cmp(other_v))
                } else {
                    None
                }
            },
            IndexType::Wildcard => {
                None
            },
        }
    }
}

impl IndexType {
    fn from_str(from: &str, table_settings: &meta::TableSettings) -> Option<IndexType> {
        if from == "*" {
            return Some(IndexType::Wildcard);
        }
        if let Ok(v) = from.parse::<u64>() {
            return Some(IndexType::Index(v));
        }
        let subtract_split: Vec<&str> = from.split("-").collect();
        if subtract_split.len() == 2 {
            if let (is_wildcard, Ok(val_2)) = (subtract_split[0] == "*", subtract_split[1].parse::<u64>()) {
                if is_wildcard {
                    let val_1 = table_settings.biggest_id;
                    if val_1 > val_2 {
                        return Some(IndexType::Index(val_1 - val_2));
                    }
                }
            }
            if let (Ok(val_1), Ok(val_2)) = (subtract_split[0].parse::<u64>(), subtract_split[1].parse::<u64>()) {
                if val_1 > val_2 {
                    return Some(IndexType::Index(val_1 - val_2));
                }
            }
        } 
        None
    }
    fn into_range_inclusive(self, to: IndexType) -> Result<RangeInclusive<u64>> {
        if self == IndexType::Wildcard || to == IndexType::Wildcard {
            return Err(Error::new(ErrorKind::InvalidInput, "range does not work on Wildcard"));
        }

        if let (IndexType::Index(from_v), IndexType::Index(to_v)) = (self, to) {
            return Ok(from_v..=to_v);
        }
        Err(Error::new(ErrorKind::Other, "range couldn't be made"))
    }
}

#[derive(Debug, PartialEq)]
pub struct QueryResult {
    pub table_name: String,
    pub indexes: Vec<IndexType>,
    pub fn_name: String,
    pub fn_params: Vec<String>,
    pub sub_fn_names: Vec<String>,
    pub sub_fn_params: Vec<Vec<String>>,
}

impl fmt::Display for QueryResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.sub_fn_names.is_empty() {
            write!(f, "Query: {}{:?} {} {:?}", self.table_name, self.indexes, self.fn_name, self.fn_params)
        } else {
            write!(f, "Query: {}{:?} {} {:?} | {:?} {:?}", self.table_name, self.indexes, self.fn_name, self.fn_params, self.sub_fn_names, self.sub_fn_params)
        }
    }
}

/// Parses database query
pub fn parse_query(query_str: &str, db_settings: &meta::DBSettings) -> Result<QueryResult> {
    let re = Regex::new(r"(?<table_name>[[:alnum:]]*)\[(?<index>[[:digit:],.*-]*)\] ?(?<function_name>[[:alnum:]]*) ?(?<function_params>[[:ascii:]]*)\)?").unwrap();

    let Some(captures) = re.captures(query_str) else {
        return Err(Error::new(ErrorKind::NotFound, "No captures found"));
    };

    let table_name = captures["table_name"].to_string();

    if !db_settings.table_exists(&table_name) {
        return Err(Error::new(ErrorKind::InvalidInput, format!("Table {} doesn't exist", &captures["table_name"])))
    }
    let table_settings = db_settings.tables.get(&table_name).unwrap();

    let mut indexes: Vec<IndexType> = Vec::new();

    for (i, index_str) in captures["index"].split(",").enumerate() {
        if i == 0 {
            let range_re = Regex::new(r"(?<start_index>[[:digit:]*-]*)\.\.(?<end_index>[[:digit:]*-]*)").unwrap();
            match range_re.captures(index_str) {
                Some(captures) => {
                    let Some(mut start_index) = IndexType::from_str(&captures["start_index"], table_settings) else {
                        return Err(Error::new(ErrorKind::Other, "Index couldn't be parsed"));
                    };
                    let Some(mut end_index) = IndexType::from_str(&captures["end_index"], table_settings) else {
                        return Err(Error::new(ErrorKind::Other, "Index couldn't be parsed"));
                    };

                    if start_index == IndexType::Wildcard {
                        start_index = IndexType::Index(0);
                    }
                    if end_index == IndexType::Wildcard {
                        end_index = IndexType::Index(db_settings.tables.get(&table_name).unwrap().biggest_id);
                    }

                    if start_index >= end_index {
                        return Err(Error::new(ErrorKind::Other, "Range not possible"));
                    }

                    let Ok(range) = start_index.into_range_inclusive(end_index) else {
                        return Err(Error::new(ErrorKind::Other, "Range creation failed"));
                    };
                    for index in range {
                        indexes.push(IndexType::Index(index));
                    }
                    break;
                },
                None => (),
            }
        }

        let Some(index) = IndexType::from_str(index_str, table_settings) else {
            return Err(Error::new(ErrorKind::Other, "index couldn't be parsed"));
        };
        if i != 0 && index == IndexType::Wildcard {
            return Err(Error::new(ErrorKind::InvalidInput, "Wildcard can only be used as the first index"));
        }
        if index == IndexType::Wildcard {
            indexes.push(index);
            break;
        }

        indexes.push(index)
    }

    // Possibly temporary solution until I come up with a better regex
    let param_split = util::escape_split(&captures["function_params"], '|');

    let mut fn_params: Vec<String> = Vec::new();
    let mut sub_fn_names: Vec<String> = Vec::new();
    let mut sub_fn_params: Vec<Vec<String>> = Vec::new();

    for (index, fn_str) in param_split.iter().enumerate() {
        let fn_str = fn_str.trim();

        if index == 0 { // we are on the first index which is main function params
            fn_params = util::escape_split(fn_str, ',').iter().map(|v| String::from(*v)).collect();
            continue;
        }

        let sub_fn_split: Vec<&str> = fn_str.splitn(2, ' ').collect();

        sub_fn_names.push(sub_fn_split[0].to_string());

        if sub_fn_split.len() != 2 {
            return Err(Error::new(ErrorKind::InvalidInput, "sub functions require parameters"));
        }
        sub_fn_params.push(util::escape_split(sub_fn_split[1].trim(), ',').iter().map(|v| String::from(*v)).collect());
    }

    Ok(QueryResult {
        table_name: table_name,
        indexes: indexes,
        fn_name: captures["function_name"].to_string(),
        fn_params: fn_params,
        sub_fn_names: sub_fn_names,
        sub_fn_params: sub_fn_params,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_parse() {
        let mut table_map = HashMap::new();
        table_map.insert(String::from("test"), meta::TableSettings{
            columns: Vec::new(),
            biggest_id: 6,
        });
        let db_settings = meta::DBSettings {
            tables: table_map,
            compartment_rows: 10,
        };

        assert_eq!(parse_query(r"test[1,2,3,4] write 3n1298ud8h9apb,sksdo\,kdskd | test dsadija,daisdoi", &db_settings).unwrap(), QueryResult {
            table_name: String::from("test"),
            indexes: vec![IndexType::Index(1),IndexType::Index(2),IndexType::Index(3),IndexType::Index(4)],
            fn_name: String::from("write"),
            fn_params: vec![String::from("3n1298ud8h9apb"),String::from(r"sksdo\,kdskd")],
            sub_fn_names: vec![String::from("test")],
            sub_fn_params: vec![vec![String::from("dsadija"),String::from("daisdoi")]],
        });
        assert_eq!(parse_query("test[*] | test thingy | othertest thingy,1", &db_settings).unwrap(), QueryResult {
            table_name: String::from("test"),
            indexes: vec![IndexType::Wildcard],
            fn_name: String::new(),
            fn_params: vec![String::new()],
            sub_fn_names: vec![String::from("test"), String::from("othertest")],
            sub_fn_params: vec![vec![String::from("thingy")], vec![String::from("thingy"), String::from("1")]],
        });
        assert_eq!(parse_query("test[1..5]", &db_settings).unwrap(), QueryResult {
            table_name: String::from("test"),
            indexes: vec![IndexType::Index(1),IndexType::Index(2),IndexType::Index(3),IndexType::Index(4),IndexType::Index(5)],
            fn_name: String::new(),
            fn_params: vec![String::new()],
            sub_fn_names: Vec::new(),
            sub_fn_params: Vec::new(),
        });
        assert_eq!(parse_query("test[1..*]", &db_settings).unwrap(), QueryResult {
            table_name: String::from("test"),
            indexes: vec![IndexType::Index(1),IndexType::Index(2),IndexType::Index(3),IndexType::Index(4),IndexType::Index(5),IndexType::Index(6)],
            fn_name: String::new(),
            fn_params: vec![String::new()],
            sub_fn_names: Vec::new(),
            sub_fn_params: Vec::new(),
        });
        assert_eq!(parse_query("test[1..*-1]", &db_settings).unwrap(), QueryResult {
            table_name: String::from("test"),
            indexes: vec![IndexType::Index(1),IndexType::Index(2),IndexType::Index(3),IndexType::Index(4),IndexType::Index(5)],
            fn_name: String::new(),
            fn_params: vec![String::new()],
            sub_fn_names: Vec::new(),
            sub_fn_params: Vec::new(),
        });
        parse_query("test 1 dsadsa i", &db_settings).expect_err("Succeeded in parsing incorrect query");
        parse_query("test[1.2]", &db_settings).expect_err("Succeeded in parsing incorrect query");
        parse_query("test[1,m,5,ia,4]", &db_settings).expect_err("Succeeded in parsing incorrect query");
        parse_query("test[0..*-10]", &db_settings).expect_err("Succeeded in parsing incorrect query");
        parse_query("test[0-10]", &db_settings).expect_err("Succeeded in parsing incorrect query");
    }
}

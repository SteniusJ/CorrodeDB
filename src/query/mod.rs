use regex::Regex;
use std::io::{Result, Error, ErrorKind};
use std::fmt;
use crate::util;

#[derive(Debug, PartialEq)]
pub enum IndexType {
    Index(u64),
    Wildcard,
}

#[derive(Debug, PartialEq)]
pub struct QueryResult {
    pub table_name: String,
    pub indexes: Vec<IndexType>,
    pub fn_name: String,
    pub fn_params: Vec<String>,
    pub sub_fn_name: String,
    pub sub_fn_params: Vec<String>,
}

impl fmt::Display for QueryResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.sub_fn_name.is_empty() {
            write!(f, "Query: {}{:?} {} {:?}", self.table_name, self.indexes, self.fn_name, self.fn_params)
        } else {
            write!(f, "Query: {}{:?} {} {:?} | {} {:?}", self.table_name, self.indexes, self.fn_name, self.fn_params, self.sub_fn_name, self.sub_fn_params)
        }
    }
}

/// Parses database query
pub fn parse_query(query_str: &str) -> Result<QueryResult> {
    let re = Regex::new(r"(?<table_name>[[:alnum:]]*)\[(?<index>[[:digit:],.*]*)\] ?(?<function_name>[[:alnum:]]*) ?(?<function_params>[[:ascii:]]*)\)?").unwrap();

    let Some(captures) = re.captures(query_str) else {
        return Err(Error::new(ErrorKind::NotFound, "No captures found"));
    };

    let mut indexes: Vec<IndexType> = Vec::new();

    for (i, index_str) in captures["index"].split(",").enumerate() {
        if i == 0 && index_str == "*" {
            indexes.push(IndexType::Wildcard);
            break;
        }

        if i == 0 {
            let range_re = Regex::new(r"(?<start_index>[[:digit:]]*)\.\.(?<end_index>[[:digit:]]*)").unwrap();
            match range_re.captures(index_str) {
                Some(captures) => {
                    let start_index = captures["start_index"].parse::<u64>().unwrap(); // safe to assume capture is digit as regex statment makes sure of that
                    let end_index = captures["end_index"].parse::<u64>().unwrap();

                    if start_index >= end_index {
                        return Err(Error::new(ErrorKind::Other, "Range not possible"));
                    }

                    for index in start_index..=end_index {
                        indexes.push(IndexType::Index(index));
                    }
                    break;
                },
                None => {
                    if index_str.parse::<u64>().is_err() {
                        return Err(Error::new(ErrorKind::Other, "Range syntax incorrect"));
                    }
                },
            }
        }

        indexes.push(IndexType::Index(index_str.parse().unwrap())) // safe to assume capture is digit as regex statment makes sure of that
    }

    // Possibly temporary solution until I come up with a better regex
    let param_split = util::escape_split(&captures["function_params"], '|');
    
    let fn_params: Vec<String> = util::escape_split(param_split[0].trim(), ',').iter().map(|v| String::from(*v)).collect();

    if param_split.len() > 1 {
        let sub_split: Vec<&str> = param_split[1].trim().splitn(2, ' ').collect();
        
        if sub_split.len() != 2 {
            return Err(Error::new(ErrorKind::InvalidInput, "sub functions require parameters"));
        }

        let sub_fn_params: Vec<String> = util::escape_split(sub_split[1].trim(), ',').iter().map(|v| String::from(*v)).collect();

        return Ok(QueryResult {
            table_name: captures["table_name"].to_string(),
            indexes: indexes,
            fn_name: captures["function_name"].to_string(),
            fn_params: fn_params,
            sub_fn_name: sub_split[0].to_string(),
            sub_fn_params: sub_fn_params,
        });
    }

    Ok(QueryResult {
        table_name: captures["table_name"].to_string(),
        indexes: indexes,
        fn_name: captures["function_name"].to_string(),
        fn_params: fn_params,
        sub_fn_name: String::new(),
        sub_fn_params: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_parse() {
        assert_eq!(parse_query("test[1,2,3,4] write 3n1298ud8h9apb").unwrap(), QueryResult {
            table_name: String::from("test"),
            indexes: vec![IndexType::Index(1),IndexType::Index(2),IndexType::Index(3),IndexType::Index(4)],
            fn_name: String::from("write"),
            fn_param: String::from("3n1298ud8h9apb"),
        });
        assert_eq!(parse_query("test[*]").unwrap(), QueryResult {
            table_name: String::from("test"),
            indexes: vec![IndexType::Wildcard],
            fn_name: String::new(),
            fn_param: String::new(),
        });
        assert_eq!(parse_query("test[1..5]").unwrap(), QueryResult {
            table_name: String::from("test"),
            indexes: vec![IndexType::Index(1),IndexType::Index(2),IndexType::Index(3),IndexType::Index(4),IndexType::Index(5)],
            fn_name: String::new(),
            fn_param: String::new(),
        });
        parse_query("test 1 dsadsa i").expect_err("Succeeded in parsing incorrect query");
        parse_query("test[1.2]").expect_err("Succeeded in parsing incorrect query");
        parse_query("test[1,m,5,ia,4]").expect_err("Succeeded in parsing incorrect query");
    }
}

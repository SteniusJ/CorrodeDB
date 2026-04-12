use std::io::{Result, Error, ErrorKind};
use std::fmt;
use crate::meta;

pub mod tokenizer;

#[derive(Debug, PartialEq)]
pub struct QueryResult {
    pub table_name: String,
    pub indexes: Vec<tokenizer::IndexType>,
    pub fn_name: String,
    pub fn_params: Vec<tokenizer::Token>,
    pub sub_fn_names: Vec<String>,
    pub sub_fn_params: Vec<Vec<tokenizer::Token>>,
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

pub fn parse_query(query_str: &str, db_settings: &meta::DBSettings) -> Result<QueryResult> {
    let mut tokens = tokenizer::tokenize(query_str).into_iter().peekable();

    let Some(tokenizer::Token::String(table_name)) = tokens.next() else {
        return Err(Error::new(ErrorKind::InvalidInput, "Expected string for table name"));
    };
    if !db_settings.table_exists(&table_name) {
        return Err(Error::new(ErrorKind::InvalidInput, format!("Table {table_name} doesn't exist")))
    }
    let table_settings = db_settings.tables.get(&table_name).unwrap();

    let mut indexes: Vec<tokenizer::IndexType> = Vec::new();
    while tokens.peek().is_some() && tokens.peek().unwrap().is_valid_index() {
        match tokens.next() {
            Some(tokenizer::Token::Integer(index)) => indexes.push(tokenizer::IndexType::Index(index as u64)),
            Some(tokenizer::Token::Wildcard(modifier)) => {
                if modifier == 0 {
                    indexes.push(tokenizer::IndexType::Wildcard(0));
                    continue;
                }
                if table_settings.biggest_id > modifier {
                    let index = table_settings.biggest_id - modifier;
                    indexes.push(tokenizer::IndexType::Index(index));
                    continue;
                }
                return Err(Error::new(ErrorKind::InvalidInput, "Invalid wildcard modifier"));
            },
            Some(tokenizer::Token::Range((start, end))) => {
                let start = start.to_int(table_settings);
                let end = end.to_int(table_settings);
                for index in start..=end {
                    indexes.push(tokenizer::IndexType::Index(index));
                }
            },
            Some(tokenizer::Token::Page(page)) => {
                match page {
                    tokenizer::IndexType::Index(page) => {
                        let page = page + 1;
                        let start = db_settings.compartment_rows as u64 * (page - 1);
                        let end = db_settings.compartment_rows as u64 * page - 1;

                        for index in start..=end {
                            if index > table_settings.biggest_id {
                                continue;
                            }
                            indexes.push(tokenizer::IndexType::Index(index));
                        }
                    },
                    tokenizer::IndexType::Wildcard(modifier) => {
                        /* The reason this syntax uses start = comp_rows * page & end = comp_rows * (page + 1)
                         * is because the page indexing uses 0 index but the math doesn't work with
                         * 0 obviously. So in the wildcard syntax we don't modify the page value in
                         * start and end.
                         */
                        let page = (table_settings.biggest_id / db_settings.compartment_rows as u64) - modifier;
                        let start = db_settings.compartment_rows as u64 * page;
                        let end = db_settings.compartment_rows as u64 * (page + 1);

                        println!("start: {start} end: {end}");

                        for index in start..=end {
                            if index > table_settings.biggest_id {
                                continue;
                            }
                            indexes.push(tokenizer::IndexType::Index(index));
                        }
                    }
                }
            },
            _ => (),
        }
    }

    let mut fn_name = String::new();
    let mut fn_params = Vec::new();
    let mut sub_fn_names = Vec::new();
    let mut sub_fn_params = Vec::new();

    let token = tokens.next();
    if let Some(tokenizer::Token::String(name)) = token {
        fn_name = name;
    } else if token.is_some() && token != Some(tokenizer::Token::Pipe) {
        return Err(Error::new(ErrorKind::InvalidInput, "Expected String for function name"));
    }

    if !fn_name.is_empty() {
        while tokens.peek().is_some() {
            let token = tokens.next().unwrap();
            if token == tokenizer::Token::Pipe {
                break;
            }
            fn_params.push(token);
        }
    }

    while tokens.peek().is_some() {
        let token = tokens.next().unwrap();
        if let tokenizer::Token::String(name) = token {
            sub_fn_names.push(name);
        } else {
            return Err(Error::new(ErrorKind::InvalidInput, "Expected String for sub function name"));
        }

        let mut params = Vec::new();
        while tokens.peek().is_some() {
            let token = tokens.next().unwrap();
            if token == tokenizer::Token::Pipe {
                break;
            }
            params.push(token);
        }
        sub_fn_params.push(params);
    }

    Ok(QueryResult {
        table_name: table_name,
        indexes: indexes,
        fn_name: fn_name,
        fn_params: fn_params,
        sub_fn_names: sub_fn_names,
        sub_fn_params: sub_fn_params,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

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
            fn_params: vec![tokenizer::Token::String(String::from("3n1298ud8h9apb")), tokenizer::Token::String(String::from(r"sksdo\,kdskd"))],
            sub_fn_names: vec![String::from("test")],
            sub_fn_params: vec![vec![tokenizer::Token::String(String::from("dsadija")), tokenizer::Token::String(String::from("daisdoi"))]],
        });
        assert_eq!(parse_query("test[*] | test thingy | othertest thingy,1", &db_settings).unwrap(), QueryResult {
            table_name: String::from("test"),
            indexes: vec![IndexType::Wildcard(0)],
            fn_name: String::new(),
            fn_params: vec![],
            sub_fn_names: vec![String::from("test"), String::from("othertest")],
            sub_fn_params: vec![vec![tokenizer::Token::String(String::from("thingy"))], vec![tokenizer::Token::String(String::from("thingy")), tokenizer::Token::Integer(1)]],
        });
        assert_eq!(parse_query("test[1..5]", &db_settings).unwrap(), QueryResult {
            table_name: String::from("test"),
            indexes: vec![IndexType::Index(1),IndexType::Index(2),IndexType::Index(3),IndexType::Index(4),IndexType::Index(5)],
            fn_name: String::new(),
            fn_params: vec![],
            sub_fn_names: Vec::new(),
            sub_fn_params: Vec::new(),
        });
        assert_eq!(parse_query("test[1..*]", &db_settings).unwrap(), QueryResult {
            table_name: String::from("test"),
            indexes: vec![IndexType::Index(1),IndexType::Index(2),IndexType::Index(3),IndexType::Index(4),IndexType::Index(5),IndexType::Index(6)],
            fn_name: String::new(),
            fn_params: vec![],
            sub_fn_names: Vec::new(),
            sub_fn_params: Vec::new(),
        });
        assert_eq!(parse_query("test[1..*-1]", &db_settings).unwrap(), QueryResult {
            table_name: String::from("test"),
            indexes: vec![IndexType::Index(1),IndexType::Index(2),IndexType::Index(3),IndexType::Index(4),IndexType::Index(5)],
            fn_name: String::new(),
            fn_params: vec![],
            sub_fn_names: Vec::new(),
            sub_fn_params: Vec::new(),
        });
        parse_query("test[1.2]", &db_settings).expect_err("Succeeded in parsing incorrect query");
    }
}

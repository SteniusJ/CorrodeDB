use regex::Regex;
use std::io::Result;

#[derive(Debug)]
pub enum IndexType {
    Index(u64),
    Wildcard,
}

#[derive(Debug)]
pub struct QueryResult {
    pub table_name: String,
    pub indexes: Vec<IndexType>,
    pub fn_name: String,
    pub fn_param: String,
}

pub fn parse_query(query_str: &str) -> Result<QueryResult> {
    let re = Regex::new(r"(?<table_name>[[:alnum:]]*)\[(?<index>[[:digit:],.*]*)\] ?(?<function_name>[[:alnum:]]*) ?(?<function_params>[[:ascii:]]*)\)?").unwrap(); // <- proper error handling needed

    let Some(captures) = re.captures(query_str) else {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No captures found"));
    };

    let mut indexes: Vec<IndexType> = Vec::new();

    for (i, index_str) in captures["index"].split(",").enumerate() {
        if i == 0 && index_str == "*" {
            indexes.push(IndexType::Wildcard);
            break;
        }

        if i == 0 {
            let range_re = Regex::new(r"(?<start_index>[[:digit:]]*)..(?<end_index>[[:digit:]]*)").unwrap();
            match range_re.captures(index_str) {
                Some(captures) => {
                    let start_index = captures["start_index"].parse::<u64>().unwrap();
                    let end_index = captures["end_index"].parse::<u64>().unwrap();
                    for index in start_index..=end_index {
                        indexes.push(IndexType::Index(index));
                    }
                    break;
                },
                None => (),
            }
        }

        indexes.push(IndexType::Index(index_str.parse().unwrap())) // error handling
    }

    Ok(QueryResult {
        table_name: captures["table_name"].to_string(),
        indexes: indexes,
        fn_name: captures["function_name"].to_string(),
        fn_param: captures["function_params"].to_string()
    })
}

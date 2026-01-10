use regex::Regex;
use std::io::Result;

#[derive(Debug)]
pub struct QueryResult {
    pub table_name: String,
    pub indexes: Vec<u64>,
    pub fn_name: String,
    pub fn_param: String,
}

pub fn parse_query(query_str: &str) -> Result<QueryResult> {
    let re = Regex::new(r"(?<table_name>[[:alnum:]]*)\[(?<index>[[:digit:],*]*)\].?(?<function_name>[[:alnum:]]*)\(?(?<function_params>[[:alnum:] ,'`]*)\)?").unwrap(); // <- proper error handling needed

    let Some(captures) = re.captures(query_str) else {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No captures found"));
    };

    Ok(QueryResult {
        table_name: captures["table_name"].to_string(),
        indexes: captures["index"].split(",").map(|i| i.parse().unwrap()).collect(), //error handling todo
        fn_name: captures["function_name"].to_string(),
        fn_param: captures["function_params"].to_string()
    })
}

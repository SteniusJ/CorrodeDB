use regex::Regex;
use std::io::Result;

/* Old struct for old parsing, may have use in the future
#[derive(Debug)]
pub struct Result {
    f_name: String,
    params: Vec<String>,
}
*/

pub fn parse_query(query_str: &str) -> Result<(String, Vec<u64>, String, String)> {
    let re = Regex::new(r"(?<table_name>[[:alnum:]]*)\[(?<index>[[:digit:],]*)\].?(?<function_name>[[:alnum:]]*)\(?(?<function_params>[[:alnum:] ,]*)\)?").unwrap(); // <- proper error handling needed

    let Some(captures) = re.captures(query_str) else {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No captures found"));
    };
    
    let table_name = captures["table_name"].to_string();
    let indexes: Vec<u64> = captures["index"].split(",").map(|i| i.parse().unwrap()).collect(); // error handling todo
    let fn_name: String = captures["function_name"].to_string();
    let fn_param: String = captures["function_params"].to_string();

    Ok((table_name, indexes, fn_name, fn_param))

/* Old parsing code for old regex
    for (_, [name, params]) in re.captures_iter(query_str).map(|c| c.extract()) {
        let res = Result{
            f_name: name.to_string(),
            params: params.split(",").map(|p| p.to_string()).collect(),
        };
        results.push(res);
    }
    results
*/
}

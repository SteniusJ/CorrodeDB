use regex::Regex;

#[derive(Debug)]
pub struct Result {
    f_name: String,
    params: Vec<String>,
}

pub fn parse_query(query_str: &str) -> Vec<Result> {
    let re = Regex::new(r"([^\W]*)\(([[:alnum:]\\',. ]*)\)").unwrap();

    let mut results: Vec<Result> = Vec::new();

    for (_, [name, params]) in re.captures_iter(query_str).map(|c| c.extract()) {
        let res = Result{
            f_name: name.to_string(),
            params: params.split(",").map(|p| p.to_string()).collect(),
        };
        results.push(res);
    }
    results
}

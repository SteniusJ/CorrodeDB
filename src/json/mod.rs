pub enum JSONValue {
    String(String),
    NumI(i64),
    NumDec(f64),
}

// Simple JSON encoder that fits this projects needs
pub fn encode(values: Vec<(&str, JSONValue)>) -> String {
    let mut json_string = String::from("{");

    for val in values {
        match val.1 {
            JSONValue::String(v) => {
                json_string.push_str(format!("\"{}\":\"{}\",", val.0, v).as_str());
            },
            JSONValue::NumI(v) => {
                json_string.push_str(format!("\"{}\":{},", val.0, v).as_str());
            },
            JSONValue::NumDec(v) => {
                json_string.push_str(format!("\"{}\":{},", val.0, v).as_str());
            },
        }
    }

    json_string.pop(); // removes last comma from json string
    json_string.push('}');
    json_string
}

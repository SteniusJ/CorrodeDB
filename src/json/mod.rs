use std::fmt;

pub enum JSONValue {
    String(String),
    NumI(i64),
    NumF(f64),
    Array(Vec<JSONValue>),
    Object(Vec<(String, JSONValue)>),
}

impl fmt::Display for JSONValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JSONValue::String(v) => {
                write!(f, "\"{}\"", v)
            },
            JSONValue::NumI(v) => {
                write!(f, "{}", v)
            },
            JSONValue::NumF(v) => {
                write!(f, "{}", v)
            },
            JSONValue::Array(v) => {
                let mut ret = String::from("[");
                for val in v {
                    ret.push_str(format!("{},", val).as_str());
                }
                if ret.len() > 1 {
                    ret.pop();
                }
                ret.push(']');
                write!(f, "{}", ret)
            },
            JSONValue::Object(v) => {
                let mut ret = String::from("{");
                for val in v {
                    ret.push_str(format!("\"{}\":{},", val.0, val.1).as_str());
                }
                if ret.len() > 1 {
                    ret.pop();
                }
                ret.push('}');
                write!(f, "{}", ret)
            }
        }
    }
}

// Simple JSON encoder that fits this projects needs
pub fn encode(values: Vec<(&str, JSONValue)>) -> String {
    let mut json_string = String::from("{");

    for val in values {
        json_string.push_str(format!("\"{}\":{},", val.0, val.1).as_str());
    }

    json_string.pop(); // removes last comma from json string
    json_string.push('}');
    json_string
}

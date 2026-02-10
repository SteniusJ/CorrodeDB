use regex::Regex;
use std::io::{Result, Error, ErrorKind};
use crate::{file, query, meta, db_engine};
use std::collections::HashMap;

/// Splits str by char ignoring those that have been escaped
pub fn escape_split(input: &str, split_char: char) -> Vec<&str> {
    let mut skip = false;
    let mut last_split_i = 0;
    let mut splits: Vec<&str> = Vec::new();

    for (index, char) in input.chars().enumerate() {
        if skip {
            skip = false;
            continue;
        }
        
        if char == '\\' {
            skip = true;
            continue;
        }

        if char == split_char {
            splits.push(input.get(last_split_i..index).unwrap());
            last_split_i = index + 1;
        }
    }

    splits.push(input.get(last_split_i..input.len()).unwrap());
    splits
}

pub fn sanitize_db_entry(input: String) -> String {
    let mut sanitized_string = String::new();
    for char in input.chars() {
        match char {
            '\n' => sanitized_string.push_str(r"\n"),
            '"' => sanitized_string.push_str("\""),
            char => sanitized_string.push(char),
        }
    }
    sanitized_string
}

pub fn rehydrate_db_entry(input: String) -> String {
    let mut rehydrated_string = String::new();
    for char in input.chars() {
        match char {
            '\\' => (),
            char => rehydrated_string.push(char),
        }
    }
    rehydrated_string
}

/// Parses program arguments
pub fn parse_program_args(args: Vec<String>) -> Vec<(String, String)>{
    let arguments_string = args.join(" ");
    let re = Regex::new(r"(?<flag>-[[:alpha:]]*) (?<param>[[:ascii:]-- -]*)").unwrap();

    let mut parsed_args: Vec<(String, String)> = Vec::new();

    let captures = re.captures_iter(&arguments_string);

    for argument in captures {
        parsed_args.push((argument["flag"].to_string(), argument["param"].to_string()));
    }

    parsed_args
}

/// Reads file from database
pub fn file_read(file_name: &str, file_system: &mut file::FileSystem) -> Result<Vec<String>> {
    match file_system.open(file_name) {
        Ok(_) => (),
        Err(e) if e.kind() == ErrorKind::InvalidInput => (),
        Err(e) => {
            return Err(e);
        }
    }

    let Ok(file_data) = file_system.read_from_cache(file_name) else {
        return Err(Error::new(ErrorKind::NotFound, "File not in cache"));
    };

    Ok(file_data)
}

/// Returns line, function name and database index
pub fn get_line_fname_idx(db_settings: &meta::DBSettings, query: &query::QueryResult, index: u64) -> (u64, String, u64) {
    let container = num::integer::div_floor(index, db_settings.compartment_rows as u64);
    let line = if index < db_settings.compartment_rows as u64 {index} else {index - (db_settings.compartment_rows as u64 * container)};
    let file_name = format!("./tables/{}/{}", query.table_name, container);
    let index = get_index(line, container, db_settings);

    (line, file_name, index)
}

/// Returns database index
pub fn get_index(line: u64, container: u64, db_settings: &meta::DBSettings) -> u64 {
    line + (container * db_settings.compartment_rows as u64)
}

/// Writes to database
pub fn file_write(file_name: &str, file_data: Vec<String>, file_system: &mut file::FileSystem) -> bool {
    match file_system.write_to_cache(file_name, file_data.join("\n")) {
        Ok(_) => (),
        Err(_) => {
            return false;
        }
    }

    match file_system.write_entire_cache_to_disk() {
        Ok(_) => return true,
        Err(_) => return false, 
    }
}

/// Reads line from database
pub fn read_line(file_name: &str, file_system: &mut file::FileSystem, line: u64) -> Result<String> {
    match file_system.open(file_name) {
        Ok(_) => (),
        Err(e) if e.kind() == ErrorKind::InvalidInput => (),
        Err(e) => {
            return Err(e);
        }
    } 

    match file_system.read_line_from_cache(file_name, line as usize) {
        Ok(content) => {
            if !content.is_empty() {
                return Ok(content);
            }

            return Err(Error::new(ErrorKind::Other, "content is empty"));
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub fn parse_db_line(line: String, index: u64, col_settings: &Vec<meta::ColSettings>) -> HashMap<String, db_engine::DBDatatype> {
    let split_line = escape_split(line.as_str(), ',');
    let mut row_content: HashMap<String, db_engine::DBDatatype> = HashMap::new();
    row_content.insert(String::from("index"), db_engine::DBDatatype::NumberI(index as i64));

    for (column_index, column_content) in split_line.iter().enumerate() {
        let column_name = col_settings[column_index].name.clone();

        match col_settings[column_index].value {
            meta::ColValue::NumberI => row_content.insert(column_name, db_engine::DBDatatype::NumberI(column_content.parse().unwrap())),
            meta::ColValue::NumberF => row_content.insert(column_name, db_engine::DBDatatype::NumberF(column_content.parse().unwrap())),
            meta::ColValue::VarChar => row_content.insert(column_name, db_engine::DBDatatype::VarChar(column_content.to_string())),
        };
    }

    row_content
}

pub fn db_result_prettify(result: Vec<HashMap<String, db_engine::DBDatatype>>) -> String {
    let mut pretty_string = String::new();

    for row_data in result {
        pretty_string.push_str("{\n");
        
        for (col_name, col_data) in row_data {
            let col_string = format!("  {col_name}: {col_data:?}\n");
            pretty_string.push_str(&col_string);
        }
        pretty_string.push_str("}\n");
    }

    pretty_string
}

pub fn merge_sort(data: Vec<HashMap<String, db_engine::DBDatatype>>, sort_order: &str, sort_column: &str) -> Result<Vec<HashMap<String, db_engine::DBDatatype>>> {
    if data.len() <= 1 {
        return Ok(data);
    }

    let mut left: Vec<HashMap<String, db_engine::DBDatatype>> = Vec::new();
    let mut right: Vec<HashMap<String, db_engine::DBDatatype>> = Vec::new();
    let data_len = data.len();

    for (index, value) in data.into_iter().enumerate() {
        if index < (data_len / 2) {
            left.push(value);
        } else {
            right.push(value);
        }
    }

    left = match merge_sort(left, sort_order, sort_column) {
        Ok(data) => data,
        Err(e) => return Err(e),
    };
    right = match merge_sort(right, sort_order, sort_column) {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    merge(&mut left, &mut right, sort_order, sort_column)
}

fn merge(left: &mut Vec<HashMap<String, db_engine::DBDatatype>>, right: &mut Vec<HashMap<String, db_engine::DBDatatype>>, sort_order: &str, sort_column: &str) -> Result<Vec<HashMap<String, db_engine::DBDatatype>>> {
    let mut result: Vec<HashMap<String, db_engine::DBDatatype>> = Vec::new();

    while !left.is_empty() && !right.is_empty() {
        match sort_order {
            "asc" => {
                match left.first().unwrap().get(sort_column).unwrap() {
                    db_engine::DBDatatype::NumberI(left_v) => {
                        if let db_engine::DBDatatype::NumberI(right_v) = right.first().unwrap().get(sort_column).unwrap() {
                            if left_v <= right_v {
                                result.push(left.remove(0));
                            } else {
                                result.push(right.remove(0));
                            }
                        }
                    },
                    db_engine::DBDatatype::NumberF(left_v) => {
                        if let db_engine::DBDatatype::NumberF(right_v) = right.first().unwrap().get(sort_column).unwrap() {
                            if left_v <= right_v {
                                result.push(left.remove(0));
                            } else {
                                result.push(right.remove(0));
                            }
                        }
                    },
                    db_engine::DBDatatype::VarChar(left_v) => {
                        if let db_engine::DBDatatype::VarChar(right_v) = right.first().unwrap().get(sort_column).unwrap() {
                            if left_v <= right_v {
                                result.push(left.remove(0));
                            } else {
                                result.push(right.remove(0));
                            }
                        }
                    },
                }
            },
            "dsc" => {
                match left.first().unwrap().get(sort_column).unwrap() {
                    db_engine::DBDatatype::NumberI(left_v) => {
                        if let db_engine::DBDatatype::NumberI(right_v) = right.first().unwrap().get(sort_column).unwrap() {
                            if left_v >= right_v {
                                result.push(left.remove(0));
                            } else {
                                result.push(right.remove(0));
                            }
                        }
                    },
                    db_engine::DBDatatype::NumberF(left_v) => {
                        if let db_engine::DBDatatype::NumberF(right_v) = right.first().unwrap().get(sort_column).unwrap() {
                            if left_v >= right_v {
                                result.push(left.remove(0));
                            } else {
                                result.push(right.remove(0));
                            }
                        }
                    },
                    db_engine::DBDatatype::VarChar(left_v) => {
                        if let db_engine::DBDatatype::VarChar(right_v) = right.first().unwrap().get(sort_column).unwrap() {
                            if left_v >= right_v {
                                result.push(left.remove(0));
                            } else {
                                result.push(right.remove(0));
                            }
                        }
                    },
                }
            },
            ord => return Err(Error::new(ErrorKind::InvalidInput, format!("{ord} is not a valid sort order"))),
        }

    }

    while !left.is_empty() {
        result.push(left.remove(0));
    }
    while !right.is_empty() {
        result.push(right.remove(0));
    }

    Ok(result)
}

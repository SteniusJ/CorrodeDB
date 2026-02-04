mod file;
mod meta;
mod http;
mod query;
mod json;
mod util;

use rand::prelude::*;
use std::io::{Result, Error, ErrorKind};
use std::env;

const DEFAULT_SCHEMA_FILE_PATH: &str = "./schema.yaml";
const DEFAULT_PORT: &str = "4067";

pub struct ProgramArgs {
    pub schema_path: String,
    pub port: String,
}

/// Loads program arguments
pub fn load_program_arguments() -> ProgramArgs {
    let mut program_args = ProgramArgs {
        schema_path: String::from(DEFAULT_SCHEMA_FILE_PATH),
        port: String::from(DEFAULT_PORT),
    };

    // Read program arguments
    let args = util::parse_program_args(env::args().collect());

    for (flag, value) in args {
        match flag.as_str() {
            "-s" => {
                program_args.schema_path = value;
            },
            "-p" => {
                program_args.port = value;
            },
            f=> {
                println!("flag {f} is not a valid flag");
            },
        }
    }

    program_args
}

/// Starts database server
pub fn start_database(schema_path: &str, port: &str) {
    // Order of definition is critical,
    // variables used inside endpoints of the http_server
    // need to be defined before the server itself
    let mut file_system = file::FileSystem::new();
    let mut db_settings = meta::load_meta(schema_path);
    let db_password = db_settings.password.clone();
    let mut http_server = http::HTTPServer::new(format!("127.0.0.1:{port}"));

    http_server.add_middleware(|_body, url_params| {
        if !url_params.contains_key("password") {
            return (false, "Password url parameter required".to_string());
        }
 
        if url_params.get("password").unwrap() == &db_password {
            return (true, String::new());
        }
        (false, "Given password is incorrect".to_string())
    });

    http_server.add_endpoint("/", http::HTTPRequestMethods::POST, |body, _url_params| {
        let mut query = match query::parse_query(body.as_str()) {
            Ok(query) => query,
            Err(e) => {
                println!("Query parse error: {e}");
                return http::create_http_response(400, "application/json",  json::encode(vec![("error", json::JSONValue::String("Query could not be parsed".to_string()))]).as_str());
            }
        };

        println!("{query}");

        if !db_settings.table_exists(&query.table_name) {
            return http::create_http_response(404, "application/json", json::encode(vec![("error", json::JSONValue::String("Given table does not exist".to_string()))]).as_str());
        }

        // Run any function if none read from database
        match query.fn_name.as_str() {
            "write" => write_to_db(&mut db_settings, &mut file_system, &query),
            "random" => random_from_db(&mut db_settings, &mut file_system, &mut query),
            "remove" => remove_from_db(&mut db_settings, &mut file_system, &query),
            "where" => where_from_db(&mut db_settings, &mut file_system, &query),
            "" => read_from_db(&mut db_settings, &mut file_system, &query),
            _ => http::create_http_response(404, "application/json", json::encode(vec![("error", json::JSONValue::String("Given function does not exist".to_string()))]).as_str()),
        }
    });

    http_server.listen();
}

/// Reads file from database
fn file_read(file_name: &str, file_system: &mut file::FileSystem) -> Result<Vec<String>> {
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
fn get_line_fname_idx(db_settings: &meta::DBSettings, query: &query::QueryResult, index: u64) -> (u64, String, u64) {
    let container = num::integer::div_floor(index, db_settings.compartment_rows as u64);
    let line = if index < db_settings.compartment_rows as u64 {index} else {index - db_settings.compartment_rows as u64};
    let file_name = format!("./tables/{}/{}", query.table_name, container);
    let index = get_index(line, container, db_settings);

    (line, file_name, index)
}

/// Returns database index
fn get_index(line: u64, container: u64, db_settings: &meta::DBSettings) -> u64 {
    line + (container * db_settings.compartment_rows as u64)
}

/// Writes to database
fn file_write(file_name: &str, file_data: Vec<String>, file_system: &mut file::FileSystem) -> bool {
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
fn read_line(file_name: &str, file_system: &mut file::FileSystem, line: u64) -> Result<String> {
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

/// Where function logic
///
/// Returns data which matches condition
fn where_from_db(db_settings: &mut meta::DBSettings, file_system: &mut file::FileSystem, query: &query::QueryResult) -> String {
    let mut result: Vec<(u64, String)> = Vec::new();
    let mut arguments = util::escape_split(query.fn_param.as_str(), ',').into_iter();

    // Assign variables necessary for conditon matching
    let column = {
        let Some(column) = arguments.next() else {
            return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("Please give a column name".to_string()))]).as_str());
        };
        if column.is_empty() {
            return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("Please give a column name".to_string()))]).as_str());
        }
        if !db_settings.tables.get(&query.table_name).unwrap().has_column(column.to_string()) {
            return http::create_http_response(404, "application/json", json::encode(vec![("error", json::JSONValue::String(format!("Column '{}' does not exist in table '{}'", column, &query.table_name)))]).as_str());
        }
        column
    };
    let operator = {
        let Some(operator) = arguments.next() else {
            return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("Please give a operator".to_string()))]).as_str());
        };
        if operator.is_empty() {
            return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("Please give a operator".to_string()))]).as_str());
        }
        if !match operator {
            ">" => true,
            "<" => true,
            "=" => true,
            _ => false,
        } {
            return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String(format!("{operator} is not a valid operator")))]).as_str());
        }
        operator
    };
    let Some(value) = arguments.next() else {
        return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("Please give a comparison value".to_string()))]).as_str());
    };

    /// Checks if condition is matching
    /// Returns Ok(true) if condition matches
    /// Returns Ok(false) if condition is not matching
    /// Returns Err(_) on incorrect operator value pair
    fn is_matching(column_value: &meta::ColValue, column_content: &str, match_value: &str, operator: &str) -> Result<bool> {
        match operator {
            ">" => {
                match column_value {
                    meta::ColValue::NumberI => {
                        if let Ok(value) = match_value.parse::<i64>() {
                            if column_content.parse::<i64>().unwrap() > value {
                                return Ok(true);
                            }
                        }
                        return Ok(false);
                    },
                    meta::ColValue::NumberF => {
                        if let Ok(value) = match_value.parse::<f64>() {
                            if column_content.parse::<f64>().unwrap() > value {
                                return Ok(true);
                            }
                        }
                        return Ok(false);
                    },
                    meta::ColValue::VarChar => return Err(Error::new(ErrorKind::Other, "cannot use > operator on VarChar")),
                }
            },
            "<" => {
                match column_value {
                    meta::ColValue::NumberI => {
                        if let Ok(value) = match_value.parse::<i64>() {
                            if column_content.parse::<i64>().unwrap() < value {
                                return Ok(true);
                            }
                        }
                        return Ok(false);
                    },
                    meta::ColValue::NumberF => {
                        if let Ok(value) = match_value.parse::<f64>() {
                            if column_content.parse::<f64>().unwrap() < value {
                                return Ok(true);
                            }
                        }
                        return Ok(false);
                    },
                    meta::ColValue::VarChar => return Err(Error::new(ErrorKind::Other, "cannot use < operator on VarChar")),
                }
            },
            "=" => {
                if column_content == match_value {
                    return Ok(true);
                }
                return Ok(false);
            },
            _ => return Ok(false),
        }
    }

    for index in &query.indexes {
        match index {
            query::IndexType::Index(i) => {
                let (line, file_name, index) = get_line_fname_idx(db_settings, query, *i);
                let (column_index, column) = db_settings.tables.get(&query.table_name).unwrap().get_column(column.to_string()).unwrap();

                match read_line(file_name.as_str(), file_system,  line) {
                    Ok(content) => {
                            if content.is_empty() {
                                continue;
                            }

                            let column_content = util::escape_split(content.as_str(), ',')[column_index];

                            match is_matching(&column.value, column_content, value, operator) {
                                Ok(b) => {
                                    if b {
                                        result.push((index, content));
                                    }
                                },
                                Err(e) => {
                                    return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String(format!("{e}")))]).as_str())
                                },
                            }
                    },
                    Err(e) if e.kind() == ErrorKind::Other => (),
                    Err(_) => {
                        return http::create_http_response(404, "application/json", json::encode(vec![("error", json::JSONValue::String("Index out of table range".to_string()))]).as_str());
                    }
                };
            },
            query::IndexType::Wildcard => {
                let dir_name = format!("./tables/{}", query.table_name);
                let dir = file_system.read_folder(dir_name.as_str());
                let (column_index, column) = db_settings.tables.get(&query.table_name).unwrap().get_column(column.to_string()).unwrap();

                for file in dir {
                    match file {
                        Ok(dir_entry) => {
                            let container = dir_entry.file_name().into_string().unwrap().parse::<u64>().unwrap();
                            let file_name = format!("{}/{}", dir_name, container);
                            match file_system.open(file_name.as_str()) {
                                Ok(_) => (),
                                Err(e) if e.kind() == ErrorKind::InvalidInput => (),
                                Err(e) => {
                                    println!("File open failed: {e}");
                                    continue;
                                },
                            }

                            match file_system.read_from_cache(file_name.as_str()) {
                                Ok(contents) => {
                                    for (index, content) in contents.into_iter().enumerate() {
                                        if content.is_empty() {
                                            continue;
                                        }

                                        let index = get_index(index as u64, container, db_settings);

                                        let column_content = util::escape_split(content.as_str(), ',')[column_index];

                                        match is_matching(&column.value, column_content, value, operator) {
                                            Ok(b) => {
                                                if b {
                                                    result.push((index, content));
                                                }
                                            },
                                            Err(e) => {
                                                return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String(format!("{e}")))]).as_str())
                                            },
                                        }
                                    }
                                },
                                Err(e) => {
                                    println!("Read failed: {e}");
                                    continue;
                                },
                            }
                        },
                        Err(_) => (),
                    }
                }
            },
        }
    }

    file_system.drop_entire_cache();
    http::create_http_response(200, "application/json", encode_db_return(result, &db_settings, &query).as_str())
}

/// Remove function logic
///
/// Removes data from database
/// Overwrites data with empty string effectively deleting it
fn remove_from_db(db_settings: &mut meta::DBSettings, file_system: &mut file::FileSystem, query: &query::QueryResult) -> String {
    if query.indexes.len() > 1 {
        return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("Data can only be removed from one index at a time".to_string()))]).as_str());
    }

    let query::IndexType::Index(i) = query.indexes[0] else {
        return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("Index type is incorrect".to_string()))]).as_str());
    };

    let (line, file_name, index) = get_line_fname_idx(db_settings, query, i);

    let Ok(mut file_data) = file_read(file_name.as_str(), file_system) else {
        return http::create_http_response(500, "application/json", json::encode(vec![("error", json::JSONValue::String("File read error".to_string()))]).as_str());
    };

    file_data.insert(line as usize, String::new()); // Overwrite current value with empty String
    
    if file_write(file_name.as_str(), file_data, file_system) {
        return http::create_http_response(200, "application/json", json::encode(vec![("status", json::JSONValue::String(format!("Item at index {index} has been removed")))]).as_str());
    } else {
        return http::create_http_response(500, "application/json", json::encode(vec![("error", json::JSONValue::String("Removal of Item failed due to a write error".to_string()))]).as_str())
    }
}

/// random function logic
///
/// Returns random values from database
fn random_from_db(db_settings: &mut meta::DBSettings, file_system: &mut file::FileSystem, query: &mut query::QueryResult) -> String {
    let mut rng = rand::rng();

    let Ok(nr_of_random_values) = query.fn_param.parse::<u64>() else {
        return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("Incorrect parameter type for random function".to_string()))]).as_str());
    };

    let biggest_id = db_settings.tables.get(&query.table_name).unwrap().biggest_id;

    match &query.indexes[0] {
        query::IndexType::Index(_) => {
            if query.indexes.len() < nr_of_random_values as usize {
                return http::create_http_response(404, "application/json", json::encode(vec![("error", json::JSONValue::String("Attempting to retrieve more random values than the given query includes".to_string()))]).as_str())
            }

            let mut indexes: Vec<u64> = (0..=query.indexes.len() as u64).collect();
            indexes.shuffle(&mut rng);
            indexes.truncate(nr_of_random_values as usize);

            query.indexes = indexes.iter().map(|i| query::IndexType::Index(*i)).collect();
        },
        query::IndexType::Wildcard => {
            if biggest_id + 1 < nr_of_random_values {
                return http::create_http_response(404, "application/json", json::encode(vec![("error", json::JSONValue::String("Attempting to retrieve more random values than the table includes".to_string()))]).as_str())
            }

            let mut indexes: Vec<u64> = (0..=biggest_id).collect();
            indexes.shuffle(&mut rng);

            let mut result: Vec<(u64, String)> = Vec::new();

            for i in indexes {
                let (line, file_name, index) = get_line_fname_idx(db_settings, query, i);

                match read_line(file_name.as_str(), file_system, line) {
                    Ok(content) => result.push((index, content)),
                    Err(e) if e.kind() == ErrorKind::Other => (),
                    Err(_) => {
                        return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("Index out of table range".to_string()))]).as_str());
                    }
                };

                if result.len() >= nr_of_random_values as usize{
                    file_system.drop_entire_cache();
                    return http::create_http_response(200, "application/json", encode_db_return(result, &db_settings, &query).as_str());
                }
            }
            return http::create_http_response(404, "application/json", json::encode(vec![("error", json::JSONValue::String("Attempting to retrieve more values that the table includes".to_string()))]).as_str());
        }
    }

    file_system.drop_entire_cache();
    read_from_db(db_settings, file_system, query)
}

/// Reads data from database
/// Default functionality
fn read_from_db(db_settings: &meta::DBSettings, file_system: &mut file::FileSystem, query: &query::QueryResult) -> String {
    let mut result: Vec<(u64, String)> = Vec::new();

    for index in &query.indexes {
        match index {
            query::IndexType::Index(i) => {
                let (line, file_name, index) = get_line_fname_idx(db_settings, query, *i);

                match read_line(file_name.as_str(), file_system, line) {
                    Ok(content) => result.push((index, content)),
                    Err(e) if e.kind() == ErrorKind::Other => (),
                    Err(_) => {
                        return http::create_http_response(404, "application/json", json::encode(vec![("error", json::JSONValue::String("Index out of table range".to_string()))]).as_str());
                    }
                };
            },
            query::IndexType::Wildcard => {
                let dir_name = format!("./tables/{}", query.table_name);
                let dir = file_system.read_folder(dir_name.as_str());
                let mut containers: Vec<u64> = dir.map(|res_dir_entry| 
                    res_dir_entry.unwrap()
                        .file_name()
                        .to_str()
                        .unwrap()
                        .parse::<u64>()
                        .unwrap())
                    .collect();
                containers.sort(); // This is necessary to make sure that the indexes in the
                                   // response are in order. Looping trough ReadDir results in a
                                   // iterator which is not always in order.

                for container in containers {
                    let file_name = format!("{}/{}", dir_name, container);
                    match file_system.open(file_name.as_str()) {
                        Ok(_) => (),
                        Err(e) if e.kind() == ErrorKind::InvalidInput => (),
                        Err(e) => {
                            println!("File open failed: {e}");
                            continue;
                        },
                    }

                    match file_system.read_from_cache(file_name.as_str()) {
                        Ok(contents) => {
                            let mut contents_with_index: Vec<(u64, String)> = contents.iter().enumerate().map(|line| (get_index(line.0 as u64, container, db_settings), line.1.clone())).collect();
                            result.append(&mut contents_with_index);
                        },
                        Err(e) => {
                            println!("Read failed: {e}");
                            continue;
                        },
                    }
                }
            },
        }
    }

    file_system.drop_entire_cache();
    http::create_http_response(200, "application/json", encode_db_return(result, &db_settings, &query).as_str())
}

/// write function logic
///
/// Writes data to database
fn write_to_db(db_settings: &mut meta::DBSettings, file_system: &mut file::FileSystem, query: &query::QueryResult) -> String {
    if query.indexes.len() > 1 {
        return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("Data can only be written to one index at a time".to_string()))]).as_str());
    }

    let row_data_split = util::escape_split(query.fn_param.as_str(), ',');
    let columns = &db_settings.tables.get(&query.table_name).unwrap().columns;

    if row_data_split.len() != columns.len() {
        return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("The number of arguments does not match the number of data columns in the database".to_string()))]).as_str());
    }

    // Check if given data matches column data types
    for row_data in row_data_split.iter().enumerate() {
        let col_data = &columns[row_data.0];

        match col_data.value {
            meta::ColValue::NumberI => {
                if row_data.1.parse::<i64>().is_err() {
                    return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String(format!("Data type does not match column data type which is NumberI")))]).as_str());
                }
            },
            meta::ColValue::NumberF => {
                if row_data.1.parse::<f64>().is_err() {
                    return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String(format!("Data type does not match column data type which is NumberF")))]).as_str());
                }
            },
            _ => (),
        }
    }

    // Since data can only be written to one index at a time
    // we only need to look at the first index.
    match &query.indexes[0] {
        query::IndexType::Index(i) => {
            let (line, file_name, index) = get_line_fname_idx(db_settings, query, *i);

            let Ok(mut file_data) = file_read(file_name.as_str(), file_system) else {
                return http::create_http_response(500, "application/json", json::encode(vec![("error", json::JSONValue::String("File read error".to_string()))]).as_str());
            };

            file_data.insert(line as usize, query.fn_param.clone());

            if file_write(file_name.as_str(), file_data, file_system) {
                return http::create_http_response(200, "application/json", json::encode(vec![("status", json::JSONValue::String(format!("Item at index {index} has been removed")))]).as_str());
            } else {
                return http::create_http_response(500, "application/json", json::encode(vec![("error", json::JSONValue::String("Removal of Item failed due to a write error".to_string()))]).as_str())
            }
        },
        query::IndexType::Wildcard => {
            let table_max_index = if db_settings.tables.get(&query.table_name).unwrap().biggest_id > 0 {
                db_settings.tables.get(&query.table_name).unwrap().biggest_id + 1
            } else {
                0
            };
            let (_line, file_name, _index) = get_line_fname_idx(db_settings, query, table_max_index);

            let Ok(mut file_data) = file_read(file_name.as_str(), file_system) else {
                return http::create_http_response(500, "application/json", json::encode(vec![("error", json::JSONValue::String("File read error".to_string()))]).as_str());
            };

            file_data.push(query.fn_param.clone());

            if file_write(file_name.as_str(), file_data, file_system) {
                db_settings.iterate_id(&query.table_name);
                return http::create_http_response(200, "application/json", json::encode(vec![("status", json::JSONValue::String(format!("Write success"))), ("index", json::JSONValue::NumI(table_max_index as i64))]).as_str());
            } else {
                return http::create_http_response(500, "application/json", json::encode(vec![("error", json::JSONValue::String("Write failed".to_string()))]).as_str())
            }
        },
    }
}

/// Encodes returned data from file read into a json string
fn encode_db_return(vec: Vec<(u64, String)>, db_settings: &meta::DBSettings, query: &query::QueryResult) -> String {
    let db_cols = &db_settings.tables.get(query.table_name.as_str()).unwrap().columns;
    let mut json_array: Vec<json::JSONValue> = Vec::new();

    for data_row in vec {
        if data_row.1.is_empty() {
            continue;
        }

        let mut json_object: Vec<(String, json::JSONValue)> = Vec::new();

        json_object.push(("index".to_string(), json::JSONValue::NumI(data_row.0 as i64)));

        for data in util::escape_split(data_row.1.as_str(), ',').iter().enumerate() {
            let col_data = &db_cols[data.0];

            match col_data.value {
                meta::ColValue::NumberI => {
                    json_object.push((col_data.name.clone(), json::JSONValue::NumI(data.1.parse().unwrap())));
                },
                meta::ColValue::NumberF => {
                    json_object.push((col_data.name.clone(), json::JSONValue::NumF(data.1.parse().unwrap())));
                },
                meta::ColValue::VarChar => {
                    json_object.push((col_data.name.clone(), json::JSONValue::String(util::remove_escape_characters(data.1.to_string()))));
                },
            }
        }
        json_array.push(json::JSONValue::Object(json_object));
    }

    json::encode(vec![("data", json::JSONValue::Array(json_array))])
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;
    use query::IndexType;
    
    const TEST_TABLE_1: &str = "testtable1";
    const TEST_TABLE_2: &str = "testtable2";

    fn common(table_name: &str, fn_name: &str, indexes: Vec<IndexType>, fn_param: &str) -> (meta::DBSettings, file::FileSystem, query::QueryResult) {
        (
            meta::load_meta("./test_data/test_schema.yaml"),
            file::FileSystem::new(),
            query::QueryResult {
                table_name: table_name.to_string(),
                fn_name: fn_name.to_string(),
                indexes: indexes,
                fn_param: fn_param.to_string(),
            }
        )
    }

    #[test]
    fn test_return_encode() {
        let (db_settings, _file_system, query) = common(TEST_TABLE_1, "", Vec::new(), "");

        let pre_encode: Vec<(u64, String)> = vec![(1, String::from(r"1,2.123,hello\, world")),(2, String::from(r"1,2.123,hello\, world"))];
        let result = encode_db_return(pre_encode, &db_settings, &query);
        assert_eq!(result, String::from("{\"data\":[{\"index\":1,\"numi\":1,\"numf\":2.123,\"varchar\":\"hello, world\"},{\"index\":2,\"numi\":1,\"numf\":2.123,\"varchar\":\"hello, world\"}]}"));
    }

    #[test]
    fn test_write_to_db() {
        let mut rng = rand::rng();
        let (mut db_settings, mut file_system, query) = common(TEST_TABLE_1, "write", vec![IndexType::Wildcard], r"1,2.22,hello\, world");

        let result = write_to_db(&mut db_settings, &mut file_system, &query);
        assert_eq!("200", result.get(9..12).unwrap()); // checks if return is 200

        let random_uuid = format!("0x{:X}", rng.random::<u128>());
        let query = query::QueryResult { table_name: TEST_TABLE_1.to_string(), indexes: vec![IndexType::Index(1)], fn_name: "write".to_string(), fn_param: format!("1,2.22,{}", random_uuid) };
        let result = write_to_db(&mut db_settings, &mut file_system, &query);
        assert_eq!("200", result.get(9..12).unwrap()); // checks if return is 200
        let query = query::QueryResult { table_name: TEST_TABLE_1.to_string(), indexes: vec![IndexType::Index(1)], fn_name: String::new(), fn_param: String::new() };
        let result = read_from_db(&db_settings, &mut file_system, &query);
        assert_eq!(result.split("\r\n\r\n").last().unwrap().to_string(), String::from("{\"data\":[{\"index\":1,\"numi\":1,\"numf\":2.22,\"varchar\":\"") + random_uuid.as_str() + "\"}]}"); // has to be done like this since format! doesn't work due to use of curly braces in json

        let query = query::QueryResult { table_name: TEST_TABLE_1.to_string(), indexes: vec![IndexType::Index(1)], fn_name: "write".to_string(), fn_param: String::from("this is incorrect data for this table, 22121, ghjello") };
        let result = write_to_db(&mut db_settings, &mut file_system, &query);
        assert_eq!("400", result.get(9..12).unwrap()); // checks if return is 400
    }
}

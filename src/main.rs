mod file;
mod meta;
mod http;
mod query;
mod json;
mod util;

use std::env;
use rand::prelude::*;

const _TEST_FILE_PATH: &str = "./tables/test";
const _DEFAULT_META_FILE_PATH: &str = "./meta.yaml";

fn main() {
    // Read program arguments
    let args: Vec<String> = env::args().collect();

    let meta_file_path: &str = args[1].as_str();

    // Order of definition is critical,
    // variables used inside endpoints of the http_server
    // need to be defined before the server itself
    let mut file_system = file::FileSystem::new();
    let mut db_settings = meta::load_meta(meta_file_path);
    let db_password = db_settings.password.clone();
    let mut http_server = http::HTTPServer::new("127.0.0.1:7878".to_string());

    http_server.add_middleware(|_body, url_params| {
        if !url_params.contains_key("password") {
            return (false, "Password url parameter required".to_string());
        }
 
        if url_params.get("password").unwrap() == &db_password {
            println!("password: {}, matches given_password: {}", url_params.get("password").unwrap(), db_password);
            return (true, String::new());
        }
        (false, "Given password is incorrect".to_string())
    });
    
    http_server.add_endpoint("/", http::HTTPRequestMethods::POST, |body, _url_params| {
        let mut query = match query::parse_query(body.as_str()) {
            Ok(query) => query,
            Err(e) => {
                println!("{e}");
                return http::create_http_response(400, "application/json",  json::encode(vec![("error", json::JSONValue::String("Query could not be parsed".to_string()))]).as_str());
            }
        };

        println!("{query:?}");

        if !db_settings.table_exists(&query.table_name) {
            return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("Given table does not exist".to_string()))]).as_str());
        }

        match query.fn_name.as_str() {
            "write" => write_to_db(&mut db_settings, &mut file_system, &query),
            "random" => random_from_db(&mut db_settings, &mut file_system, &mut query),
            "" => read_from_db(&mut db_settings, &mut file_system, &query),
            _ => http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("Given function does not exist".to_string()))]).as_str()),
        }
    });

    http_server.listen();
}

fn random_from_db(db_settings: &mut meta::DBSettings, file_system: &mut file::FileSystem, query: &mut query::QueryResult) -> String {
    let mut rng = rand::rng();
    let nr_of_random_values: u64 = match &query.fn_param.parse::<u64>() {
        Ok(v) => *v,
        Err(_) => return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("Incorrect parameter type for random function".to_string()))]).as_str()),

    };
    let biggest_id = db_settings.tables.get(&query.table_name).unwrap().biggest_id;

    match &query.indexes[0] {
        query::IndexType::Index(_) => {
            if query.indexes.len() < nr_of_random_values as usize {
                return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("Attempting to retrieve more random values than the given query includes".to_string()))]).as_str())
            }

            let mut indexes: Vec<u64> = (0..=query.indexes.len() as u64).collect();
            indexes.shuffle(&mut rng);
            indexes.truncate(nr_of_random_values as usize);

            query.indexes = indexes.iter().map(|i| query::IndexType::Index(*i)).collect();
        },
        query::IndexType::Wildcard => {
            if biggest_id + 1 < nr_of_random_values {
                return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("Attempting to retrieve more random values than the table includes".to_string()))]).as_str())
            }

            let mut indexes: Vec<u64> = (0..=biggest_id).collect();
            indexes.shuffle(&mut rng);
            indexes.truncate(nr_of_random_values as usize);

            query.indexes = indexes.iter().map(|i| query::IndexType::Index(*i)).collect();
        }
    }

    file_system.drop_entire_cache();
    read_from_db(db_settings, file_system, query)
}

fn read_from_db(db_settings: &meta::DBSettings, file_system: &mut file::FileSystem, query: &query::QueryResult) -> String {
    let mut result: Vec<(u64, String)> = Vec::new();

    for index in &query.indexes {
        match index {
            query::IndexType::Index(i) => {
                let container = num::integer::div_floor(*i, db_settings.compartment_rows as u64);
                let line = if *i < db_settings.compartment_rows as u64 {*i} else {*i - db_settings.compartment_rows as u64};
                let file_name = format!("./tables/{}/{}", query.table_name, container);

                if !file_system.is_in_cache(file_name.as_str()) {
                    match file_system.open(file_name.as_str()) {
                        Ok(status) => println!("{status:?}"),
                        Err(e) => {
                            println!("{e}");
                            continue; //todo proper error handling here and rollback for changes incase
                                      //of failiure
                        }
                    } 
                }

                match file_system.read_line_from_cache(file_name.as_str(), line as usize) {
                    Ok(content) => {
                        let index = {
                            if line > 0 && container > 0{
                                line * container
                            } else if container == 0 {
                                line
                            } else {
                                container * db_settings.compartment_rows as u64
                            }
                        };
                        result.push((index, content));
                    },
                    Err(e) => {
                        println!("{e}");
                        return http::create_http_response(200, "application/json", json::encode(vec![("error", json::JSONValue::String("Index out of table range".to_string()))]).as_str());
                    }
                }
            },
            query::IndexType::Wildcard => {
                let dir_name = format!("./tables/{}", query.table_name);
                let dir = file_system.read_folder(dir_name.as_str());

                for file in dir {
                    match file {
                        Ok(dir_entry) => {
                            let container = dir_entry.file_name().into_string().unwrap().parse::<u64>().unwrap();
                            let file_name = format!("{}/{}", dir_name, container);
                            match file_system.open(file_name.as_str()) {
                                Ok(_) => println!("Success"),
                                Err(_) => {
                                    println!("File open failed");
                                    continue;
                                },
                            }

                            match file_system.read_from_cache(file_name.as_str()) {
                                Ok(contents) => {
                                    let mut contents_with_index: Vec<(u64, String)> = contents.iter().enumerate().map(|line| 
                                        ({
                                            if line.0 as u64 > 0 && container > 0{
                                                line.0 as u64 * container
                                            } else if container == 0 {
                                                line.0 as u64
                                            } else {
                                                container * db_settings.compartment_rows as u64
                                            }
                                        }, line.1.clone())
                                    ).collect();
                                    result.append(&mut contents_with_index);
                                },
                                Err(_) => {
                                    println!("Read failed");
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

fn encode_db_return(vec: Vec<(u64, String)>, db_settings: &meta::DBSettings, query: &query::QueryResult) -> String {
    let db_cols = &db_settings.tables.get(query.table_name.as_str()).unwrap().columns;
    let mut json_array: Vec<json::JSONValue> = Vec::new();

    for data_row in vec {
        let mut json_object: Vec<(String, json::JSONValue)> = Vec::new();

        json_object.push(("index".to_string(), json::JSONValue::NumI(data_row.0 as i64)));

        for data in util::escape_split(data_row.1.as_str(), ',').iter().enumerate() {
            let col_data = &db_cols[data.0];

            match col_data.value {
                meta::ColValue::NumberI => {
                    json_object.push((col_data.name.clone(), json::JSONValue::NumI(data.1.parse().unwrap())));
                },
                meta::ColValue::NumberDec => {
                    json_object.push((col_data.name.clone(), json::JSONValue::NumDec(data.1.parse().unwrap())));
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

fn write_to_db(db_settings: &mut meta::DBSettings, file_system: &mut file::FileSystem, query: &query::QueryResult) -> String {
    if query.indexes.len() > 1 {
        return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("Data can only be written to one index at a time".to_string()))]).as_str());
    }

    let row_data_split = util::escape_split(query.fn_param.as_str(), ',');
    let columns = &db_settings.tables.get(&query.table_name).unwrap().columns;

    if row_data_split.len() != columns.len() {
        return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("The number of arguments does not match the number of data columns in the database".to_string()))]).as_str());
    }

    for row_data in row_data_split.iter().enumerate() {
        let col_data = &columns[row_data.0];

        match col_data.value {
            meta::ColValue::NumberI => {
                if !row_data.1.parse::<i64>().is_ok() {
                    return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("Data type does not match column data type".to_string()))]).as_str()); // More descriptive error. Include index and correct data type
                }
            },
            meta::ColValue::NumberDec => {
                if !row_data.1.parse::<f64>().is_ok() {
                    return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("Data type does not match column data type".to_string()))]).as_str());  // More descriptive error. Include index and correct data type
                }
            },
            _ => (),
        }
    }

    match &query.indexes[0] {
        query::IndexType::Index(i) => {
            let container = num::integer::div_floor(*i, db_settings.compartment_rows as u64);
            let line = if *i < db_settings.compartment_rows as u64 {*i} else {*i - db_settings.compartment_rows as u64};
            let file_name = format!("./tables/{}/{}", query.table_name, container);

            match file_system.open(file_name.as_str()) {
                Ok(status) => println!("{status:?}"),
                Err(_) => {
                    return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("File open error".to_string()))]).as_str());
                }
            }

            let mut file_data = match file_system.read_from_cache(file_name.as_str()) {
                Ok(f) => f,
                Err(_) => {
                    return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("File read error".to_string()))]).as_str());
                }
            };

            file_data.insert(line as usize, query.fn_param.clone());

            match file_system.write_to_cache(file_name.as_str(), file_data.join("\n")) {
                Ok(s) => println!("{s:?}"),
                Err(_) => {
                    return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("Cache write error".to_string()))]).as_str());
                }
            }
    
            match file_system.write_entire_cache_to_disk() {
                Ok(_) => return http::create_http_response(200, "application/json", json::encode(vec![("error", json::JSONValue::String("Write success".to_string()))]).as_str()),
                Err(_) => return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("Write failed".to_string()))]).as_str()), 
            }
        },
        query::IndexType::Wildcard => {
            let table_max_index = if db_settings.tables.get(&query.table_name).unwrap().biggest_id > 0 {
                db_settings.tables.get(&query.table_name).unwrap().biggest_id + 1
            } else {
                0
            };
            let container = num::integer::div_floor(table_max_index, db_settings.compartment_rows as u64);
            let file_name = format!("./tables/{}/{}", query.table_name, container);

            println!("filename: {file_name}");

            match file_system.open(file_name.as_str()) {
                Ok(status) => println!("{status:?}"),
                Err(_) => {
                    return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("File open error".to_string()))]).as_str());
                }
            }

            let mut file_data = match file_system.read_from_cache(file_name.as_str()) {
                Ok(f) => f,
                Err(_) => {
                    return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("File read error".to_string()))]).as_str());
                }
            };
            
            file_data.push(query.fn_param.clone());

            match file_system.write_to_cache(file_name.as_str(), file_data.join("\n")) {
                Ok(s) => println!("{s:?}"),
                Err(_) => {
                    return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("Cache write error".to_string()))]).as_str());
                }
            }
    
            match file_system.write_entire_cache_to_disk() {
                Ok(_) => {
                    db_settings.iterate_id(&query.table_name);
                    return http::create_http_response(200, "application/json", json::encode(vec![("error", json::JSONValue::String(format!("Write success, new id {table_max_index}")))]).as_str());
                },
                Err(_) => return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("Write failed".to_string()))]).as_str()), 
            }
        },
    }
}

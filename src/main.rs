mod file;
mod meta;
mod http;
mod query;
mod json;

use std::env;

const _TEST_FILE_PATH: &str = "./tables/test";
const _DEFAULT_META_FILE_PATH: &str = "./meta.yaml";

fn main() {
    // Load program arguments
    let args: Vec<String> = env::args().collect();

    let meta_file_path: &str = args[1].as_str();

    // Order of definition is critical,
    // variables used inside endpoints of the http_server
    // need to be defined before the server itself
    let mut file_system = file::FileSystem::new();
    let mut db_settings = meta::load_meta(meta_file_path);
    let mut http_server = http::HTTPServer::new("127.0.0.1:7878".to_string());

    http_server.add_middleware(|_body, _url_params| {
        println!("Execute middleware");
        (true, "Custom middleware failure message".to_string()) // Rewrite to use std::io::Result
    });
    
    http_server.add_endpoint("/", http::HTTPRequestMethods::POST, |body, _url_params| {
        let query = match query::parse_query(body.as_str()) {
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
            _ => read_from_db(&mut db_settings, &mut file_system, &query),
        }
    });

    http_server.listen();
}

fn read_from_db(db_settings: &meta::DBSettings, file_system: &mut file::FileSystem, query: &query::QueryResult) -> String {
    let mut result: Vec<String> = Vec::new();

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
                    Ok(content) => result.push(content),
                    Err(e) => {
                        println!("{e}");
                        continue;
                    }
                }
            },
            query::IndexType::Wildcard => {
                let dir_name = format!("./tables/{}", query.table_name);
                let dir = file_system.read_folder(dir_name.as_str());

                for file in dir {
                    match file {
                        Ok(dir_entry) => {
                            let file_name = format!("{}/{}", dir_name, dir_entry.file_name().into_string().unwrap());
                            match file_system.open(file_name.as_str()) {
                                Ok(_) => println!("Success"),
                                Err(_) => {
                                    println!("File open failed");
                                    continue;
                                },
                            }

                            match file_system.read_from_cache(file_name.as_str()) {
                                Ok(contents) => {
                                    result.append(&mut contents.clone());
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

fn encode_db_return(vec: Vec<String>, db_settings: &meta::DBSettings, query: &query::QueryResult) -> String {
    let db_cols = &db_settings.tables.get(query.table_name.as_str()).unwrap().columns;
    let mut json_array: Vec<json::JSONValue> = Vec::new();

    for data_row in vec {
        let mut json_object: Vec<(String, json::JSONValue)> = Vec::new();

        for data in data_row.split(',').enumerate() {
            let col_data = &db_cols[data.0];

            json_object.push((col_data.name.clone(), json::JSONValue::String(data.1.to_string())));
        }
        json_array.push(json::JSONValue::Object(json_object));
    }

    json::encode(vec![("data", json::JSONValue::Array(json_array))])
}

fn write_to_db(db_settings: &mut meta::DBSettings, file_system: &mut file::FileSystem, query: &query::QueryResult) -> String {
    if query.indexes.len() > 1 {
        return http::create_http_response(400, "application/json", json::encode(vec![("error", json::JSONValue::String("Data can only be written to one index at a time".to_string()))]).as_str());
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

mod file;
mod meta;
mod http;
mod query;

const _TEST_FILE_PATH: &str = "./tables/test";
const META_FILE_PATH: &str = "./meta.yaml";

fn main() {
    // Order of definition is critical,
    // variables used inside endpoints of the http_server
    // need to be defined before the server itself
    let mut file_system = file::FileSystem::new();
    let mut db_settings = meta::load_meta(META_FILE_PATH);
    let mut http_server = http::HTTPServer::new("127.0.0.1:7878".to_string());
    
    http_server.add_endpoint("/", http::HTTPRequestMethods::POST, |body| {
        let query = match query::parse_query(body.as_str()) {
            Ok(query) => query,
            Err(e) => {
                println!("{e}");
                return http::create_http_response(400, "application/json", "\"err\":\"Query could not be parsed\"");
            }
        };

        println!("{query:?}");

        if !db_settings.table_exists(&query.table_name) {
            return http::create_http_response(400, "application/json", "\"err\":\"Given table does not exist\"");
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
    http::create_http_response(200, "application/json", result.join("\n").as_str())
}

fn write_to_db(db_settings: &mut meta::DBSettings, file_system: &mut file::FileSystem, query: &query::QueryResult) -> String {
    if query.indexes.len() > 1 {
        return http::create_http_response(400, "application/json", "\"err\":\"data can only be written to one index at a time\"");
    }

    match &query.indexes[0] {
        query::IndexType::Index(i) => {
            let container = num::integer::div_floor(*i, db_settings.compartment_rows as u64);
            let line = if *i < db_settings.compartment_rows as u64 {*i} else {*i - db_settings.compartment_rows as u64};
            let file_name = format!("./tables/{}/{}", query.table_name, container);

            match file_system.open(file_name.as_str()) {
                Ok(status) => println!("{status:?}"),
                Err(_) => {
                    return http::create_http_response(400, "application/json", "\"err\":\"file open error\"");
                }
            }

            let mut file_data = match file_system.read_from_cache(file_name.as_str()) {
                Ok(f) => f,
                Err(_) => {
                    return http::create_http_response(400, "application/json", "\"err\":\"file read error\"");
                }
            };

            file_data.insert(line as usize, query.fn_param.clone());

            match file_system.write_to_cache(file_name.as_str(), file_data.join("\n")) {
                Ok(s) => println!("{s:?}"),
                Err(_) => {
                    return http::create_http_response(400, "application/json", "\"err\":\"cache write error\"");
                }
            }
    
            match file_system.write_entire_cache_to_disk() {
                Ok(_) => return http::create_http_response(200, "application/json", "\"status\":\"write success\""),
                Err(_) => return http::create_http_response(400, "application/json", "\"err\":\"write failed\""), 
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
                    return http::create_http_response(400, "application/json", "\"err\":\"file open error\"");
                }
            }

            let mut file_data = match file_system.read_from_cache(file_name.as_str()) {
                Ok(f) => f,
                Err(_) => {
                    return http::create_http_response(400, "application/json", "\"err\":\"file read error\"");
                }
            };
            
            file_data.push(query.fn_param.clone());

            match file_system.write_to_cache(file_name.as_str(), file_data.join("\n")) {
                Ok(s) => println!("{s:?}"),
                Err(_) => {
                    return http::create_http_response(400, "application/json", "\"err\":\"cache write error\"");
                }
            }
    
            match file_system.write_entire_cache_to_disk() {
                Ok(_) => {
                    db_settings.iterate_id(&query.table_name);
                    return http::create_http_response(200, "application/json", format!("\"status\":\"write success new id {table_max_index}\"").as_str());
                }
                Err(_) => return http::create_http_response(400, "application/json", "\"err\":\"write failed\""), 
            }
        },
    }
}

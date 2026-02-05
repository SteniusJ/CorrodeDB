mod file;
mod meta;
mod http;
mod query;
mod json;
mod util;
mod db_engine;

use std::env;
use std::collections::HashMap;

const DEFAULT_SCHEMA_FILE_PATH: &str = "./schema.yaml";
const DEFAULT_PORT: &str = "4067";

pub struct ProgramArgs {
    pub schema_path: String,
    pub port: String,
    pub data_integrity_check: bool,
}

/// Loads program arguments
pub fn load_program_arguments() -> ProgramArgs {
    let mut program_args = ProgramArgs {
        schema_path: String::from(DEFAULT_SCHEMA_FILE_PATH),
        port: String::from(DEFAULT_PORT),
        data_integrity_check: false,
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
            "-di" => {
                if value == "true" {
                    program_args.data_integrity_check = true;
                } else {
                    println!("data integrity has to be called with the value 'true' to have any effect");
                }
            },
            f=> {
                println!("flag {f} is not a valid flag");
            },
        }
    }

    program_args
}

/// Utility function for checking integrity of database data
pub fn data_integrity_check(schema_path: &str) {
    let mut file_system = file::FileSystem::new();
    let db_settings = meta::load_meta(schema_path);

    println!("--------------- Starting data integrity check ---------------\n");
    for (table, table_settings) in db_settings.tables {
        println!("Checking data for table: {table}");

        let dir_path = format!("./tables/{table}");
        let Ok(dir) = file_system.read_folder(dir_path.as_str()) else {
            panic!("table '{table}' does not have a folder");
        };

        for file in dir {
            let file = file.unwrap();
            let file_name = file.file_name().into_string().unwrap();

            if let Ok(file_content) = util::file_read(format!("./tables/{table}/{file_name}").as_str(), &mut file_system) {
                for (index, line) in file_content.iter().enumerate() {
                    if line.is_empty() {
                        continue;
                    } 

                    let columns_data = util::escape_split(line.as_str(), ',');

                    if columns_data.len() != table_settings.columns.len() {
                        println!("! {table}: {file_name} - error on line {} - number of columns does not match!", index + 1);
                    }

                    for (column_index, column_data) in columns_data.iter().enumerate() {
                        if column_index >= table_settings.columns.len() {
                            break;
                        }

                        match table_settings.columns[column_index].value {
                            meta::ColValue::NumberI => {
                                if column_data.parse::<i64>().is_err() {
                                    println!("! {table}: {file_name} - error on line {} - datatype does not match, expected NumberI for column {column_index}!", index + 1);
                                }
                            },
                            meta::ColValue::NumberF => {
                                if column_data.parse::<f64>().is_err() {
                                    println!("! {table}: {file_name} - error on line {} - datatype does not match, expected NumberF for column {column_index}!", index + 1);
                                }
                            },
                            meta::ColValue::VarChar => (),
                        }
                    }
                }
            } else {
                panic!("failed to read file {file_name}");
            } 
        }

        println!("Finished checking data for table: {table}\n");
    }
}

/// Starts database server
pub fn start_database(schema_path: &str, port: &str) {
    // Order of definition is critical,
    // variables used inside endpoints of the http_server
    // need to be defined before the server itself
    let db_settings = meta::load_meta(schema_path); // shouldn't be needed here
    let mut db_engine = db_engine::DBEngine::new(schema_path);
    let db_password = db_settings.password.clone(); // make some new way to get db_password maybe util function?
    let mut http_server = http::HTTPServer::new(format!("127.0.0.1:{port}"));

    http_server.add_middleware(|_body, url_params| {
        if !url_params.contains_key("password") {
            return (false, "Password url parameter required".to_string());
        }
 
        if url_params.get("password").unwrap() == &db_password { // safe to assume value is Some
            return (true, String::new());
        }
        (false, "Given password is incorrect".to_string())
    });

    http_server.add_endpoint("/", http::HTTPRequestMethods::POST, |body, _url_params| {
        let query = match query::parse_query(body.as_str()) {
            Ok(query) => query,
            Err(e) => {
                println!("Query parse error: {e}");
                return http::create_http_response(400, "application/json",  json::encode(vec![("error", json::JSONValue::String("Query could not be parsed".to_string()))]).as_str());
            }
        };

        println!("{query}");

        match db_engine.query(&query) {
            Ok(result) => return http::create_http_response(200, "application/json",  encode_db_return(result).as_str()),
            Err(_) => return http::create_http_response(400, "application/json",  json::encode(vec![("error", json::JSONValue::String("Generic error".to_string()))]).as_str()),
        }
    });

    http_server.listen();
}

/// Encodes returned data from file read into a json string
fn encode_db_return(vec: Vec<HashMap<String, db_engine::DBDatatype>>) -> String {
    let mut json_array: Vec<json::JSONValue> = Vec::new();

    for data_row in vec {
        /*
        if data_row.1.is_empty() {
            continue;
        }
        */

        let mut json_object: Vec<(String, json::JSONValue)> = Vec::new();

        for (col_name, content) in data_row {
            match content {
                db_engine::DBDatatype::NumberI(v) => json_object.push((col_name, json::JSONValue::NumI(v))),
                db_engine::DBDatatype::NumberF(v) => json_object.push((col_name, json::JSONValue::NumF(v))),
                db_engine::DBDatatype::VarChar(v) => json_object.push((col_name, json::JSONValue::String(util::remove_escape_characters(v)))),
            }
        }

        /*
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
        */
        json_array.push(json::JSONValue::Object(json_object));
    }

    json::encode(vec![("data", json::JSONValue::Array(json_array))])
}

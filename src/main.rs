mod file;
mod meta;
mod http;
mod query;

const _TEST_FILE_PATH: &str = "./tables/test";
const META_FILE_PATH: &str = "./meta.yaml";

fn main() {
    let mut file_system = file::FileSystem::new();
    let mut http_server = http::HTTPServer::new("127.0.0.1:7878".to_string());
    let db_settings = meta::load_meta(META_FILE_PATH);

    http_server.add_endpoint("/", http::HTTPRequestMethods::POST, {
        fn ep(body: String) -> String {
            let query = match query::parse_query(body.as_str()) {
                Ok(query) => query,
                Err(e) => {
                    println!("{e}");
                    return http::create_http_response(400, "application/json", "\"err\":\"Query could not be parsed\"");
                }
            };

            println!("{query:?}");

            if !db_settings.table_exists(&query.0) {
                return http::create_http_response(400, "application/json", "\"err\":\"Given table does not exist\"");
            }

            String::new()
        }
        ep
    });

    /* For reference!!!!
    http_server.add_endpoint("/".to_string(), http::HTTPRequestMethods::POST, {
        fn ep(content: String) -> String {
            http::create_http_response(200, "application/json", format!("{{\"msg\":\"you sent '{}' to the server\"}}", content).as_str())
        }
        ep
    });
    */

    // testing of db query chain
    let res = query::parse_query("messages[10]").unwrap();
    println!("{res:?}");

    let table_existence = db_settings.table_exists(&res.0); // checks if table exists
    println!("{table_existence:?}");
    
    match file_system.open("./tables/messages/0") {
        Ok(_) => (),
        Err(e) => {
            println!("{e}");
        }
    }

    match file_system.read_line_from_cache("./tables/messages/0", 2) {
        Ok(string) => {
            println!("{string}");
        },
        Err(e) => {
            println!("{e}");
        }
    }

    let res = query::parse_query("messages[10,11,2].random()").unwrap();
    println!("{res:?}");

    http_server.listen();
}

/*
fn main() {
    // Init new filesystem
    let mut file_system = file::FileSystem::new();

    // Open meta file and read contents, remove from cache when done
    match file_system.open(META_FILE_PATH) {
        Ok(status) => println!("Meta file open status: {status:?}"),
        Err(e) => {
            println!("Meta file open failed: {e:?}");
            std::process::exit(0x0100);
        }
    }

    let test_config_yaml: String;

    match file_system.read_from_cache(META_FILE_PATH) {
        Ok(contents) => {
            println!("Meta file read success");
            test_config_yaml = contents.join("\n");
        },
        Err(e) => {
            println!("File read failed: {e:?}");
            std::process::exit(0x0100);
        },
    }

    match file_system.drop_from_cache(META_FILE_PATH) {
        Ok(status) => println!("Meta file removed from cache: {status:?}"),
        Err(e) => println!("Meta file drop failed: {e:?}"),
    }

    println!("-----------------------------");

    // Init db_settings struct and print data for debug purposes
    let db_settings = meta::DBSettings::new(test_config_yaml.as_str());

    let max_size = db_settings.cache_max_size;
    let life_time = db_settings.cache_life_time;
    let compartment_rows = db_settings.compartment_rows;

    println!("max size: {max_size:?}\nlife time: {life_time:?}\ncompartment rows: {compartment_rows:?}\n------------------------------\n");

    let tables = &db_settings.tables;

    println!("Tables: {tables:?}\n-----------------------------\n");

    let rows = &tables.get("messages").unwrap().rows;

    println!("Rows: {rows:?}\n------------------------\n");

    // Test all file system funtions by:
    //
    // Creating folder
    // Opening file
    // Reading content
    // Writing to cache
    // Reading data written to cache
    // Try to open file again
    // Write file cache to disk
    match file_system.create_folder("./tables") {
        Ok(status) => println!("Dir creation status: {status:?}"),
        Err(e) => println!("Dir create failed: {e:?}"),
    }

     match file_system.open(TEST_FILE_PATH) {
         Ok(status) => println!("File open status: {status:?}"),
         Err(e) => println!("file open failed: {e:?}"),
     }

     match file_system.read_from_cache(TEST_FILE_PATH) {
         Ok(contents) => println!("Read return: {contents:?}"),
         Err(e) => println!("Read failed: {e:?}"),
     }

     match file_system.write_to_cache(TEST_FILE_PATH, "New content in cache\nrow2\nrow3".to_string()) {
         Ok(s) => println!("Write status: {s:?}"),
         Err(e) => println!("Write fail: {e:?}"),
     }

     match file_system.read_from_cache(TEST_FILE_PATH) {
         Ok(contents) => println!("Read return: {contents:?}"),
         Err(e) => println!("Read failed: {e:?}"),
     }

     match file_system.open(TEST_FILE_PATH) {
         Ok(status) => println!("File open status: {status:?}"),
         Err(e) => println!("file open failed: {e:?}"),
     }

     match file_system.write_cache_to_disk(TEST_FILE_PATH) {
         Ok(s) => println!("File disk write status: {s:?}"),
         Err(e) => println!("File open failed: {e:?}"),
     }

     // Http server testing
     let mut http_server = http::HTTPServer::new("127.0.0.1:7878".to_string());

     fn endpoint1(_: String) -> String {
         http::create_http_response(200, "application/json", "{\"msg\":\"hello from rust!\"}")
     }
     http_server.add_endpoint("/".to_string(), http::HTTPRequestMethods::GET, endpoint1);

     fn post_endpoint(body: String) -> String {
         match body.parse::<i64>() {
             Ok(num) => http::create_http_response(200, "application/json", format!("{{\"msg\": \"string: '{}' is a number!\"}}", num).as_str()),
             Err(_) => http::create_http_response(404, "application/json", "{\"msg\":\"not a number!\"}"),
         }
     }
     http_server.add_endpoint("/".to_string(), http::HTTPRequestMethods::POST, post_endpoint);

     http_server.listen();
}
*/


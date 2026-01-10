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
    let db_settings = meta::load_meta(META_FILE_PATH);
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

        read_from_db(&db_settings, &mut file_system, &query)
    });

    http_server.listen();
}

fn read_from_db(db_settings: &meta::DBSettings, file_system: &mut file::FileSystem, query: &query::QueryResult) -> String {
    let mut result: Vec<String> = Vec::new();
    for index in &query.indexes {
        let container = num::integer::div_floor(*index, db_settings.compartment_rows as u64);
        let line = if *index < db_settings.compartment_rows as u64 {*index} else {*index - db_settings.compartment_rows as u64};
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
    }

    file_system.drop_entire_cache();
    http::create_http_response(200, "application/json", result.join("\n").as_str())
}

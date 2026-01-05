mod file;
mod meta;
mod http;
mod query;
mod db_manager;

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

        if !db_settings.table_exists(&query.0) {
            return http::create_http_response(400, "application/json", "\"err\":\"Given table does not exist\"");
        }

        // calculate container and which line in said container has the value
        // TODO! apply this calculation to all indexes given
        let container = num::integer::div_floor(query.1[0], db_settings.compartment_rows as u64);
        let line = if query.1[0] < db_settings.compartment_rows as u64 {query.1[0]} else {query.1[0] - db_settings.compartment_rows as u64};

        match file_system.open(format!("./tables/{}/{}", &query.0, container).as_str()) {
            Ok(status) => println!("{status:?}"),
            Err(e) => {
                println!("{e}");
                return http::create_http_response(400, "application/json", "\"err\":\"file open error\"");
            }
        }
        
        match file_system.read_line_from_cache(format!("./tables/{}/{}", &query.0, container).as_str(), line as usize) {
            Ok(content) => {
                println!("{content}");
                return http::create_http_response(200, "application/json", format!("\"res\":\"{}\"", content).as_str());
            },
            Err(e) => {
                println!("{e}");
                return http::create_http_response(400, "application/json", "\"err\":\"line read error\"");
            }
        }
    });
}

use std::env;
use db_project::start_database;

mod util;

const DEFAULT_SCHEMA_FILE_PATH: &str = "./schema.yaml";
const DEFAULT_PORT: &str = "4067";

fn main() {
    // Read program arguments
    let args = util::parse_program_args(env::args().collect());

    let mut schema_file_path: String = DEFAULT_SCHEMA_FILE_PATH.to_string();
    let mut port: String = DEFAULT_PORT.to_string();

    for (flag, value) in args {
        match flag.as_str() {
            "-s" => {
                schema_file_path = value;
            },
            "-p" => {
                port = value;
            },
            f=> {
                panic!("flag {f} is not a valid flag");
            },
        }
    }

    start_database(schema_file_path.as_str(), port.as_str());
}

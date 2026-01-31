use std::env;
use db_project::start_database;

const _DEFAULT_META_FILE_PATH: &str = "./meta.yaml";

fn main() {
    // Read program arguments
    let args: Vec<String> = env::args().collect();

    let meta_file_path: &str = args[1].as_str();

    start_database(meta_file_path);
}

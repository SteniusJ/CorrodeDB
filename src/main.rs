use CorrodeDB::{start_database, load_program_arguments};

fn main() {
    let program_args = load_program_arguments();

    start_database(program_args.schema_path.as_str(), program_args.port.as_str());
}

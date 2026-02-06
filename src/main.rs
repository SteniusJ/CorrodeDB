use corrode_db::{start_database, load_program_arguments, data_integrity_check, start_databese_console_queries_mode};

fn main() {
    let program_args = load_program_arguments();

    if program_args.data_integrity_check {
        data_integrity_check(program_args.schema_path.as_str());
        return;
    }

    if program_args.console_queries {
        start_databese_console_queries_mode(program_args.schema_path.as_str());
        return;
    }

    start_database(program_args.schema_path.as_str(), program_args.port.as_str());
}

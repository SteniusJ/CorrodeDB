use corrode_db::{
    data_integrity_check, load_program_arguments, start_database,
    start_databese_console_queries_mode,
};

fn main() {
    let program_args = load_program_arguments();

    if program_args.data_integrity_check {
        data_integrity_check(&program_args.schema_path);
        return;
    }

    if program_args.console_queries {
        start_databese_console_queries_mode(&program_args.schema_path);
        return;
    }

    start_database(program_args.schema_path.as_str(), &program_args.port);
}

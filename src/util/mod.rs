pub fn escape_split(input: &str, split_char: char) -> Vec<&str> {
    let mut skip = false;
    let mut last_split_i = 0;
    let mut splits: Vec<&str> = Vec::new();

    for char in input.chars().enumerate() {
        if skip {
            skip = false;
            continue;
        }
        
        if char.1 == '\\' {
            skip = true;
            continue;
        }

        if char.1 == split_char {
            splits.push(input.get(last_split_i..char.0).unwrap());
            last_split_i = char.0 + 1;
        }
    }

    splits.push(input.get(last_split_i..input.len()).unwrap());
    splits
}

pub fn remove_escape_characters(input: String) -> String {
    input.replace("\\", "")
}

pub fn parse_program_args(args: Vec<String>) -> Vec<(String, String)>{
    let mut parsed_args: Vec<(String, String)> = Vec::new();
    let mut flag = String::new();

    for arg in args {
        if arg == "target/debug/db-project" { // temp fix, skips debug argument which is
                                              // automatically sent
            continue;
        }

        if flag.is_empty() {
            if !is_flag(&arg) {
                panic!("expected flag in program arguments, please give arguments in the format '[flag] [value]'");
            }
            flag = arg;
            continue;
        }

        if is_flag(&arg) {
            panic!("expected value in program arguments, please give arguments in the format '[flag] [value]'")
        }

        parsed_args.push((flag, arg));
        flag = String::new();
    }

    parsed_args
}

fn is_flag(argument: &String) -> bool {
    argument.chars().next().unwrap() == '-'
}

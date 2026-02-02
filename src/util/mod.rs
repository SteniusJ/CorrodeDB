use regex::Regex;

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
    let arguments_string = args.join(" ");
    let re = Regex::new(r"(?<flag>-[[:alpha:]]*) (?<param>[[:ascii:]-- -]*)").unwrap();

    let mut parsed_args: Vec<(String, String)> = Vec::new();

    let captures = re.captures_iter(&arguments_string);

    for argument in captures {
        parsed_args.push((argument["flag"].to_string(), argument["param"].to_string()));
    }

    parsed_args
}

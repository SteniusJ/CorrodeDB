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

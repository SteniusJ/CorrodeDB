use regex::Regex;

pub fn parse_query(query_str: &str) {
    //let functions: Vec<&str> = query_str.split(".").collect();

    let re = Regex::new(r"([^\W]*)\(([[:alnum:]\\',. ]*)\)").unwrap();

    for item in re.captures_iter(query_str) {
        println!("{item:?}");
    }
}
/*
    for function in functions.iter() {
        let mut fn_name: String = String::new();
        let mut params: Vec<&str> = Vec::new();

        let mut char_iter = function.chars().into_iter();

        // Replace some loops with this regex: ([^\W]*)\(([[:alnum:]\\',. ]*)\)
        



        loop {
            let char = match char_iter.next() {
                Some(char) => char,
                None => break,
            };

            match char {
                '(' => {
                    params = parse_params(char_iter.clone());
                    break;
                },
                _ => fn_name.push(char),
            }
        }

        println!("{fn_name}");
        println!("{params:?}");
        /*
        for char in function.chars() {
            match char {
                '(' => building_params = true,
                ')' => continue,
                ',' => (),
                _ => {
                    if building_params {
                        param.push(char);
                        continue;
                    }

                    fn_name.push(char);
                },
            }
        }
        println!("{fn_name}");
        */
    }
}

fn parse_params(char_iter: std::str::Chars<'_>) -> Vec<&str> {
    let params: Vec<&str> = Vec::new();

    params
}
*/

use std::collections::HashSet;
use std::option::Option;
use crate::{util};

#[derive(Debug)]
pub enum Token {
    String(String),
    Integer(i64),
    FloatingPoint(f64),
    Page(u64),
    Wildcard,
    BiggerThen,
    LessThen,
    Equals,
    Includes,
    Pipe,
    SortAscending,
    SortDescending,
    Range((u64, u64)),
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    let split_character_set = HashSet::from([' ', ',', '[', ']']);
    let splits = util::escape_split_by_many(input, split_character_set);

    for part in splits {
        match part {
            ">" => tokens.push(Token::BiggerThen),
            "<" => tokens.push(Token::LessThen),
            "=" => tokens.push(Token::Equals),
            "|" => tokens.push(Token::Pipe),
            "asc" => tokens.push(Token::SortAscending),
            "dsc" => tokens.push(Token::SortDescending),
            "in" => tokens.push(Token::Includes),
            "*" => tokens.push(Token::Wildcard),
            part => {
                if let Ok(number) = part.parse::<i64>() {
                    tokens.push(Token::Integer(number));
                    continue;
                }
                if let Ok(number) = part.parse::<f64>() {
                    tokens.push(Token::FloatingPoint(number));
                    continue;
                }
                if let Some(page) = try_into_page(part) {
                    tokens.push(page);
                    continue;
                }
                if let Some(range) = try_into_range(part) {
                    tokens.push(range);
                    continue;
                }

                tokens.push(Token::String(part.to_string()));
            }
        }
    }

    tokens
}

fn try_into_range(input: &str) -> Option<Token> {
    let range_split: Vec<&str> = input.split("..").collect();
    if range_split.len() != 2 {
        return None;
    }
    if let (Ok(range_start), Ok(range_end)) = (range_split[0].parse::<u64>(), range_split[1].parse::<u64>()) {
        return Some(Token::Range((range_start, range_end)));
    }
    None
}

fn try_into_page(input: &str) -> Option<Token> {
    let mut input_iter = input.chars().into_iter();

    if input_iter.next() != Some('p') {
        return None;
    }

    let mut index_string = String::with_capacity(input_iter.size_hint().1.unwrap_or(0));
    input_iter.for_each(|char| index_string.push(char));

    if let Ok(index) = index_string.parse::<u64>() {
        return Some(Token::Page(index));
    }
    None
}

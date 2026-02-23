use std::collections::HashSet;
use std::option::Option;
use crate::{util, query};

#[derive(Debug, PartialEq)]
pub enum Token {
    String(String),
    Integer(i64),
    FloatingPoint(f64),
    Page(u64),
    Wildcard(u64), // value for wildcard is modifier, if it is 10 it equates to *-10
    BiggerThen,
    LessThen,
    Equals,
    Includes,
    Pipe,
    SortAscending,
    SortDescending,
    Range((query::IndexType, query::IndexType)),
}

impl Token {
    pub fn is_valid_index(&self) -> bool {
        match self {
            Token::Integer(self_v) => {
                if self_v >= &0 {
                    true
                } else {
                    false
                }
            },
            Token::Wildcard(_) => true,
            Token::Range(_) => true,
            Token::Page(_) => true,
            _ => false,
        }
    }
    pub fn is_wildcard(&self) -> bool {
        if let Token::Wildcard(_) = self {
            return true;
        }
        false
    }
    pub fn is_int(&self) -> bool {
        if let Token::Integer(_) = self {
            return true;
        }
        false
    }
    pub fn is_float(&self) -> bool {
        if let Token::FloatingPoint(_) = self {
            return true;
        }
        false
    }
    pub fn is_string(&self) -> bool {
        if let Token::String(_) = self {
            return true;
        }
        false
    }
    pub fn is_where_operator(&self) -> bool {
        match self {
            Token::BiggerThen => true,
            Token::LessThen => true,
            Token::Equals => true,
            Token::Includes => true,
            _ => false,
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            Token::Pipe => String::from("|"),
            Token::Page(page) => format!("Page({page})"),
            Token::Equals => String::from("=="),
            Token::Range((start, end)) => format!("Range({start:?}..={end:?})"),
            Token::String(string) => string.clone(),
            Token::LessThen => String::from("<"),
            Token::Includes => String::from("Includes"),
            Token::Integer(int) => int.to_string(),
            Token::Wildcard(modifier) => format!("*-{modifier}"),
            Token::BiggerThen => String::from(">"),
            Token::SortAscending => String::from("Sort(asc)"),
            Token::SortDescending => String::from("Sort(dsc)"),
            Token::FloatingPoint(float) => float.to_string(),
        }
    }
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
            "*" => tokens.push(Token::Wildcard(0)),
            part => {
                if let Ok(number) = part.parse::<i64>() {
                    tokens.push(Token::Integer(number));
                    continue;
                }
                if let Ok(number) = part.parse::<f64>() {
                    tokens.push(Token::FloatingPoint(number));
                    continue;
                }
                if let Some(token) = try_subtraction(part) {
                    tokens.push(token);
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

    if let (Some(range_start), Some(range_end)) = (index_from_str(range_split[0]), index_from_str(range_split[1])) {
        return Some(Token::Range((range_start, range_end)));
    }
    None
}

fn index_from_str(from: &str) -> Option<query::IndexType> {
    if from == "*" {
        return Some(query::IndexType::Wildcard(0));
    }
    if let Some(token) = try_subtraction(from) {
        return Some(query::IndexType(token));
    }
    if let Ok(index) = from.parse::<i64>() {
        return Some(Token::Integer(index));
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

fn try_subtraction(input: &str) -> Option<Token> {
    let subtraction_split: Vec<&str> = input.split("-").collect();
    if subtraction_split.len() != 2 {
        return None;
    }
    if let Ok(subtractor) = subtraction_split[1].parse::<u64>() {
        if subtraction_split[0] == "*" {
            return Some(Token::Wildcard(subtractor));
        }
        if let Ok(subtractee) = subtraction_split[0].parse::<u64>() {
            if subtractee > subtractor {
                return Some(Token::Integer(subtractee as i64 - subtractor as i64));
            }
        }
    }

    None
}

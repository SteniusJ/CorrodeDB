use crate::{query, meta, util};
use crate::db_engine::DBDatatype;
use std::collections::HashMap;
use std::io::{Result, Error, ErrorKind};
use rand::prelude::*;

pub fn sort_by(data: &mut Vec<HashMap<String, DBDatatype>>, query: &query::QueryResult, params: &Vec<String>, db_settings: &meta::DBSettings) -> Result<()> {
    if params.len() != 2 {
        return Err(Error::new(ErrorKind::InvalidInput, "sort sub function accepts 2 parameters"));
    }

    if !db_settings.tables.get(&query.table_name).unwrap().has_column(params[0].to_string()) {
        return Err(Error::new(ErrorKind::InvalidInput, format!("table {} does not have a column called {}", query.table_name, params[0])));
    }

    match util::merge_sort(data, &params[1], &params[0], 0, data.len() - 1) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

/// random function logic
///
/// Returns random values from database
pub fn random_from_db(data: &mut Vec<HashMap<String, DBDatatype>>, _query: &query::QueryResult, params: &Vec<String>, _db_settings: &meta::DBSettings) -> Result<()> {
    if params.len() != 1 {
        return Err(Error::new(ErrorKind::InvalidInput, "Random accepts 1 parameter"));
    }

    let mut rng = rand::rng();
    let Ok(nr_of_random_values) = params[0].parse::<u64>() else {
        if params[0] == "*" {
            data.shuffle(&mut rng);
            return Ok(());
        }
        return Err(Error::new(ErrorKind::InvalidInput, "Incorrect parameter type for random function"));
    };

    if data.len() < nr_of_random_values as usize {
        return Err(Error::new(ErrorKind::InvalidInput, "Attempting to retrieve more values than the query includes"));
    }

    data.shuffle(&mut rng);
    if data.len() == nr_of_random_values as usize {
        return Ok(())
    }
    data.truncate(nr_of_random_values as usize);

    Ok(())
}

/// Where function logic
///
/// Returns data which matches condition
pub fn where_from_db(data: &mut Vec<HashMap<String, DBDatatype>>, query: &query::QueryResult, params: &Vec<String>, db_settings: &meta::DBSettings) -> Result<()> {
    let mut arguments = params.clone().into_iter();

    // Assign variables necessary for conditon matching
    let column = {
        let Some(column) = arguments.next() else {
            return Err(Error::new(ErrorKind::InvalidInput, "Please give column name"));
        };
        if column.is_empty() {
            return Err(Error::new(ErrorKind::InvalidInput, "Please give column name"));
        }
        if !db_settings.tables.get(&query.table_name).unwrap().has_column(column.to_string()) && &column != "index" {
            return Err(Error::new(ErrorKind::InvalidInput, format!("Column {} does not exist in table {}", column, query.table_name)));
        }
        column
    };
    let operator = {
        let Some(operator) = arguments.next() else {
            return Err(Error::new(ErrorKind::InvalidInput, "Please give a operator"));
        };
        if operator.is_empty() {
            return Err(Error::new(ErrorKind::InvalidInput, "Please give a operator"));
        }
        operator
    };
    let comparison_value = {
        let Some(comparison_value) = arguments.next() else {
            return Err(Error::new(ErrorKind::InvalidInput, "Please give comparison value"));
        };
        if column.is_empty() {
            return Err(Error::new(ErrorKind::InvalidInput, "Please give column name"));
        }
        DBDatatype::from_str(&comparison_value)
    };

    let mut index: usize = 0;
    while data.len() > index {
        let column_data = data[index].get(&column).unwrap();

        if !column_data.compare_type(&comparison_value) {
            return Err(Error::new(ErrorKind::InvalidInput, "Comparison value type and column value type do not match"));
        }

        match is_matching(column_data, &comparison_value, &operator) {
            Ok(matching) => {
                if !matching {
                    data.remove(index);
                    continue;
                }
                index += 1;
            },
            Err(e) => return Err(e),
        }
    }

    Ok(())
}

/// Checks if condition is matching
/// Returns Ok(true) if condition matches
/// Returns Ok(false) if condition is not matching
/// Returns Err(_) on incorrect operator value pair
fn is_matching(column_content: &DBDatatype, match_value: &DBDatatype, operator: &str) -> Result<bool> {
    match operator {
        ">" => {
            if column_content > match_value {
                return Ok(true);
            }
            return Ok(false);
        },
        "<" => {
            if column_content < match_value {
                return Ok(true);
            }
            return Ok(false);
        },
        "=" => {
            if column_content == match_value {
                return Ok(true);
            }
            return Ok(false);
        },
        "in" => {
            if let DBDatatype::VarChar(match_value) = match_value {
                if column_content.contains(match_value) {
                    return Ok(true);
                }
                return Ok(false);
            } else {
                return Err(Error::new(ErrorKind::Other, "cannot use in operator on Number value"));
            }
        },
        op => return Err(Error::new(ErrorKind::InvalidInput, format!("{op} is not a valid operator"))),
    }
}

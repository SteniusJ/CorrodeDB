use std::io::{Result, Error, ErrorKind};
use std::collections::HashMap;
use crate::db_engine::DBDatatype;
use crate::{util, query, file, meta};

/// Reads data from database
/// Default functionality
pub fn read_from_db(db_settings: &mut meta::DBSettings, file_system: &mut file::FileSystem, query: &query::QueryResult) -> Result<Vec<HashMap<String, DBDatatype>>> {
    let mut result: Vec<HashMap<String, DBDatatype>> = Vec::new();
    let col_settings = &db_settings.tables.get(&query.table_name).unwrap().columns;

    for index in &query.indexes {
        match index {
            query::IndexType::Index(i) => {
                let (line, file_name, index) = util::get_line_fname_idx(db_settings, query, *i);

                match util::read_line(&file_name, file_system, line) {
                    Ok(content) => {
                        result.push(util::parse_db_line(content, index, &col_settings));
                    }
                    Err(e) if e.kind() == ErrorKind::Other => (),
                    Err(_) => {
                        return Err(Error::new(ErrorKind::Other, "Index out of table range"));
                    }
                };
            },
            query::IndexType::Wildcard => {
                let dir_name = format!("./tables/{}", query.table_name);
                let Ok(dir) = file_system.read_folder(&dir_name) else {
                    panic!("Critical failiure! Table '{}' does not have a folder", query.table_name);
                };
                let mut containers: Vec<u64> = dir.map(|res_dir_entry| 
                    res_dir_entry.unwrap()
                        .file_name()
                        .to_str()
                        .unwrap()
                        .parse::<u64>()
                        .unwrap())
                    .collect();
                containers.sort(); // This is necessary to make sure that the indexes in the
                                    // response are in order. Looping trough ReadDir yields in a
                                    // iterator which is not always in order.

                for container in containers {
                    let file_name = format!("{}/{}", dir_name, container);
                    match file_system.open(&file_name) {
                        Ok(_) => (),
                        Err(e) if e.kind() == ErrorKind::InvalidInput => (),
                        Err(e) => {
                            println!("File open failed: {e}");
                            continue;
                        },
                    }

                    match file_system.read_from_cache(&file_name) {
                        Ok(contents) => {
                            for (line_index, content) in contents.iter().enumerate() {
                                if !content.is_empty() {
                                    result.push(util::parse_db_line(content.clone(), util::get_index(line_index as u64, container, db_settings), &col_settings));
                                }
                            }
                        },
                        Err(e) => {
                            println!("Read failed: {e}");
                            continue;
                        },
                    }
                }
            },
        }
    }

    file_system.drop_entire_cache();
    Ok(result)
}

/// write function logic
/// Writes data to database
pub fn write_to_db(db_settings: &mut meta::DBSettings, file_system: &mut file::FileSystem, query: &query::QueryResult) -> Result<String> {
    if query.indexes.len() > 1 {
        return Err(Error::new(ErrorKind::InvalidInput, "Data can only be written to one index at a time"));
    }

    let write_data = &query.fn_params;
    let columns = &db_settings.tables.get(&query.table_name).unwrap().columns;

    if write_data.len() != columns.len() {
        return Err(Error::new(ErrorKind::InvalidInput, "The number of arguments does not match the number of data columns in the table"));
    }

    // Check if given data matches column data types
    for row_data in write_data.iter().enumerate() {
        let col_data = &columns[row_data.0];

        match col_data.value {
            meta::ColValue::NumberI => {
                if row_data.1.parse::<i64>().is_err() {
                    return Err(Error::new(ErrorKind::InvalidInput, "Data type does not match column type which is NumberI"));
                }
            },
            meta::ColValue::NumberF => {
                if row_data.1.parse::<f64>().is_err() {
                    return Err(Error::new(ErrorKind::InvalidInput, "Data type does not match column type which is NumberF"));
                }
            },
            _ => (),
        }
    }

    // Since data can only be written to one index at a time
    // we only need to look at the first index.
    match &query.indexes[0] {
        query::IndexType::Index(i) => {
            let (line, file_name, index) = util::get_line_fname_idx(db_settings, query, *i);

            let Ok(mut file_data) = util::file_read(&file_name, file_system) else {
                return Err(Error::new(ErrorKind::Other, "File read error"));
            };

            if file_data.len() > line as usize {
                file_data[line as usize] = util::sanitize_db_entry(query.fn_params.join(","));
            } else {
                return Err(Error::new(ErrorKind::Other, "Attempting to write outside index bounds"));
            }

            if util::file_write(&file_name, file_data, file_system) {
                return Ok(format!("Write to index: {index} succeeded"));
            } else {
                return Err(Error::new(ErrorKind::Other, "Write failed"));
            }
        },
        query::IndexType::Wildcard => {
            let table_max_index = if db_settings.tables.get(&query.table_name).unwrap().biggest_id > 0 {
                db_settings.tables.get(&query.table_name).unwrap().biggest_id + 1
            } else {
                0
            };
            let (_line, file_name, index) = util::get_line_fname_idx(db_settings, query, table_max_index);

            let Ok(mut file_data) = util::file_read(&file_name, file_system) else {
                return Err(Error::new(ErrorKind::Other, "File read error"));
            };

            file_data.push(util::sanitize_db_entry(query.fn_params.join(",")));

            if util::file_write(&file_name, file_data, file_system) {
                db_settings.iterate_id(&query.table_name);
                return Ok(format!("Write success, new index: {index}"));
            } else {
                return Err(Error::new(ErrorKind::InvalidInput, "Write failed"));
            }
        },
    }
} 

/// Remove function logic
///
/// Removes data from database
/// Overwrites data with empty string effectively deleting it
pub fn remove_from_db(db_settings: &mut meta::DBSettings, file_system: &mut file::FileSystem, query: &query::QueryResult) -> Result<String> {
    if query.indexes.len() > 1 {
        return Err(Error::new(ErrorKind::InvalidInput, "Data can only be removed from one index at a time"));
    }

    let query::IndexType::Index(i) = query.indexes[0] else {
        return Err(Error::new(ErrorKind::InvalidInput, "Index type is incorrect"));
    };

    let (line, file_name, index) = util::get_line_fname_idx(db_settings, query, i);

    let Ok(mut file_data) = util::file_read(&file_name, file_system) else {
        return Err(Error::new(ErrorKind::Other, "File read error"));
    };

    file_data.insert(line as usize, String::new()); // Overwrite current value with empty String
    
    if util::file_write(&file_name, file_data, file_system) {
        return Ok(format!("Row at index {index} has been removed"));
    } else {
        return Err(Error::new(ErrorKind::Other, "Write error"));
    }
}

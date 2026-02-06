use std::collections::HashMap;
use rand::prelude::*;
use std::io::{Result, Error, ErrorKind};
use crate::{file, meta, query, util};

#[derive(Debug)]
pub enum DBDatatype {
    NumberI(i64),
    NumberF(f64),
    VarChar(String),
}

#[derive(Debug)]
enum DBFunction {
    Main(fn(&mut meta::DBSettings, &mut file::FileSystem, &query::QueryResult) -> Result<Vec<HashMap<String, DBDatatype>>>),
    MainReturnStatus(fn(&mut meta::DBSettings, &mut file::FileSystem, &query::QueryResult) -> Result<String>),
    Sub(fn(&mut Vec<HashMap<String, DBDatatype>>) -> Vec<HashMap<String, DBDatatype>>),
}

pub struct DBEngine {
    file_system: file::FileSystem,
    db_settings: meta::DBSettings,
    functions: HashMap<String, DBFunction>,
    sub_functions: HashMap<String, DBFunction>,
}

impl DBEngine {
    pub fn new(schema_file_path: &str) -> DBEngine {
        DBEngine {
            file_system: file::FileSystem::new(),
            db_settings: meta::load_meta(schema_file_path),
            functions: load_functions(),
            sub_functions: load_sub_functions(),
        }
    }
    pub fn query(&mut self, query: &query::QueryResult) -> Result<Vec<HashMap<String, DBDatatype>>> {
        if !self.db_settings.table_exists(&query.table_name) {
            return Err(Error::new(ErrorKind::NotFound, "Table doesn't exist'"));
        }

        let Some(main_function) = self.functions.get(&query.fn_name) else {
            return Err(Error::new(ErrorKind::NotFound, "Function not found"));
        };

        match *main_function {
            DBFunction::Main(func) => {
                match func(&mut self.db_settings, &mut  self.file_system, &query) {
                    Ok(result) => return Ok(result),
                    Err(e) => {
                        return Err(e);
                    },
                }
            }
            DBFunction::MainReturnStatus(func) => {
                match func(&mut self.db_settings, &mut  self.file_system, &query) {
                    Ok(status) => {
                        return Err(Error::new(ErrorKind::Other, status));
                    },
                    Err(e) => {
                        return Err(e);
                    },
                }
            },
            _ => return Err(Error::new(ErrorKind::Other, "not reachable")),
        }
    }
}

fn load_functions() -> HashMap<String, DBFunction> {
    let mut functions = HashMap::new();

    functions.insert(String::from(""), {
        /// Reads data from database
        /// Default functionality
        fn read_from_db(db_settings: &mut meta::DBSettings, file_system: &mut file::FileSystem, query: &query::QueryResult) -> Result<Vec<HashMap<String, DBDatatype>>> {
            let mut result: Vec<HashMap<String, DBDatatype>> = Vec::new();
            let col_settings = &db_settings.tables.get(&query.table_name).unwrap().columns;

            for index in &query.indexes {
                match index {
                    query::IndexType::Index(i) => {
                        let (line, file_name, index) = util::get_line_fname_idx(db_settings, query, *i);

                        match util::read_line(file_name.as_str(), file_system, line) {
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
                        let Ok(dir) = file_system.read_folder(dir_name.as_str()) else {
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
                            match file_system.open(file_name.as_str()) {
                                Ok(_) => (),
                                Err(e) if e.kind() == ErrorKind::InvalidInput => (),
                                Err(e) => {
                                    println!("File open failed: {e}");
                                    continue;
                                },
                            }

                            match file_system.read_from_cache(file_name.as_str()) {
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
        DBFunction::Main(read_from_db)
    });

    functions.insert(String::from("write"), {
        /// write function logic
        /// Writes data to database
        fn write_to_db(db_settings: &mut meta::DBSettings, file_system: &mut file::FileSystem, query: &query::QueryResult) -> Result<String> {
            if query.indexes.len() > 1 {
                return Err(Error::new(ErrorKind::InvalidInput, "Data can only be written to one index at a time"));
            }

            let row_data_split = util::escape_split(query.fn_param.as_str(), ',');
            let columns = &db_settings.tables.get(&query.table_name).unwrap().columns;

            if row_data_split.len() != columns.len() {
                return Err(Error::new(ErrorKind::InvalidInput, "The number of arguments does not match the number of data columns in the table"));
            }

            // Check if given data matches column data types
            for row_data in row_data_split.iter().enumerate() {
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

                    let Ok(mut file_data) = util::file_read(file_name.as_str(), file_system) else {
                        return Err(Error::new(ErrorKind::Other, "File read error"));
                    };

                    file_data.insert(line as usize, query.fn_param.clone());

                    if util::file_write(file_name.as_str(), file_data, file_system) {
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
                    let (_line, file_name, _index) = util::get_line_fname_idx(db_settings, query, table_max_index);

                    let Ok(mut file_data) = util::file_read(file_name.as_str(), file_system) else {
                        return Err(Error::new(ErrorKind::Other, "File read error"));
                    };

                    file_data.push(query.fn_param.clone());

                    if util::file_write(file_name.as_str(), file_data, file_system) {
                        db_settings.iterate_id(&query.table_name);
                        return Ok(String::from("Write success"));
                    } else {
                        return Err(Error::new(ErrorKind::InvalidInput, "Write failed"));
                    }
                },
            }
        } 
        DBFunction::MainReturnStatus(write_to_db)
    });

    functions.insert(String::from("remove"), {
        /// Remove function logic
        ///
        /// Removes data from database
        /// Overwrites data with empty string effectively deleting it
        fn remove_from_db(db_settings: &mut meta::DBSettings, file_system: &mut file::FileSystem, query: &query::QueryResult) -> Result<String> {
            if query.indexes.len() > 1 {
                return Err(Error::new(ErrorKind::InvalidInput, "Data can only be removed from one index at a time"));
            }

            let query::IndexType::Index(i) = query.indexes[0] else {
                return Err(Error::new(ErrorKind::InvalidInput, "Index type is incorrect"));
            };

            let (line, file_name, index) = util::get_line_fname_idx(db_settings, query, i);

            let Ok(mut file_data) = util::file_read(file_name.as_str(), file_system) else {
                return Err(Error::new(ErrorKind::Other, "File read error"));
            };

            file_data.insert(line as usize, String::new()); // Overwrite current value with empty String
            
            if util::file_write(file_name.as_str(), file_data, file_system) {
                return Ok(format!("Row at index {index} has been removed"));
            } else {
                return Err(Error::new(ErrorKind::Other, "Write error"));
            }
        }
        DBFunction::MainReturnStatus(remove_from_db)
    });

    functions.insert(String::from("random"), {
        /// random function logic
        ///
        /// Returns random values from database
        fn random_from_db(db_settings: &mut meta::DBSettings, file_system: &mut file::FileSystem, query: &query::QueryResult) -> Result<Vec<HashMap<String, DBDatatype>>> {
            let mut rng = rand::rng();
            let Ok(nr_of_random_values) = query.fn_param.parse::<u64>() else {
                return Err(Error::new(ErrorKind::InvalidInput, "Incorrect parameter type for random function"));
            };
            let biggest_id = db_settings.tables.get(&query.table_name).unwrap().biggest_id;
            let col_settings = &db_settings.tables.get(&query.table_name).unwrap().columns;

            match &query.indexes[0] {
                query::IndexType::Index(_) => {
                    if query.indexes.len() < nr_of_random_values as usize {
                        return Err(Error::new(ErrorKind::InvalidInput, "Attempting to retrieve more values than the given query includes"));
                    }

                    let mut indexes: Vec<u64> = Vec::new();
                    for index in &query.indexes {
                        if let query::IndexType::Index(i) = index {
                            indexes.push(*i);
                        }
                    }

                    indexes.shuffle(&mut rng);

                    let mut result: Vec<HashMap<String, DBDatatype>> = Vec::new();

                    for index in indexes {
                        let (line, file_name, index) = util::get_line_fname_idx(db_settings, query, index);

                        match util::read_line(file_name.as_str(), file_system, line) {
                            Ok(content) => {
                                if content.is_empty() {
                                    return Err(Error::new(ErrorKind::NotFound, "Line is empty"));
                                }
                                result.push(util::parse_db_line(content, index, &col_settings));
                            }
                            Err(e) if e.kind() == ErrorKind::Other => (),
                            Err(_) => {
                                return Err(Error::new(ErrorKind::Other, "Index out of table range"));
                            }
                        };

                        if result.len() >= nr_of_random_values as usize{
                            file_system.drop_entire_cache();
                            return Ok(result);
                        }
                    }
                    return Err(Error::new(ErrorKind::InvalidInput, "Attempting to retrieve more values than the table includes"));
                },
                query::IndexType::Wildcard => {
                    if biggest_id + 1 < nr_of_random_values {
                        return Err(Error::new(ErrorKind::InvalidInput, "Attempting to retrieve more values than the table includes"));
                    }

                    let mut indexes: Vec<u64> = (0..=biggest_id).collect();
                    indexes.shuffle(&mut rng);

                    let mut result: Vec<HashMap<String, DBDatatype>> = Vec::new();

                    for i in indexes {
                        let (line, file_name, index) = util::get_line_fname_idx(db_settings, query, i);

                        match util::read_line(file_name.as_str(), file_system, line) {
                            Ok(content) => {
                                if !content.is_empty() {
                                    result.push(util::parse_db_line(content, index, &col_settings));
                                }
                            }
                            Err(e) if e.kind() == ErrorKind::Other => (),
                            Err(_) => {
                                return Err(Error::new(ErrorKind::Other, "Index out of table range"));
                            }
                        };

                        if result.len() >= nr_of_random_values as usize{
                            file_system.drop_entire_cache();
                            return Ok(result);
                        }
                    }
                    return Err(Error::new(ErrorKind::InvalidInput, "Attempting to retrieve more values than the table includes"));
                }
            }
        }
        DBFunction::Main(random_from_db)
    });

    functions.insert(String::from("where"), {
        /// Where function logic
        ///
        /// Returns data which matches condition
        fn where_from_db(db_settings: &mut meta::DBSettings, file_system: &mut file::FileSystem, query: &query::QueryResult) -> Result<Vec<HashMap<String, DBDatatype>>> {
            let mut result: Vec<HashMap<String, DBDatatype>> = Vec::new();
            let mut arguments = util::escape_split(query.fn_param.as_str(), ',').into_iter();

            // Assign variables necessary for conditon matching
            let column = {
                let Some(column) = arguments.next() else {
                    return Err(Error::new(ErrorKind::InvalidInput, "Please give column name"));
                };
                if column.is_empty() {
                    return Err(Error::new(ErrorKind::InvalidInput, "Please give column name"));
                }
                if !db_settings.tables.get(&query.table_name).unwrap().has_column(column.to_string()) {
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
                if !match operator {
                    ">" => true,
                    "<" => true,
                    "=" => true,
                    _ => false,
                } {
                    return Err(Error::new(ErrorKind::InvalidInput, format!("{operator} is not a valid operator")));
                }
                operator
            };
            let Some(value) = arguments.next() else {
                return Err(Error::new(ErrorKind::InvalidInput, "Please give comparison value"));
            };
            let col_settings = &db_settings.tables.get(&query.table_name).unwrap().columns;

            /// Checks if condition is matching
            /// Returns Ok(true) if condition matches
            /// Returns Ok(false) if condition is not matching
            /// Returns Err(_) on incorrect operator value pair
            fn is_matching(column_value: &meta::ColValue, column_content: &str, match_value: &str, operator: &str) -> Result<bool> {
                match operator {
                    ">" => {
                        match column_value {
                            meta::ColValue::NumberI => {
                                if let Ok(value) = match_value.parse::<i64>() {
                                    if column_content.parse::<i64>().unwrap() > value {
                                        return Ok(true);
                                    }
                                }
                                return Ok(false);
                            },
                            meta::ColValue::NumberF => {
                                if let Ok(value) = match_value.parse::<f64>() {
                                    if column_content.parse::<f64>().unwrap() > value {
                                        return Ok(true);
                                    }
                                }
                                return Ok(false);
                            },
                            meta::ColValue::VarChar => return Err(Error::new(ErrorKind::Other, "cannot use > operator on VarChar")),
                        }
                    },
                    "<" => {
                        match column_value {
                            meta::ColValue::NumberI => {
                                if let Ok(value) = match_value.parse::<i64>() {
                                    if column_content.parse::<i64>().unwrap() < value {
                                        return Ok(true);
                                    }
                                }
                                return Ok(false);
                            },
                            meta::ColValue::NumberF => {
                                if let Ok(value) = match_value.parse::<f64>() {
                                    if column_content.parse::<f64>().unwrap() < value {
                                        return Ok(true);
                                    }
                                }
                                return Ok(false);
                            },
                            meta::ColValue::VarChar => return Err(Error::new(ErrorKind::Other, "cannot use < operator on VarChar")),
                        }
                    },
                    "=" => {
                        if column_content == match_value {
                            return Ok(true);
                        }
                        return Ok(false);
                    },
                    _ => return Ok(false),
                }
            }

            for index in &query.indexes {
                match index {
                    query::IndexType::Index(i) => {
                        let (line, file_name, index) = util::get_line_fname_idx(db_settings, query, *i);
                        let (column_index, column) = db_settings.tables.get(&query.table_name).unwrap().get_column(column.to_string()).unwrap();

                        match util::read_line(file_name.as_str(), file_system,  line) {
                            Ok(content) => {
                                    if content.is_empty() {
                                        continue;
                                    }

                                    let column_content = util::escape_split(content.as_str(), ',')[column_index];

                                    match is_matching(&column.value, column_content, value, operator) {
                                        Ok(b) => {
                                            if b {
                                                result.push(util::parse_db_line(content, index, col_settings));
                                            }
                                        },
                                        Err(e) => {
                                            return Err(e);
                                        },
                                    }
                            },
                            Err(e) if e.kind() == ErrorKind::Other => (),
                            Err(_) => {
                                return Err(Error::new(ErrorKind::InvalidInput, "Index out of range"));
                            }
                        };
                    },
                    query::IndexType::Wildcard => {
                        let dir_name = format!("./tables/{}", query.table_name);
                        let Ok(dir) = file_system.read_folder(dir_name.as_str()) else {
                            panic!("Critical failiure! table '{}' does not have a folder", query.table_name);
                        };
                        let (column_index, column) = db_settings.tables.get(&query.table_name).unwrap().get_column(column.to_string()).unwrap();

                        for file in dir {
                            match file {
                                Ok(dir_entry) => {
                                    let container = dir_entry.file_name().into_string().unwrap().parse::<u64>().unwrap();
                                    let file_name = format!("{}/{}", dir_name, container);
                                    match file_system.open(file_name.as_str()) {
                                        Ok(_) => (),
                                        Err(e) if e.kind() == ErrorKind::InvalidInput => (),
                                        Err(e) => {
                                            println!("File open failed: {e}");
                                            continue;
                                        },
                                    }

                                    match file_system.read_from_cache(file_name.as_str()) {
                                        Ok(contents) => {
                                            for (index, content) in contents.into_iter().enumerate() {
                                                if content.is_empty() {
                                                    continue;
                                                }

                                                let index = util::get_index(index as u64, container, db_settings);

                                                let column_content = util::escape_split(content.as_str(), ',')[column_index];

                                                match is_matching(&column.value, column_content, value, operator) {
                                                    Ok(b) => {
                                                        if b {
                                                            result.push(util::parse_db_line(column_content.to_string(), index, col_settings));
                                                        }
                                                    },
                                                    Err(e) => {
                                                        return Err(e);
                                                    },
                                                }
                                            }
                                        },
                                        Err(e) => {
                                            println!("Read failed: {e}");
                                            continue;
                                        },
                                    }
                                },
                                Err(_) => (),
                            }
                        }
                    },
                }
            }

            file_system.drop_entire_cache();
            Ok(result)
        }
        DBFunction::Main(where_from_db)
    });

    functions
}

fn load_sub_functions() -> HashMap<String, DBFunction> {
    HashMap::new()
}

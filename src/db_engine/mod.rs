use std::collections::HashMap;
use std::io::{Result, Error, ErrorKind};
use std::option::Option;
use crate::{file, meta, query};

mod db_functions;
mod db_sub_functions;

#[derive(Debug, Clone)]
pub enum DBDatatype {
    NumberI(i64),
    NumberF(f64),
    VarChar(String),
}

impl DBDatatype {
    pub fn as_f64(&self) -> Option<f64>{
        if let DBDatatype::NumberF(v) = self {
            return Some(*v);
        }
        return None;
    }
    pub fn as_i64(&self) -> Option<i64> {
        if let DBDatatype::NumberI(v) = self {
            return Some(*v);
        }
        return None;
    }
    pub fn as_string(&self) -> Option<String> {
        if let DBDatatype::VarChar(v) = self {
            return Some(v.clone());
        }
        return None;
    }
    pub fn from_str(from: &str) -> DBDatatype {
        if let Ok(v) = from.parse::<i64>() {
            return DBDatatype::NumberI(v);
        }
        if let Ok(v) = from.parse::<f64>() {
            return DBDatatype::NumberF(v);
        }
        DBDatatype::VarChar(from.to_string())
    }
    pub fn contains(&self, substr: &str) -> bool {
        if let DBDatatype::VarChar(self_v) = self {
            return self_v.contains(substr);
        } else {
            return false;
        }
    }
}

impl PartialEq for DBDatatype {
    fn eq(&self, other: &Self) -> bool {
        match self {
            DBDatatype::NumberI(self_v) => {
                if let DBDatatype::NumberI(other_v) = other {
                    self_v == other_v
                } else {
                    false
                }
            },
            DBDatatype::NumberF(self_v) => {
                if let DBDatatype::NumberF(other_v) = other {
                    self_v == other_v
                } else {
                    false
                }
            },
            DBDatatype::VarChar(self_v) => {
                if let DBDatatype::VarChar(other_v) = other {
                    self_v == other_v
                } else {
                    false
                }
            },
        }
    }
}

impl PartialOrd for DBDatatype {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self {
            DBDatatype::NumberI(self_v) => {
                if let DBDatatype::NumberI(other_v) = other {
                    Some(self_v.cmp(other_v))
                } else {
                    None
                }
            },
            DBDatatype::NumberF(self_v) => {
                 if let DBDatatype::NumberF(other_v) = other {
                    self_v.partial_cmp(other_v)
                } else {
                    None
                }               
            },
            DBDatatype::VarChar(self_v) => {
                if let DBDatatype::VarChar(other_v) = other {
                    Some(self_v.cmp(other_v))
                } else {
                    None
                }
            }
        }
    }
}

pub enum DBResult {
    Data(Vec<HashMap<String, DBDatatype>>),
    Status((String, Vec<i64>)),
    Error(Error),
}

#[derive(Debug)]
enum DBFunction {
    Main(fn(&mut meta::DBSettings, &mut file::FileSystem, &query::QueryResult) -> Result<Vec<HashMap<String, DBDatatype>>>),
    MainReturnStatus(fn(&mut meta::DBSettings, &mut file::FileSystem, &query::QueryResult) -> Result<(String, Vec<i64>)>),
    Sub(fn(&mut Vec<HashMap<String, DBDatatype>>, &query::QueryResult, &Vec<String>, &meta::DBSettings) -> Result<()>),
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
    pub fn query(&mut self, query: &str) -> DBResult {
        let query = match query::parse_query(query) {
            Ok(query) => query,
            Err(e) => {
                println!("Query parse error: {e}");
                return DBResult::Error(Error::new(ErrorKind::InvalidInput, "Query parse error"));
            }
        };

        println!("{query}");

        if !self.db_settings.table_exists(&query.table_name) {
            return DBResult::Error(Error::new(ErrorKind::NotFound, "Table doesn't exist'"));
        }

        let Some(main_function) = self.functions.get(&query.fn_name) else {
            return DBResult::Error(Error::new(ErrorKind::NotFound, "Function not found"));
        };

        let mut result = match *main_function {
            DBFunction::Main(func) => {
                match func(&mut self.db_settings, &mut  self.file_system, &query) {
                    Ok(result) => result,
                    Err(e) => {
                        return DBResult::Error(e);
                    },
                }
            }
            DBFunction::MainReturnStatus(func) => {
                match func(&mut self.db_settings, &mut  self.file_system, &query) {
                    Ok((status, affected_indexes)) => {
                        return DBResult::Status((status, affected_indexes));
                    },
                    Err(e) => {
                        return DBResult::Error(e);
                    },
                }
            },
            _ => return DBResult::Error(Error::new(ErrorKind::Other, "not reachable")),
        };

        if query.sub_fn_names.is_empty() {
            return DBResult::Data(result);
        }

        for (index, sub_fn_name) in query.sub_fn_names.iter().enumerate() {
            if let Some(DBFunction::Sub(sub_fn)) = self.sub_functions.get(sub_fn_name) {
                match sub_fn(&mut result, &query, &query.sub_fn_params[index], &self.db_settings) {
                    Ok(_) => (),
                    Err(e) => return DBResult::Error(e),
                }
            } else {
                return DBResult::Error(Error::new(ErrorKind::NotFound, "sub function not found"));
            }
        }

        DBResult::Data(result)
    }
}

fn load_functions() -> HashMap<String, DBFunction> {
    let mut functions = HashMap::new();

    functions.insert(String::from(""), {
        DBFunction::Main(db_functions::read_from_db)
    });

    functions.insert(String::from("write"), {
        DBFunction::MainReturnStatus(db_functions::write_to_db)
    });

    functions.insert(String::from("remove"), {
        DBFunction::MainReturnStatus(db_functions::remove_from_db)
    });

    functions
}

fn load_sub_functions() -> HashMap<String, DBFunction> {
    let mut sub_functions = HashMap::new();

    sub_functions.insert(String::from("sort"), {
        DBFunction::Sub(db_sub_functions::sort_by)
    });

    sub_functions.insert(String::from("random"), {
        DBFunction::Sub(db_sub_functions::random_from_db)
    });

    sub_functions.insert(String::from("where"), {
        DBFunction::Sub(db_sub_functions::where_from_db)
    });

    sub_functions
}

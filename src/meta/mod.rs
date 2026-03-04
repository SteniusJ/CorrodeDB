use yaml_rust2::YamlLoader;
use std::collections::HashMap;
use std::io::{Result, ErrorKind, Error};

#[derive(Clone, Debug)]
pub enum ColValue {
    NumberI,
    NumberF,
    VarChar
}
#[derive(Debug)]
pub struct DBSettings {
    pub tables: HashMap<String, TableSettings>,
    pub compartment_rows: i16,
}

#[derive(Debug)]
pub struct TableSettings {
    pub columns: Vec<ColSettings>,
    pub biggest_id: u64,
}

#[derive(Clone, Debug)]
pub struct ColSettings {
    pub name: String,
    pub value: ColValue,
}

impl DBSettings {
    /// Create new DBSettings struct from yaml string
    pub fn new(settings_yaml: &str) -> DBSettings {
        let docs = YamlLoader::load_from_str(settings_yaml).unwrap();
        let doc = &docs[0];

        let mut table_map: HashMap<String, TableSettings> = HashMap::new();

        for tables in doc["tables"].as_hash().expect("Expected object 'tables'").iter() {

            let table_name = tables.0.clone().into_string().expect("Expected table name");
            let mut columns: Vec<ColSettings> = Vec::new();

            for column in tables.1["columns"].as_hash().expect(&format!("expected table {table_name} to have columns, please create a object called 'columns'")).iter() {
                let col_settings = ColSettings {
                    name: column.0.clone().into_string().expect("Expected column name"),
                    value: match column.1["value"].as_str().expect("Expected variable 'value'") {
                        "NumberI" => ColValue::NumberI,
                        "NumberF" => ColValue::NumberF,
                        "VarChar" => ColValue::VarChar,
                        v => panic!("Expected column value to be 'NumberI', 'NumberF' or 'VarChar', not '{v}'"),
                    },
                };

                columns.push(col_settings);
            }

            let table_settings = TableSettings {
                columns: columns,
                biggest_id: 0,
            };

            table_map.insert(table_name, table_settings);
        }

        DBSettings {
            tables: table_map,
            compartment_rows: doc["settings"]["compartment"]["rows"].as_i64().expect("Expected compartmet rows value") as i16,
        }
    }
    pub fn table_exists(&self, table_name: &String) -> bool {
        self.tables.contains_key(table_name)
    }
    pub fn iterate_id(&mut self, table_name: &String) {
        self.tables.get_mut(table_name).unwrap().iterate_id();
    }
    pub fn reset_id(&mut self, table_name: &String) {
        self.tables.get_mut(table_name).unwrap().biggest_id = 0;
    }
}

impl TableSettings {
    pub fn get_column(&self, column_name: String) -> Result<(usize, &ColSettings)> {
        for (index, column) in self.columns.iter().enumerate() {
            if column.name == column_name {
                return Ok((index, column));
            }
        }
        Err(Error::new(ErrorKind::Other, "column not found"))
    }
    pub fn has_column(&self, column_name: &str) -> bool {
        for column in &self.columns {
            if column.name == column_name {
                return true;
            }
        }

        false
    }
    fn iterate_id(&mut self) {
        self.biggest_id += 1;
    }
}

/// Loads DBSettings struct from file path to valid yaml file
pub fn load_meta(meta_file_path: &str) -> DBSettings {
    println!("---------------- loading schema -------------------\nloading schema yaml file from: {meta_file_path}\n");

    let mut file_system = crate::file::FileSystem::new();

    match file_system.open(meta_file_path) {
        Ok(status) => println!("Schema file open: {status:?}"),
        Err(e) => panic!("Schema file open failed: {e:?}"),
    }

    let test_config_yaml = match file_system.read_from_cache(meta_file_path) {
        Ok(contents) => {
            println!("Schema file Read: Success");
            contents.join("\n")
        },
        Err(e) => panic!("Schema file read failed: {e:?}"),
    };

    match file_system.drop_from_cache(meta_file_path) {
        Ok(status) => println!("Schema file remove from cache: {status:?}"),
        Err(e) => println!("Schema file drop failed: {e:?}"),
    }

    let mut db_settings = DBSettings::new(test_config_yaml.as_str());

    println!("------------ settings loaded --------------\ninitializing db directories\n");

    for table in &mut db_settings.tables {
        let table_name = table.0;
        println!("configuring directory for: {table_name}");

        match file_system.create_folder(format!("./tables/{table_name}").as_str()) {
            Ok(_) => println!("created folder for table: {table_name}"),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                let last_file_name = {
                    let mut highest_file: u32 = 0;
                    for dir_entry in file_system.read_folder(format!("./tables/{table_name}").as_str()).unwrap() {
                        let dir_entry = dir_entry.unwrap();
                        let fname = dir_entry.file_name().into_string().unwrap().parse::<u32>().unwrap();
                        if highest_file < fname {
                            highest_file = fname;
                        }
                    }

                    format!("{highest_file}")
                };

                file_system.open(format!("./tables/{table_name}/{last_file_name}").as_str()).unwrap();
                let file_contents = file_system.read_from_cache(format!("./tables/{table_name}/{last_file_name}").as_str()).unwrap();

                let latest_index = {
                    let last_file_index: u64 = last_file_name.parse().unwrap();
                    if file_contents.len() > 0 { // avoid underflow
                        (db_settings.compartment_rows as u64 * last_file_index) + file_contents.len() as u64 - 1
                    } else {
                        0
                    }
                };

                table.1.biggest_id = latest_index;
                println!("biggest id for table '{table_name}' is '{latest_index}'\n");
            },
            Err(e) => {
                panic!("folder creation for table '{table_name}' failed due to error '{e}'");
            }
        }
    }

    println!("--------- db setup finished ---------\n");
    db_settings
}

pub fn get_password(meta_file_path: &str) -> String {
    let mut file_system = crate::file::FileSystem::new();

    match file_system.open(meta_file_path) {
        Ok(_) => (),
        Err(e) => panic!("Schema file open failed: {e:?}"),
    }

    let test_config_yaml = match file_system.read_from_cache(meta_file_path) {
        Ok(contents) => {
            contents.join("\n")
        },
        Err(e) => panic!("Schema file read failed: {e:?}"),
    };

    match file_system.drop_from_cache(meta_file_path) {
        Ok(_) => (),
        Err(e) => println!("Schema file drop failed: {e:?}"),
    }

    let docs = YamlLoader::load_from_str(test_config_yaml.as_str()).unwrap();
    let doc = &docs[0];

    doc["settings"]["password"].as_str().unwrap().to_string()
}

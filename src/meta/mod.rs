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
    pub password: String,
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

        for tables in doc["tables"].as_hash().unwrap().iter().enumerate() {
            let table_name = tables.1.0.as_str().unwrap();
            let mut col_vec: Vec<ColSettings> = Vec::new();

            // Tables have the possibility to hold other data than just what rows they include, but
            // this is currently not used so we skip one and iterate the rows.
            // 
            // Not sure this is most optimal.
            for columns in tables.1.1.as_hash()
            .unwrap()
            .iter()
            .next()
            .unwrap()
            .1
            .as_hash()
            .unwrap()
            .iter()
            .enumerate() {
                let mut col_settings = ColSettings {
                    name: columns.1.0.as_str().unwrap().to_string(),
                    value: ColValue::VarChar,
                };

                for col_data in columns.1.1.as_hash().unwrap().iter().enumerate() {
                    match col_data.1.0.as_str().unwrap() {
                        "value" => {
                            col_settings.value = match col_data.1.1.as_str().unwrap() {
                                "NumberI" => ColValue::NumberI,
                                "NumberF" => ColValue::NumberF,
                                "VarChar" => ColValue::VarChar,
                                _ => ColValue::VarChar,
                            }
                        }
                        _ => (),
                    }
                }

                col_vec.push(col_settings);
            }

            let table_settings = TableSettings {
                columns: col_vec.clone(),
                biggest_id: 0,
            };

            table_map.insert(table_name.to_string(), table_settings);
        }

        DBSettings {
            tables: table_map,
            password: doc["settings"]["password"].as_str().unwrap().to_string(),
            compartment_rows: doc["settings"]["compartment"]["rows"].as_i64().unwrap() as i16,
        }
    }
    pub fn table_exists(&self, table_name: &String) -> bool {
        self.tables.contains_key(table_name)
    }
    pub fn iterate_id(&mut self, table_name: &String) {
        self.tables.get_mut(table_name).unwrap().iterate_id();
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
    pub fn has_column(&self, column_name: String) -> bool {
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
                let last_file_name = match file_system.read_folder(format!("./tables/{table_name}").as_str()).last() {
                    Some(r) => {
                        r.unwrap().file_name().into_string().unwrap()
                    },
                    None => {
                        table.1.biggest_id = 0;
                        println!("biggest id for table '{table_name}' is '0'\n");
                        continue;
                    },
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

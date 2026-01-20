use yaml_rust2::YamlLoader;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub enum RowValue {
    Number,
    VarChar
}

pub struct DBSettings {
    pub tables: HashMap<String, TableSettings>,
    pub cache_max_size: i16,
    pub cache_life_time: i16,
    pub compartment_rows: i16,
}

#[derive(Debug)]
pub struct TableSettings {
    pub rows: HashMap<String, RowSettings>,
    pub biggest_id: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct RowSettings {
    pub value: RowValue,
    pub primary_key: bool,
    pub auto_iterate: bool,
}

impl DBSettings {
    pub fn new(settings_yaml: &str) -> DBSettings {
        let docs = YamlLoader::load_from_str(settings_yaml).unwrap();
        let doc = &docs[0];

        let mut table_map: HashMap<String, TableSettings> = HashMap::new();
        let mut row_map: HashMap<String, RowSettings> = HashMap::new();

        for tables in doc["tables"].as_hash().unwrap().iter().enumerate() {
            let table_name = tables.1.0.as_str().unwrap();

            // Tables have the possibility to hold other data than just what rows they include, but
            // this is currently not used so we skip one and iterate the rows.
            // 
            // Not sure this is most optimal.
            for rows in tables.1.1.as_hash()
            .unwrap()
            .iter()
            .next()
            .unwrap()
            .1
            .as_hash()
            .unwrap()
            .iter()
            .enumerate() {
                let row_name = rows.1.0.as_str().unwrap();

                let mut row_settings = RowSettings {
                    value: RowValue::VarChar,
                    primary_key: false,
                    auto_iterate: false,
                };

                for row_data in rows.1.1.as_hash().unwrap().iter().enumerate() {
                    match row_data.1.0.as_str().unwrap() {
                        "value" => {
                            row_settings.value = match row_data.1.1.as_str().unwrap() {
                                "Number" => RowValue::Number,
                                "VarChar" => RowValue::VarChar,
                                _ => RowValue::VarChar,
                            }
                        }
                        "primary_key" => row_settings.primary_key = row_data.1.1.as_bool().unwrap(),
                        "auto_iterate" => row_settings.auto_iterate = row_data.1.1.as_bool().unwrap(),
                        _ => (),
                    }
                }

                row_map.insert(row_name.to_string(), row_settings);
            }

            let table_settings = TableSettings {
                rows: row_map.clone(),
                biggest_id: 0,
            };

            table_map.insert(table_name.to_string(), table_settings);
        }

        DBSettings {
            tables: table_map,
            cache_max_size: doc["settings"]["cache"]["max_size"].as_i64().unwrap() as i16,
            cache_life_time: doc["settings"]["cache"]["life_time"].as_i64().unwrap() as i16,
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
    fn iterate_id(&mut self) {
        self.biggest_id += 1;
    }
}

pub fn load_meta(meta_file_path: &str) -> DBSettings {
    println!("loading settings yaml file from: {meta_file_path}\n");

    let mut file_system = crate::file::FileSystem::new();

    match file_system.open(meta_file_path) {
        Ok(status) => println!("Meta file open: {status:?}"),
        Err(e) => panic!("Meta file open failed: {e:?}"),
    }

    let test_config_yaml = match file_system.read_from_cache(meta_file_path) {
        Ok(contents) => {
            println!("Meta file read success");
            contents.join("\n")
        },
        Err(e) => panic!("Meta file read failed: {e:?}"),
    };

    match file_system.drop_from_cache(meta_file_path) {
        Ok(status) => println!("Meta file remove from cache: {status:?}"),
        Err(e) => println!("Meta file drop failed: {e:?}"),
    }

    let mut db_settings = DBSettings::new(test_config_yaml.as_str());

    println!("------------ settings loaded --------------\ninitializing db directories\n");

    for table in &mut db_settings.tables {
        let table_name = table.0;
        println!("configuring directory for: {table_name}");

        match file_system.create_folder(format!("./tables/{table_name}").as_str()) {
            Ok(_) => println!("created folder for table: {table_name}"),
            Err(e) => {
                println!("folder creation for table: {table_name} failed due to error: {e}");
                if e.kind() == std::io::ErrorKind::AlreadyExists {
                    let last_file_name = file_system.read_folder(format!("./tables/{table_name}").as_str()).last().unwrap().unwrap().file_name().into_string().unwrap();

                    file_system.open("./tables/{table_name/{last_file_name}}").unwrap();
                    let file_contents = file_system.read_from_cache(format!("./tables/{table_name}/{last_file_name}").as_str()).unwrap();


                    let latest_index = {
                        let last_file_index: u64 = last_file_name.parse().unwrap();
                        (db_settings.compartment_rows as u64 * last_file_index) + file_contents.len() as u64 - 1
                    };

                    table.1.biggest_id = latest_index;
                    println!("biggest id for table: {table_name} is {latest_index}");
                }
            }
        }
        println!();
    }

    println!("--------- db setup finished ---------\n");
    db_settings
}

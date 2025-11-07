use yaml_rust2::{YamlLoader};
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
}

pub fn load_meta(mut file_system: crate::file::FileSystem, meta_file_path: &str) -> DBSettings {
    match file_system.open(meta_file_path) {
        Ok(status) => println!("Meta file open status: {status:?}"),
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
        Ok(status) => println!("Meta file removed from cache: {status:?}"),
        Err(e) => println!("Meta file drop failed: {e:?}"),
    }

    println!("-----------------------------");

    DBSettings::new(test_config_yaml.as_str())
}

mod file;
mod meta;

const TEST_FILE_PATH: &str = "./tables/test";

fn main() {
    let mut file_system = file::FileSystem::new();

    let test_config_yaml = 
"
tables:
  messages:
    rows:
      id:
        value: Number
        primary_key: true
        auto_iterate: true
      message:
        value: VarChar
      time_posted:
        value: VarChar
  reactions:
    rows:
      id:
        value: Number
        primary_key: true
        auto_iterate: true
      reaction:
        value: VarChar
  images:
    rows:
      id:
        value: Number
        primary_key: true
        auto_iterate: true
      image_link:
        value: VarChar

settings:
  cache:
    max_size: 2042 # Bytes
    life_time: 300 # Seconds
";

    let db_settings = meta::DBSettings::new(test_config_yaml);

    let max_size = db_settings.cache_max_size;
    let life_time = db_settings.cache_life_time;

    println!("max size: {max_size:?}\nlife time: {life_time:?}\n------------------------------\n");

    let tables = &db_settings.tables;

    println!("Tables: {tables:?}\n-----------------------------\n");

    let rows = &tables.get("messages").unwrap().rows;

    println!("Rows: {rows:?}\n------------------------\n");

    match file_system.create_folder("./tables") {
        Ok(status) => println!("Dir creation status: {status:?}"),
        Err(e) => println!("Dir create failed: {e:?}"),
    }

     match file_system.open(TEST_FILE_PATH) {
         Ok(status) => println!("File open status: {status:?}"),
         Err(e) => println!("file open failed: {e:?}"),
     }

     match file_system.read_from_cache(TEST_FILE_PATH) {
         Ok(contents) => println!("Read return: {contents:?}"),
         Err(e) => println!("Read failed: {e:?}"),
     }

     match file_system.write_to_cache(TEST_FILE_PATH, "New content in cache\nrow2\nrow3".to_string()) {
         Ok(s) => println!("Write status: {s:?}"),
         Err(e) => println!("Write fail: {e:?}"),
     }

     match file_system.read_from_cache(TEST_FILE_PATH) {
         Ok(contents) => println!("Read return: {contents:?}"),
         Err(e) => println!("Read failed: {e:?}"),
     }

     match file_system.open(TEST_FILE_PATH) {
         Ok(status) => println!("File open status: {status:?}"),
         Err(e) => println!("file open failed: {e:?}"),
     }

     match file_system.write_cache_to_disk(TEST_FILE_PATH) {
         Ok(s) => println!("File disk write status: {s:?}"),
         Err(e) => println!("File open failed: {e:?}"),
     }
}

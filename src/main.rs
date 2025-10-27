mod file;

const TEST_FILE_PATH: &str = "./tables/test";

fn main() {
    let mut file_system = file::FileSystem::new();

    match file_system.create_folder("./tables") {
        Ok(status) => println!("Dir creation status: {status:?}"),
        Err(e) => println!("Dir create failed: {e:?}"),
    }

     match file_system.open(TEST_FILE_PATH) {
         Ok(status) => println!("File open status: {status:?}"),
         Err(e) => println!("file open failed: {e:?}"),
     }

     match file_system.read_from_cache(TEST_FILE_PATH) {
         Ok(contents) => println!("Read return: {contents}"),
         Err(e) => println!("Read failed: {e:?}"),
     }

     match file_system.write_to_cache(TEST_FILE_PATH, "New content in cache".to_string()) {
         Ok(s) => println!("Write status: {s:?}"),
         Err(e) => println!("Write fail: {e:?}"),
     }

     match file_system.read_from_cache(TEST_FILE_PATH) {
         Ok(contents) => println!("Read return: {contents}"),
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

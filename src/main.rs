mod file;

fn main() {
    let mut file_system = file::FileSystem::new();

     match file_system.open("test") {
         Ok(status) => println!("file opened successfully: {status:?}"),
         Err(e) => println!("file open failed: {e:?}"),
     }

     match file_system.read_from_cache("test") {
         Ok(contents) => println!("{contents}"),
         Err(e) => println!("Read failed: {e:?}"),
     }

     match file_system.write_to_cache("test", "New content in cache".to_string()) {
         Ok(s) => println!("Status: {s:?}"),
         Err(e) => println!("{e:?}"),
     }

     match file_system.read_from_cache("test") {
         Ok(contents) => println!("{contents}"),
         Err(e) => println!("Read failed: {e:?}"),
     }

     match file_system.open("test") {
         Ok(status) => println!("file opened for a second time: {status:?}"),
         Err(e) => println!("file open failed: {e:?}"),
     }
}

mod file;

fn main() {
    let file_system = file::FileSystem::new();

     match file_system.open() {
         Ok(()) => println!("file opened successfully"),
         Err(e) => println!("file open failed: {e:?}"),
     }
}

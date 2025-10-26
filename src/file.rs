use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;
use std::io::Result;

#[derive(Debug)]
pub enum Status {
    Success,
    FileInCache,
    WriteSuccess
}

pub struct FileSystem {
    cache: HashMap<String, String>,
}

impl FileSystem {

    /// Creates new FileSystem
    pub fn new() -> FileSystem {
        let new_cache: HashMap<String, String> = HashMap::new();
        let file_system = FileSystem { cache: new_cache };

        file_system
    }

    /// Opens a file into cache
    /// If file is in cache file is not opened again
    /// If file doesn't exist it is created
    pub fn open(&mut self, file_name: &str) -> Result<Status> {

        if self.cache.contains_key(file_name) {
            return Ok(Status::FileInCache)
        }

        let mut file: File;

        match File::open("../tables/".to_string() + file_name) {
            Ok(f) => file = f,
            Err(_) => {
                match File::create("../tables/".to_string() + file_name) {
                    Ok(_) => println!("New file created"),
                    Err(e) => println!("{e:?}"),
                }

                file = File::open("../tables/".to_string() + file_name)?;
            },
        }

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        self.add_to_cache(file_name.to_string(), contents);

        Ok(Status::Success)
    }

    pub fn write_to_cache(&mut self, file_name: &str, new_content: String) -> Result<Status> {

        if !self.cache.contains_key(file_name) {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "File by name {file_name:?} is not in cache"))
        }

        self.add_to_cache(file_name.to_string(), new_content);

        Ok(Status::WriteSuccess)
    }

    pub fn read_from_cache(&mut self, file_name: &str) -> Result<String> {

        if !self.cache.contains_key(file_name) {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "File by name {file_name:?} is not in cache"))
        }

        let contents = String::from(self.cache.get(file_name).unwrap());

        Ok(contents)
    }

    fn add_to_cache(&mut self, file_name: String, file_contents: String) {
        self.cache.insert(file_name, file_contents);
    }
}

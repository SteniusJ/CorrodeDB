use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;
use std::io::Result;

#[derive(Debug)]
pub enum Status {
    Success
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
        if self.is_in_cache(file_name) {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "File already in cache"));
        }

        let mut file: File;

        match File::open(file_name) {
            Ok(f) => file = f,
            Err(_) => {
                match File::create(file_name) {
                    Ok(_) => println!("New file created"),
                    Err(e) => println!("{e:?}"),
                }

                file = File::open(file_name)?;
            },
        }

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        self.add_to_cache(file_name.to_string(), contents);

        Ok(Status::Success)
    }

    pub fn write_to_cache(&mut self, file_name: &str, new_content: String) -> Result<Status> {
        if !self.is_in_cache(file_name) {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not in cache"))
        }

        self.add_to_cache(file_name.to_string(), new_content);

        Ok(Status::Success)
    }

    pub fn read_from_cache(&mut self, file_name: &str) -> Result<String> {
        if !self.is_in_cache(file_name) {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not in cache"))
        }

        let contents = String::from(self.cache.get(file_name).unwrap());

        Ok(contents)
    }

    pub fn write_cache_to_disk(&mut self, file_name: &str) -> Result<Status> {
        if !self.is_in_cache(file_name) {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not in cache"))
        }

        let mut file: File;

        match File::create(file_name) {
            Ok(f) => file = f,
            Err(e) => return Err(e),
        }

        let contents = String::from(self.cache.get(file_name).unwrap());

        println!("File contents: {contents}");

        match file.write_all(contents.as_bytes()) {
            Ok(_) => {
                self.cache.remove(file_name);

                return Ok(Status::Success)
            },
            Err(e) => return Err(e),
        }
    }

    fn add_to_cache(&mut self, file_name: String, file_contents: String) {
        self.cache.insert(file_name, file_contents);
    }

    fn is_in_cache(&mut self, file_name: &str) -> bool {
        if self.cache.contains_key(file_name) {
            return true
        }
        false
    }
}

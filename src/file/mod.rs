use std::fs::File;
use std::fs::DirBuilder;
use std::path::Path;
use std::io::prelude::*;
use std::collections::HashMap;
use std::io::Result;

#[derive(Debug)]
pub enum Status {
    Success
}

pub struct FileSystem {
    cache: HashMap<String, Vec<String>>,
}

impl FileSystem {
    /// Creates new FileSystem
    pub fn new() -> FileSystem {
        let new_cache: HashMap<String, Vec<String>> = HashMap::new();
        
        FileSystem {
            cache: new_cache
        }
    }

    /// Opens a file into cache
    ///
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

        self.add_to_cache(file_name.to_string(), contents.lines().map(String::from).collect());

        Ok(Status::Success)
    }
    
    /// Writes data to cache if file is stored in cache
    /// Overwrites Current data in cache with the new
    pub fn write_to_cache(&mut self, file_name: &str, new_content: String) -> Result<Status> {
        if !self.is_in_cache(file_name) {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not in cache"))
        }

        self.add_to_cache(file_name.to_string(), new_content.lines().map(String::from).collect());

        Ok(Status::Success)
    }

    /// Returns file data from cache as a String
    /// Returns error if file does not exist in cache
    pub fn read_from_cache(&mut self, file_name: &str) -> Result<Vec<String>> {
        if !self.is_in_cache(file_name) {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not in cache"))
        }

        let contents = self.cache.get(file_name).unwrap().clone();

        Ok(contents)
    }

    pub fn read_line_from_cache(&mut self, file_name: &str, line: usize) -> Result<String> {
        let line = line - 1;

        match self.read_from_cache(file_name) {
            Ok(file_contents) => {
                if file_contents.len() >= line {
                    let line_string = file_contents[line].clone();
                    return Ok(line_string);
                }
                return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Line number outside file size"))
            },
            Err(e) => {
                return Err(e);
            }
        }
    }

    /// Writes data for specified file from the cache to long term memory
    pub fn write_cache_to_disk(&mut self, file_name: &str) -> Result<Status> {
        if !self.is_in_cache(file_name) {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not in cache"))
        }

        let mut file: File;

        match File::create(file_name) {
            Ok(f) => file = f,
            Err(e) => return Err(e),
        }

        let contents = self.cache.get(file_name).unwrap().clone().join("\n");

        match file.write_all(contents.as_bytes()) {
            Ok(_) => {
                self.cache.remove(file_name);
                self.cache.shrink_to_fit();

                return Ok(Status::Success)
            },
            Err(e) => return Err(e),
        }
    }

    pub fn write_entire_cache_to_disk(&mut self) -> Result<Status> {
        let keys: Vec<String> = self.cache.keys().cloned().collect();

        for file_path in keys {
            let mut file = match File::create(&file_path) {
                Ok(f) => f,
                Err(_) => continue,
            };

            if let Some(contents) = self.cache.get(&file_path) {
                let contents = contents.clone().join("\n");
                if file.write_all(contents.as_bytes()).is_err() {
                    continue;
                }
            }

            self.cache.remove(&file_path);
        }

        self.cache.shrink_to_fit();
        Ok(Status::Success)
    }

    /// Creates new directory by dir_name
    ///
    /// dir_name can be the name of the last folder in a chain, and the folders will get made
    /// recursively
    pub fn create_folder(&self, dir_name: &str) -> Result<Status> {
        let path = Path::new(dir_name);

        if !path.exists() && path.is_dir() {
            return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "Dir already exists"));
        }

        let mut builder = DirBuilder::new();
        builder.recursive(true);

        match builder.create(dir_name) {
            Ok(_) => return Ok(Status::Success),
            Err(e) => return Err(e),
        }
    }

    /// Removes entry from cache and frees memory
    ///
    /// Should be used when file contents are no longer needed but no write to disk is necessary
    pub fn drop_from_cache(&mut self, file_name: &str) -> Result<Status> {
        if !self.is_in_cache(file_name) {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not in cache"))
        }

        self.cache.remove(file_name);
        self.cache.shrink_to_fit();

        Ok(Status::Success)
    }

    /// Returns true/false if data is already in cache
    pub fn is_in_cache(&mut self, file_name: &str) -> bool {
        if self.cache.contains_key(file_name) {
            return true
        }
        false
    }

    pub fn get_cached_files(&self) -> std::collections::hash_map::Keys<'_, String, Vec<String>> {
        self.cache.keys()
    }

    /// Adds data to cache
    fn add_to_cache(&mut self, file_name: String, file_contents: Vec<String>) {
        self.cache.insert(file_name, file_contents);
    }
}

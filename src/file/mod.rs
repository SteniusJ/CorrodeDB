use std::fs::{
    File,
    DirBuilder,
    ReadDir,
};
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

    /// Returns data from given line
    /// Returns error if line is outside file size
    pub fn read_line_from_cache(&mut self, file_name: &str, line: usize) -> Result<String> {
        let line = line;

        match self.read_from_cache(file_name) {
            Ok(file_contents) => {
                if file_contents.len() > line {
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

    /// Writes all entires is cache to storage
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

        if path.exists() && path.is_dir() {
            return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "Dir already exists"));
        }

        let mut builder = DirBuilder::new();
        builder.recursive(true);

        match builder.create(dir_name) {
            Ok(_) => return Ok(Status::Success),
            Err(e) => return Err(e),
        }
    }

    pub fn read_folder(&self, dir_name: &str) -> ReadDir {
        let path = Path::new(dir_name);
        path.read_dir().unwrap()
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

    pub fn drop_entire_cache(&mut self) {
        self.cache.clear();
        self.cache.shrink_to_fit();
    }

    /// Returns true/false if data is already in cache
    pub fn is_in_cache(&mut self, file_name: &str) -> bool {
        if self.cache.contains_key(file_name) {
            return true
        }
        false
    }

    /// Returns file names of all files in cache
    pub fn get_cached_files(&self) -> std::collections::hash_map::Keys<'_, String, Vec<String>> {
        self.cache.keys()
    }

    /// Adds data to cache
    fn add_to_cache(&mut self, file_name: String, file_contents: Vec<String>) {
        self.cache.insert(file_name, file_contents);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;

    const TEST_READ_FILE_PATH: &str = "./test_data/test_read_file";
    const TEST_WRITE_FILE_PATH: &str = "./test_data/test_write_file";

    #[test]
    fn test_file_system_open() {
        let mut file_system = FileSystem::new();

        file_system.open(TEST_READ_FILE_PATH).unwrap();
        file_system.open(TEST_READ_FILE_PATH).expect_err("File open succeeded even when opening a already open file!");
        
        assert_eq!(&vec![String::from("hello world"), String::from("hello world1")], file_system.cache.get(TEST_READ_FILE_PATH).unwrap());
    }

    #[test]
    fn test_file_system_read() {
        let mut file_system = FileSystem::new();

        file_system.open(TEST_READ_FILE_PATH).expect("File open failed, not relevant to this test");
        
        let result = file_system.read_from_cache(TEST_READ_FILE_PATH).expect("File read failed");
        let result_single_line = file_system.read_line_from_cache(TEST_READ_FILE_PATH, 0).unwrap();
        file_system.read_line_from_cache(TEST_READ_FILE_PATH, 3).expect_err("Filesystem read a line that doesn't exist");

        assert_eq!(result, vec![String::from("hello world"), String::from("hello world1")]);
        assert_eq!(result_single_line, String::from("hello world"));
    }

    #[test]
    fn test_file_system_write() {
        let mut file_system = FileSystem::new();
        let mut rng = rand::rng();
        let rand_uuid = format!("0x{:X}",rng.random::<u128>());

        file_system.open(TEST_WRITE_FILE_PATH).expect("File open failed, not relevant to this test");
        file_system.write_to_cache(TEST_WRITE_FILE_PATH, rand_uuid.clone()).unwrap();
        let cache_result = file_system.read_from_cache(TEST_WRITE_FILE_PATH).unwrap();
        assert_eq!(cache_result.last().unwrap(), &rand_uuid);

        file_system.write_cache_to_disk(TEST_WRITE_FILE_PATH).unwrap();

        file_system.open(TEST_WRITE_FILE_PATH).expect("File open failed, not relevant to this test");
        let result = file_system.read_from_cache(TEST_WRITE_FILE_PATH).expect("File read failed, not relevant to this test");
        assert_eq!(result.last().unwrap(), &rand_uuid);
    }
}

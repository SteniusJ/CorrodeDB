use crate::file;
use crate::meta;
use std::io::Result;

pub struct DbManager {
    file_system: file::FileSystem,
    db_settings: meta::DBSettings,
}

impl DbManager {
    pub fn new(meta_file_path: &str) -> DbManager {
        DbManager {
            file_system: file::FileSystem::new(),
            db_settings: meta::load_meta(meta_file_path),
        }
    }
    pub fn read(&mut self, file_path: &str) -> Result<Vec<String>> {
        if !self.open(file_path) {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Placeholder error, something went wrong during file open"));
        }
        match self.file_system.read_from_cache(file_path) {
            Ok(contents) => Ok(contents),
            Err(e) => Err(e),
        }
    }
    pub fn write(&mut self, file_path: &str, content: String) {
        if !self.open(file_path) {
            todo!();
        }
        match self.file_system.write_to_cache(file_path, content) {
            Ok(_) => (),
            Err(_) => (),
        }
    }
    pub fn clear_cache(&mut self) {
        for file_name in self.file_system.get_cached_files() {
            self.file_system.write_cache_to_disk(file_name);
        }
    }
    fn open(&mut self, file_path: &str) -> bool {
        if self.file_system.is_in_cache(file_path) {
            return true;
        }

        match self.file_system.open(file_path) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

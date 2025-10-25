use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;

pub struct FileSystem {
    cache: HashMap<String, File>,
}

impl FileSystem {
    pub fn new() -> FileSystem {
        let mut new_cache: HashMap<String, File> = HashMap::new();
        let mut file_system = FileSystem { cache: new_cache };

        file_system
    }
    pub fn open(&self) -> std::io::Result<()> {
        let mut file = File::create("../tables/foo.txt")?;
        file.write_all(b"opened")?;

        Ok(())
    }
}

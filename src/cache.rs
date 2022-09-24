use std::{
    error::Error,
    fs::{self, File},
    io::Write,
    path::Path,
};

/// Cache for already downloaded/skipped songs, following the format set by
/// Ezwen/bandcamp-collection-downloaderr.
pub struct Cache {
    path: String,
}

// TODO: possibly move to a proper DB like sqlite/leveldb?

impl Cache {
    pub fn new(path: String) -> Cache {
        Cache { path }
    }

    pub fn content(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let path = Path::new(&self.path);
        if !path.exists() {
            return Ok(vec![]);
        }

        let lines = fs::read_to_string(path)?
            .lines()
            .map(|x| x.split("|").next().unwrap().to_string())
            .collect::<Vec<String>>();

        Ok(lines)
    }

    pub fn add(&self, id: &str, description: &str) -> Result<(), Box<dyn Error>> {
        let mut file = File::options()
            .create(true)
            .append(true)
            .open(self.path.clone())?;
        // Format compatible with bandcamp-collection-downloader
        let content = format!("{id}| {description}\n");

        file.write_all(content.as_bytes())?;

        Ok(())
    }
}

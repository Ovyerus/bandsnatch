use std::{
    error::Error,
    fs::{self, File},
    io::Write,
    path::Path,
};

/// Cache for already downloaded/skipped songs, following the format set by
/// Ezwen/bandcamp-collection-downloaderr.
pub struct Cache<P: AsRef<Path>> {
    path: P,
}

// TODO: move to something backed by sqlite or leveldb or similar, and add method to auto transform old format.

impl<P: AsRef<Path>> Cache<P> {
    pub fn new(path: P) -> Self {
        Self { path }
    }

    pub fn content(&self) -> Result<Vec<String>, Box<dyn Error>> {
        if let Ok(content) = fs::read_to_string(&self.path) {
            Ok(content
                .lines()
                .map(|x| x.split('|').next().unwrap().to_string())
                .collect::<Vec<String>>())
        } else {
            Ok(vec![])
        }
    }

    pub fn add(&self, id: &str, description: &str) -> Result<(), Box<dyn Error>> {
        let path = self.path.as_ref();
        let mut file = File::options().create(true).append(true).open(path)?;
        // Format compatible with bandcamp-collection-downloader
        let content = format!("{id}| {description}\n");

        file.write_all(content.as_bytes())?;

        Ok(())
    }
}

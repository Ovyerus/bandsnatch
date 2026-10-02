use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io::Write,
    path::Path,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CacheEntry {
    Complete,
    Preorder,
}

impl CacheEntry {
    pub fn needs_download(self, is_preorder: bool) -> bool {
        self == Self::Preorder && !is_preorder
    }
}

/// Cache for already downloaded/skipped releases, following the legacy
/// `id| description` format with an optional preorder marker in the description.
pub struct Cache<P: AsRef<Path>> {
    path: P,
    entries: HashMap<String, CacheEntry>,
}

// TODO: move to something backed by sqlite or leveldb or similar, and add method to auto transform old format.

impl<P: AsRef<Path>> Cache<P> {
    pub fn new(path: P) -> Self {
        let mut entries = HashMap::new();
        if let Ok(content) = fs::read_to_string(&path) {
            for line in content.lines() {
                let (id, description) = line.split_once('|').unwrap_or((line, ""));
                let entry = if description
                    .trim_start()
                    .starts_with("@bandsnatch:preorder ")
                {
                    CacheEntry::Preorder
                } else {
                    CacheEntry::Complete
                };
                entries.insert(id.to_string(), entry);
            }
        }
        Self { path, entries }
    }

    pub fn content(&self) -> &HashMap<String, CacheEntry> {
        &self.entries
    }

    pub fn add(
        &mut self,
        id: &str,
        description: &str,
        entry: CacheEntry,
    ) -> Result<(), Box<dyn Error>> {
        if self
            .entries
            .get(id)
            .is_some_and(|cached| !cached.needs_download(entry == CacheEntry::Preorder))
        {
            return Ok(());
        }
        let path = self.path.as_ref();
        let mut file = File::options().create(true).append(true).open(path)?;
        // The first field stays compatible with existing ID-only cache readers.
        let content = match entry {
            CacheEntry::Complete => format!("{id}| {description}\n"),
            CacheEntry::Preorder => format!("{id}| @bandsnatch:preorder {description}\n"),
        };
        file.write_all(content.as_bytes())?;
        if let Some(cached) = self.entries.get_mut(id) {
            *cached = entry;
        } else {
            self.entries.insert(id.to_string(), entry);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Cache, CacheEntry};
    use std::fs;

    #[test]
    fn preorder_becomes_downloadable_again_when_released() {
        let path = std::env::temp_dir().join(format!(
            "bandsnatch-preorder-{}-{}.cache",
            std::process::id(),
            rand::random::<u64>()
        ));
        fs::write(&path, "legacy| Already downloaded\n").unwrap();
        let mut cache = Cache::new(&path);
        assert_eq!(cache.content().get("legacy"), Some(&CacheEntry::Complete));
        cache
            .add("release", "Artist - Album", CacheEntry::Preorder)
            .unwrap();
        let cached = cache.content();
        assert!(!cached["release"].needs_download(true));
        assert!(cached["release"].needs_download(false));
        assert!(!cached["legacy"].needs_download(false));

        cache
            .add("release", "Artist - Album", CacheEntry::Complete)
            .unwrap();
        let cached = cache.content();
        assert_eq!(cached["release"], CacheEntry::Complete);
        assert!(!cached["release"].needs_download(false));
        cache
            .add("release", "Artist - Album", CacheEntry::Preorder)
            .unwrap();
        assert_eq!(cache.content()["release"], CacheEntry::Complete);
        assert_eq!(fs::read_to_string(&path).unwrap().lines().count(), 3);
        fs::remove_file(path).unwrap();
    }
}

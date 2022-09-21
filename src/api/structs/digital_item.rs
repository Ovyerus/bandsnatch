use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Deserialize, Debug)]
pub struct DigitalItem {
    pub downloads: HashMap<String, DigitalItemDownload>,
    pub package_release_dat: Option<String>,
    pub title: String,
    pub artist: String,
    pub download_type: String,
    pub download_type_str: String,
    pub item_type: String,
    // number to string
    pub art_id: String,
}

#[derive(Clone, Deserialize, Debug)]
pub struct DigitalItemDownload {
    pub size_mb: String,
    pub description: String,
    pub encoding_name: String, // Download is chosen by comparing this field and the `format` option.
    pub url: String,
}

impl DigitalItem {
    pub fn cover_url(&self) -> String {
        let art_id = &self.art_id;
        format!("https://f4.bcbits.com/img/a{art_id}")
    }
}

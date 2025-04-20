use crate::util::make_string_fs_safe;

use chrono::{Datelike, NaiveDateTime};
use serde::{self, Deserialize};
use std::{collections::HashMap, path::Path};

const FORMAT: &str = "%d %b %Y %T %Z";

// #[derive(Clone, Deserialize, Debug)]
// #[serde(untagged)]
// pub enum ArtId {
//     Str(String),
//     Num(i64),
// }

#[derive(Clone, Deserialize, Debug)]
pub struct DigitalItem {
    pub downloads: Option<HashMap<String, DigitalItemDownload>>,
    pub package_release_date: Option<String>,
    pub title: String,
    pub artist: String,
    pub download_type: Option<String>,
    pub download_type_str: String,
    pub item_type: String,
    // pub art_id: Option<ArtId>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct DigitalItemDownload {
    // pub size_mb: Option<String>,
    // pub description: String,
    // pub encoding_name: String, // Download is chosen by comparing this field and the `format` option.
    pub url: String,
}

impl DigitalItem {
    // pub fn cover_url(&self) -> String {
    //     let art_id = &self.art_id;
    //     format!("https://f4.bcbits.com/img/a{art_id}")
    // }

    pub fn is_single(&self) -> bool {
        (self.download_type.is_some() && self.download_type.as_ref().unwrap() == "t")
            || self.download_type_str == "track"
            || self.item_type == "track"
    }

    pub fn release_year(&self) -> String {
        match &self.package_release_date {
            Some(d) => match NaiveDateTime::parse_from_str(d, FORMAT) {
                Ok(dt) => dt.and_utc().year().to_string(),
                Err(err) => {
                    debug!("Failed to parse date time: {}", err);
                    String::from("0000")
                },
            },
            None => String::from("0000"),
        }
    }

    pub fn destination_path<P: AsRef<Path>>(&self, root: P) -> String {
        root.as_ref()
            .join(make_string_fs_safe(&self.artist))
            .join(format!(
                "{} ({})",
                make_string_fs_safe(&self.title),
                self.release_year()
            ))
            .to_str()
            .unwrap()
            .to_owned()
    }
}

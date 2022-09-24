use chrono::{DateTime, Datelike, TimeZone, Utc};
use serde::{self, Deserialize, Deserializer};
use serde_aux::prelude::deserialize_string_from_number;
use std::{collections::HashMap, path::Path};

const FORMAT: &str = "%d %b %Y %T %Z";

fn parse_bc_date_str<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?;
    if let Some(s) = s {
        return Ok(Some(
            Utc.datetime_from_str(&s, FORMAT)
                .map_err(serde::de::Error::custom)?,
        ));
    }

    Ok(None)
}

#[derive(Clone, Deserialize, Debug)]
pub struct DigitalItem {
    pub downloads: HashMap<String, DigitalItemDownload>,
    #[serde(deserialize_with = "parse_bc_date_str")]
    pub package_release_date: Option<DateTime<Utc>>,
    pub title: String,
    pub artist: String,
    pub download_type: String,
    pub download_type_str: String,
    pub item_type: String,
    #[serde(deserialize_with = "deserialize_string_from_number")]
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

    pub fn is_single(&self) -> bool {
        self.download_type == "t" || self.download_type_str == "track" || self.item_type == "track"
    }

    pub fn release_year(&self) -> String {
        match self.package_release_date {
            Some(d) => d.year().to_string(),
            None => String::from("0000"),
        }
    }

    pub fn destination_path(&self, root: &str) -> String {
        // TODO: append year
        Path::new(root)
            .join(&self.artist)
            .join(&format!("{} ({})", self.title, self.release_year()))
            .to_str()
            .unwrap()
            .to_owned()
    }
}

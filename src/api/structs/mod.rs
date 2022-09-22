use serde::Deserialize;
use serde_aux::prelude::deserialize_string_from_number;
use std::collections::HashMap;

pub mod digital_item;
pub use crate::api::structs::digital_item::DigitalItem;

pub type DownloadsMap = HashMap<String, String>;

// TODO: test with no hidden items, some hidden items, and no non-hidden items.
/// Structure of the JSON blob extracted from a user's Bandcamp page.
#[derive(Deserialize, Debug)]
pub struct ParsedFanpageData {
    /// Data about the fan the page is for.
    pub fan_data: FanData,
    /// Data about the user's music collection.
    pub collection_data: CollectionData,
    /// Data about items in the user's music collection that have been hidden.
    pub hidden_data: CollectionData,
    pub item_cache: ItemCache,
}

#[derive(Deserialize, Debug)]
pub struct FanData {
    #[serde(deserialize_with = "deserialize_string_from_number")]
    pub fan_id: String,
    pub is_own_page: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct CollectionData {
    pub batch_size: u16,
    pub item_count: u16,
    pub last_token: Option<String>,
    pub redownload_urls: Option<DownloadsMap>,
}

#[derive(Deserialize, Debug)]
pub struct ItemCache {
    pub collection: HashMap<String, CachedItem>,
    pub hidden: HashMap<String, CachedItem>,
}

#[derive(Deserialize, Debug)]
pub struct CachedItem {
    #[serde(deserialize_with = "deserialize_string_from_number")]
    pub sale_item_id: String,
    pub band_name: String,
    pub item_title: String,
}

/// Structure of the data returned from Bandcamp's collection API.
#[derive(Deserialize, Debug)]
pub struct ParsedCollectionItems {
    pub more_available: bool,
    pub last_token: String,
    pub redownload_urls: DownloadsMap,
}

#[derive(Deserialize, Debug)]
pub struct ParsedItemsData {
    pub digital_items: Vec<DigitalItem>,
}

#[derive(Deserialize, Debug)]
pub struct ParsedStatDownload {
    pub download_url: String,
    pub url: String,
}

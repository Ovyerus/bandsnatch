use serde::Deserialize;
use serde_aux::prelude::{deserialize_default_from_null, deserialize_string_from_number};
use std::collections::HashMap;

pub mod digital_item;
pub use crate::api::structs::digital_item::DigitalItem;

pub type DownloadsMap = HashMap<String, String>;

/// Identity data embedded in Bandcamp's homepage application.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct HomepageData {
    pub page_context: HomepageContext,
}

#[derive(Deserialize)]
pub(super) struct HomepageContext {
    pub identity: HomepageIdentity,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct HomepageIdentity {
    pub is_logged_in: bool,
    pub fan_username: Option<String>,
}

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
pub struct ItemCache {
    pub collection: HashMap<String, Item>,
}

#[derive(Deserialize, Debug)]
pub struct Item {
    pub sale_item_id: u64,
    pub sale_item_type: String,
    pub band_name: String,
    pub item_title: String,
    /// Whether Bandcamp currently exposes this item as a preorder.
    ///
    /// Bandcamp sends `null` here for collection entries that aren't releases,
    /// such as label subscriptions, so coerce those back to the default.
    #[serde(default, deserialize_with = "deserialize_default_from_null")]
    pub is_preorder: bool,
    /// Whether Bandcamp offers a digital download for this item.
    ///
    /// `None` for entries that have nothing to download, such as label
    /// subscriptions and physical-only purchases.
    #[serde(default)]
    pub download_available: Option<bool>,
    /// Hints about where the item is sold. `None` for a few older releases.
    #[serde(default)]
    pub url_hints: Option<UrlHints>,
}

#[derive(Deserialize, Debug)]
pub struct UrlHints {
    pub subdomain: Option<String>,
}

/// Optional filters narrowing down which collection items get downloaded.
#[derive(Debug, Default)]
pub struct Filters<'a> {
    pub album: Option<&'a String>,
    pub artist: Option<&'a String>,
    pub subdomain: Option<&'a String>,
}

impl Filters<'_> {
    /// Whether an item is downloadable and passes every active filter.
    pub fn matches(&self, item: &Item) -> bool {
        if item.download_available != Some(true) {
            return false;
        }

        self.album
            .is_none_or(|v| item.item_title.eq_ignore_ascii_case(v))
            && self
                .artist
                .is_none_or(|v| item.band_name.eq_ignore_ascii_case(v))
            && self.subdomain.is_none_or(|v| {
                item.url_hints
                    .as_ref()
                    .and_then(|hints| hints.subdomain.as_deref())
                    .is_some_and(|subdomain| subdomain.eq_ignore_ascii_case(v))
            })
    }
}

#[derive(Deserialize, Debug)]
pub struct FanData {
    #[serde(deserialize_with = "deserialize_string_from_number")]
    pub fan_id: String,
    pub is_own_page: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct CollectionData {
    pub batch_size: Option<u16>,
    pub item_count: Option<u16>,
    pub last_token: Option<String>,
    pub redownload_urls: Option<DownloadsMap>,
}

/// Structure of the data returned from Bandcamp's collection API.
#[derive(Deserialize, Debug)]
pub struct ParsedCollectionItems {
    pub more_available: bool,
    pub last_token: String,
    pub redownload_urls: DownloadsMap,
    pub items: Vec<Item>,
}

#[derive(Deserialize, Debug)]
pub struct ParsedItemsData {
    pub digital_items: Vec<DigitalItem>,
}

// #[derive(Deserialize, Debug)]
// pub struct ParsedStatDownload {
//     pub download_url: String,
//     pub url: String,
// }

#[cfg(test)]
mod tests {
    use super::{Filters, Item};
    use serde_json::json;

    fn album(url_hints: serde_json::Value) -> Item {
        serde_json::from_value(json!({
            "sale_item_id": 2,
            "sale_item_type": "i",
            "band_name": "An Artist",
            "item_title": "A Record",
            "item_type": "album",
            "download_available": true,
            "is_preorder": false,
            "url_hints": url_hints,
        }))
        .unwrap()
    }

    #[test]
    fn the_subdomain_filter_only_matches_that_subdomain() {
        let on_label = album(json!({"subdomain": "alabel"}));
        let elsewhere = album(json!({"subdomain": "somewhere-else"}));
        let no_hints = album(json!(null));

        let filters = Filters {
            subdomain: Some(&"AlAbEl".to_string()),
            ..Default::default()
        };

        assert!(filters.matches(&on_label));
        assert!(!filters.matches(&elsewhere));
        // Nothing to compare against, so it cannot be on the label.
        assert!(!filters.matches(&no_hints));

        // Without the filter, every downloadable item still matches.
        assert!(Filters::default().matches(&no_hints));
    }

    #[test]
    fn items_with_nothing_to_download_never_match() {
        let subscription: Item = serde_json::from_value(json!({
            "sale_item_id": 1,
            "sale_item_type": "m",
            "band_name": "A Label",
            "item_title": "A Label",
            "download_available": null,
            "is_preorder": null,
            "url_hints": {"subdomain": "alabel"},
        }))
        .unwrap();

        assert!(!Filters::default().matches(&subscription));
        assert!(!Filters {
            subdomain: Some(&"alabel".to_string()),
            ..Default::default()
        }
        .matches(&subscription));
    }

    #[test]
    fn collection_entries_that_arent_releases_deserialise() {
        // A label subscription, as Bandcamp sends it: present, but null.
        let item: Item = serde_json::from_value(json!({
            "sale_item_id": 1,
            "sale_item_type": "m",
            "band_name": "A Label",
            "item_title": "A Label",
            "item_type": "subscription",
            "download_available": null,
            "is_preorder": null,
        }))
        .unwrap();

        assert!(!item.is_preorder);
        assert_eq!(item.download_available, None);
    }

    #[test]
    fn releases_keep_their_own_values() {
        let item: Item = serde_json::from_value(json!({
            "sale_item_id": 2,
            "sale_item_type": "i",
            "band_name": "An Artist",
            "item_title": "A Record",
            "item_type": "album",
            "download_available": true,
            "is_preorder": true,
        }))
        .unwrap();

        assert!(item.is_preorder);
        assert_eq!(item.download_available, Some(true));

        // Older entries can omit both fields entirely.
        let item: Item = serde_json::from_value(json!({
            "sale_item_id": 3,
            "sale_item_type": "i",
            "band_name": "An Artist",
            "item_title": "An Old Record",
        }))
        .unwrap();

        assert!(!item.is_preorder);
        assert_eq!(item.download_available, None);
    }
}

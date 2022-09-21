use serde::Serialize;
use soup::prelude::*;
use std::convert::TryInto;
use surf::{Client, Config, Url};
// use std::sync::Arc;
use std::error::Error;

mod structs;
use crate::api::structs::*;

pub struct BandcampPage {
    pub download_urls: DownloadsMap,
    pub page_name: String,
}

/// Body used to paginate through Bandcamp's collection API.
#[derive(Serialize, Debug)]
struct PostCollectionBody<'a> {
    fan_id: &'a str,
    older_than_token: &'a str,
}

pub struct Api {
    client: Client,
    cookies: String,
}

impl Api {
    pub fn new(cookies: String) -> Api {
        let client: Client = Config::new()
            .set_base_url(Url::parse("https://bandcamp.com").unwrap())
            .try_into()
            .unwrap();

        Api { client, cookies }
    }

    fn authenticated_get(&self, path: &str) -> surf::RequestBuilder {
        self.client.get(path).header("Cookie", &self.cookies)
    }

    fn authenticated_post(&self, path: &str) -> surf::RequestBuilder {
        self.client.post(path).header("Cookie", &self.cookies)
    }

    /// Scrape a user's Bandcamp page to find download urls
    pub async fn get_download_urls(&self, name: &str) -> Result<BandcampPage, Box<dyn Error>> {
        let res = self
            .authenticated_get(&format!("/{name}"))
            .recv_string()
            .await?;
        let soup = Soup::new(&res);

        let data_el = soup
            .attr("id", "pagedata")
            .find()
            .expect("Failed to extract data from collection page.");
        let data_blob = data_el
            .get("data-blob")
            .expect("Failed to extract data from element on collection page.");
        let fanpage_data: ParsedFanpageData = serde_json::from_str(&data_blob)
            .expect("Failed to deserialise collection page data blob.");

        match fanpage_data.fan_data.is_own_page {
            Some(true) => (),
            _ => bail!(format!(
                r#"Failed to scrape collection data for "{name}" (`is_own_page` is false). Perhaps check your cookies, or your spelling."#
            )),
        }

        // TODO: make sure this exists
        let mut collection = fanpage_data
            .collection_data
            .redownload_urls
            .clone()
            .unwrap();

        let skip_hidden_items = true;
        if skip_hidden_items {
            // TODO: filter `collection` to remove items that have their value containing a `sale_item_id` from `fanpage_data.item_cache.hidden`
            // collection.iter().filter(|&(k, v)| !fanpage_data.item_cache.hidden.contains_key(k))
        }

        if fanpage_data.collection_data.item_count > fanpage_data.collection_data.batch_size {
            let rest = self
                .get_rest_downloads_in_collection(&fanpage_data, "collection_items")
                .await?;
            collection.extend(rest);
        }

        if !skip_hidden_items
            && (fanpage_data.hidden_data.item_count > fanpage_data.hidden_data.batch_size)
        {
            let rest = self
                .get_rest_downloads_in_collection(&fanpage_data, "hidden_items")
                .await?;
            collection.extend(rest);
        }

        let title = soup.tag("title").find().unwrap().text();

        Ok(BandcampPage {
            page_name: title,
            download_urls: collection,
        })
    }

    /// Loop over a user's collection to retrieve all paginated items.
    async fn get_rest_downloads_in_collection(
        &self,
        data: &ParsedFanpageData,
        collection_name: &str,
    ) -> Result<DownloadsMap, Box<dyn Error>> {
        let collection_data = match collection_name {
            "collection_items" => &data.collection_data,
            "hidden_items" => &data.hidden_data,
            x => bail!(format!(r#"unexpected value for `collection_name`: "{x}""#)),
        };

        let mut last_token = collection_data.last_token.clone().unwrap();
        let mut more_available = true;
        let mut collection = DownloadsMap::new();

        while more_available {
            // retries
            let body = PostCollectionBody {
                fan_id: &data.fan_data.fan_id,
                older_than_token: &last_token,
            };
            let resp = self
                .authenticated_post(&format!("/api/fancollection/1/{collection_name}"))
                .body_json(&body)?
                .recv_json::<ParsedCollectionItems>()
                .await
                .expect("what");

            collection.extend(resp.redownload_urls);
            more_available = resp.more_available;
            last_token = resp.last_token;
        }

        Ok(collection)
    }

    // TODO: cache on API object?
    pub async fn get_digital_item(
        &self,
        download_urls: &DownloadsMap,
        sale_item_id: &str,
    ) -> Result<DigitalItem, Box<dyn Error>> {
        // Treat 404s as null
        let url = download_urls.get(sale_item_id).unwrap();
        let resp = self.authenticated_get(url).recv_string().await?;
        let soup = Soup::new(&resp);

        let download_page_blob = soup
            .attr("id", "pagedata")
            .find()
            .unwrap()
            .get("data-blob")
            .unwrap();
        let ParsedItemsData { digital_items } = serde_json::from_str(&download_page_blob).unwrap();
        let item = digital_items.first().cloned().unwrap();

        Ok(item)
    }
}

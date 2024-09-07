use ::reqwest::IntoUrl;
use governor::{Quota, RateLimiter};
use http::header::CONTENT_DISPOSITION;
use http::Method;
use indicatif::ProgressStyle;
use nonzero_ext::*;
use pollster::FutureExt as _;
use reqwest::blocking as reqwest;
use serde::Serialize;
use soup::prelude::*;
use std::error::Error;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;
use std::str;
use std::sync::Arc;

pub mod structs;
use crate::api::structs::*;
use crate::cookies;
use crate::util;

pub struct BandcampPage {
    pub download_urls: DownloadsMap,
    // pub page_name: String,
}

/// Body used to paginate through Bandcamp's collection API.
#[derive(Serialize, Debug)]
struct PostCollectionBody<'a> {
    fan_id: &'a str,
    older_than_token: &'a str,
}

const MAX_RETRIES: u8 = 5;

pub struct Api {
    pub client: reqwest::Client,
    ratelimiter: governor::DefaultDirectRateLimiter,
}

impl Api {
    pub fn new(cookies: Vec<cookies::RawCookie>) -> Self {
        let cookie_jar = cookies::fill_cookie_jar(cookies);
        let client = reqwest::ClientBuilder::new()
            .cookie_provider(Arc::new(cookie_jar))
            .build()
            .unwrap();
        let ratelimiter = RateLimiter::direct(Quota::per_second(nonzero!(3u32)));

        Self {
            client,
            ratelimiter,
        }
    }

    fn bc_path(path: &str) -> String {
        format!("https://bandcamp.com/{path}")
    }

    fn request<U: IntoUrl + Copy>(
        &self,
        method: Method,
        url: U,
    ) -> Result<reqwest::Response, Box<dyn Error>> {
        self.request_with_retry(method, url, 0)
    }

    fn request_with_retry<U: IntoUrl + Copy>(
        &self,
        method: Method,
        url: U,
        retry_attempt: u8,
    ) -> Result<reqwest::Response, Box<dyn Error>> {
        self.ratelimiter.until_ready().block_on();

        let response = self.client.request(method.clone(), url.clone()).send()?;
        let status: http::StatusCode = response.status();

        if !status.is_success() {
            if status != http::StatusCode::TOO_MANY_REQUESTS {
                bail!(
                    "request failed with status {status} for url {}",
                    url.as_str()
                );
            }

            if retry_attempt >= MAX_RETRIES {
                bail!(format!("reached maximum retries for url {}", url.as_str()));
            }

            warn!("hit ratelimit from Bandcamp, sleeping for 10 seconds");
            std::thread::sleep(std::time::Duration::from_secs(10));
            return self.request_with_retry(method, url, retry_attempt + 1);
        }

        Ok(response)
    }

    /// Scrape a user's Bandcamp page to find download urls
    pub fn get_download_urls(&self, name: &str) -> Result<BandcampPage, Box<dyn Error>> {
        debug!("`get_download_urls` for Bandcamp page '{name}'");

        let body = self.request(Method::GET, &Self::bc_path(name))?.text()?;
        let soup = Soup::new(&body);

        let data_el = soup
            .attr("id", "pagedata")
            .find()
            .expect("Failed to extract data from collection page.");
        let data_blob = data_el
            .get("data-blob")
            .expect("Failed to extract data from element on collection page.");
        let fanpage_data: ParsedFanpageData = serde_json::from_str(&data_blob)
            .expect("Failed to deserialise collection page data blob.");
        debug!("Successfully fetched Bandcamp page, and found + deserialised data blob");

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
            debug!("Skipping hidden collection items");
            // TODO: filter `collection` to remove items that have their value containing a `sale_item_id` from `fanpage_data.item_cache.hidden`
            // collection.iter().filter(|&(k, v)| !fanpage_data.item_cache.hidden.contains_key(k))
        }

        if fanpage_data.collection_data.item_count > fanpage_data.collection_data.batch_size {
            debug!(
                "Too many in `collection_data`, so we need to paginate ({} total)",
                // This should never be `None` thanks to the comparison above.
                fanpage_data.collection_data.item_count.unwrap()
            );
            let rest = self.get_rest_downloads_in_collection(&fanpage_data, "collection_items")?;
            collection.extend(rest);
        }

        if !skip_hidden_items
            && (fanpage_data.hidden_data.item_count > fanpage_data.hidden_data.batch_size)
        {
            debug!(
                "Too many in `hidden_data`, and we're told not to skip, so we need to paginate ({} total)",
                fanpage_data.hidden_data.item_count.unwrap()
            );
            let rest = self.get_rest_downloads_in_collection(&fanpage_data, "hidden_items")?;
            collection.extend(rest);
        }

        // let title = soup.tag("title").find().unwrap().text();

        debug!("Successfully retrieved all download URLs");
        Ok(BandcampPage {
            // page_name: title,
            download_urls: collection,
        })
    }

    /// Loop over a user's collection to retrieve all paginated items.
    fn get_rest_downloads_in_collection(
        &self,
        data: &ParsedFanpageData,
        collection_name: &str,
    ) -> Result<DownloadsMap, Box<dyn Error>> {
        debug!("Paginating results for {collection_name}");
        let collection_data = match collection_name {
            "collection_items" => &data.collection_data,
            "hidden_items" => &data.hidden_data,
            x => bail!(format!(r#"unexpected value for `collection_name`: "{x}""#)),
        };

        let mut last_token = collection_data.last_token.clone().unwrap();
        let mut more_available = true;
        let mut collection = DownloadsMap::new();

        while more_available {
            trace!("More items to collect, looping...");
            // retries
            let body = PostCollectionBody {
                fan_id: &data.fan_data.fan_id,
                older_than_token: &last_token,
            };
            let body = self
                .client
                .post(&Self::bc_path(&format!(
                    "api/fancollection/1/{collection_name}"
                )))
                .json(&body)
                .send()?
                .json::<ParsedCollectionItems>()?;

            trace!("Collected {} items", body.redownload_urls.clone().len());
            collection.extend(body.redownload_urls);
            more_available = body.more_available;
            last_token = body.last_token;
        }

        debug!("Finished paginating results for {collection_name}");
        Ok(collection)
    }

    pub fn get_digital_item(
        &self,
        url: &str,
        debug: &bool,
    ) -> Result<Option<DigitalItem>, Box<dyn Error>> {
        debug!("Retrieving digital item information for {url}");
        let text = self.request(Method::GET, url)?.text()?;
        let soup = Soup::new(&text);

        let download_page_blob = soup
            .attr("id", "pagedata")
            .find()
            .expect(&format!(
                "could not find `pagedata` element for digital item {url}"
            ))
            .get("data-blob")
            .expect(&format!(
                "could not extract `data-blob` from the pagedata element for digital item {url}"
            ));

        let item_result = std::panic::catch_unwind(|| {
            serde_json::from_str::<ParsedItemsData>(&download_page_blob).unwrap()
        });

        if item_result.is_err() {
            println!("Failed to get item info for {url}.");
            if *debug {
                println!("\n{download_page_blob}\n");
            } else {
                println!("Run with `--debug` to see the full JSON blob.\n")
            }

            bail!(format!("failed parsing {url}"))
        }

        let item = item_result.unwrap().digital_items.first().cloned();

        Ok(item)
    }

    pub fn download_item(
        &self,
        item: &DigitalItem,
        path: &str,
        audio_format: &str,
        m: &indicatif::MultiProgress,
    ) -> Result<(), Box<dyn Error>> {
        let download_url = &item
            .downloads
            .as_ref()
            .expect("cannot download a release with no downloads")
            .get(audio_format)
            .unwrap()
            .url;
        let res = self.request(Method::GET, download_url)?;

        let len = res.content_length().unwrap();
        let full_title = format!("{} - {}", item.title, item.artist);
        let pb = m.add(
            indicatif::ProgressBar::new(len)
                .with_message(full_title.clone())
                .with_style(
                    ProgressStyle::with_template("{bar:10} ({bytes}/{total_bytes}) {wide_msg}")
                        .unwrap(),
                ),
        );

        let disposition = res.headers().get(CONTENT_DISPOSITION);

        if let None = disposition {
            pb.finish_and_clear();
            return Err(
                format!("could not download {full_title} when using url `{download_url}`").into(),
            );
        }

        // `HeaderValue::to_str` only handles valid ASCII bytes, and Bandcamp
        // chooses to put Unicode into the content-disposition for some reason,
        // so need to handle ourselves.
        let content = str::from_utf8(disposition.unwrap().as_bytes())?;
        // Should probably use a thing to properly parse the content of content disposition.
        let filename = util::slice_string(
            content
                .split("; ")
                .find(|x| x.starts_with("filename="))
                .unwrap(),
            9,
        )
        .trim_matches('"');
        m.suspend(|| debug!("Downloading as `{filename}` to `{path}`"));

        // TODO: drop file with `.part` extension instead, while downloading, and then rename when finished?.

        let full_path = Path::new(path).join(filename);
        let mut file = File::create(&full_path)?;
        let mut stream = res;
        m.suspend(|| debug!("Starting download"));

        util::copy_with_progress(&mut stream, &mut file, &pb)?;
        pb.set_position(len);

        // Close downloaded file.
        drop(file);

        if !item.is_single() {
            m.suspend(|| debug!("Unzipping album"));
            let file = File::open(&full_path)?;
            let reader = BufReader::new(file);
            let mut archive = zip::ZipArchive::new(reader)?;

            archive.extract(path)?;
            fs::remove_file(&full_path)?;
            m.suspend(|| debug!("Unzipped and removed original archive"));
        }
        // Cover folder downloading for singles

        pb.finish_and_clear();
        m.println(format!("(Done) {full_title}"))?;

        Ok(())
    }
}

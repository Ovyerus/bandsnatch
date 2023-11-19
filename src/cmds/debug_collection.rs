use crate::cookies;
use clap::Args as ClapArgs;
use serde_json::json;
use soup::prelude::*;
use std::fs::File;
use std::io::Write;

/// Get a view of the `pagedata` blob taken from your user collection page on
/// Bandcamp.
#[derive(Debug, ClapArgs)]
pub struct Args {
    #[arg(short, long, value_name = "COOKIES_FILE", env = "BS_COOKIES")]
    cookies: Option<String>,

    /// Name of the user to grab the profile of.
    #[arg(short, long, env = "BS_USER")]
    user: String,

    /// Show the full collection dump.
    #[arg(short, long)]
    full: bool,

    /// Save the output to a `collection.json` file.
    #[arg(short, long)]
    save: bool,
}

/// Self contained command that outputs the `pagedata` blob from a user's
/// Bandcamp collection page.
pub fn command(
    Args {
        cookies,
        full,
        user,
        save,
    }: Args,
) -> Result<(), Box<dyn std::error::Error>> {
    let cookies_file = cookies.map(|p| {
        let expanded = shellexpand::tilde(&p);
        expanded.into_owned()
    });

    let cookies = cookies::get_bandcamp_cookies(cookies_file.as_deref())?;
    let api = crate::api::Api::new(cookies);

    let body = api
        .client
        .get(&format!("https://bandcamp.com/{user}"))
        .send()?
        .text()?;
    let soup = Soup::new(&body);

    let data_el = soup
        .attr("id", "pagedata")
        .find()
        .expect("Failed to find `pagedata` element on your collection page.");
    let data_blob = data_el
        .get("data-blob")
        .expect("Failed to extract data from element on collection page.");

    let mut jason: serde_json::value::Value = serde_json::from_str(&data_blob).unwrap();
    // Clear out info that we dont want shared
    jason["collection_data"]["redownload_urls"] = json!("[redacted by bandsnatch]");
    jason["collection_data"]["sequence"] = json!("[redacted by bandsnatch]");
    jason["collection_data"]["pending_sequence"] = json!("[redacted by bandsnatch]");
    jason["hidden_data"]["sequence"] = json!("[redacted by bandsnatch]");
    jason["hidden_data"]["pending_sequence"] = json!("[redacted by bandsnatch]");

    let jason = if !full {
        json!({
            "collection_data": jason["collection_data"],
            "fan_data": jason["fan_data"],
            "hidden_data": jason["hidden_data"],
            // "item_cache": jason["item_cache"],
        })
    } else {
        jason
    };

    let pretty = format!("{:#}", jason);

    if save {
        let mut file = File::create("./debug_collection.json")?;
        file.write_all(pretty.as_bytes())?;
        println!("Wrote out collection data to `./debug_collection.json`.");
    } else {
        println!("{pretty}");
    }

    Ok(())
}

mod api;
mod cache;
mod cookies;
mod util;

#[macro_use]
extern crate log;
#[macro_use]
extern crate simple_error;

use env_logger::{Env, DEFAULT_FILTER_ENV};
use std::fs;

use clap::Parser;

const FORMATS: &'static [&'static str] = &[
    "flac",
    "wav",
    "aac-hi",
    "mp3-320",
    "aiff-lossless",
    "vorbis",
    "mp3-v0",
    "alac",
];

fn validate_audio_format(name: &str) -> Result<(), String> {
    if !FORMATS.contains(&name) {
        Err(String::from("format must be one of the following: flac, wav, aac-hi, mp3-320, aiff-lossless, vorbis, mp3-v0, alac"))
    } else {
        Ok(())
    }
}

macro_rules! skip_err {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(e) => {
                println!("An error: {}; skipped.", e);
                continue;
            }
        }
    };
}

#[derive(Parser, Debug)]
#[clap(name = "bcdl", author, version, long_about = None)]
struct Args {
    /// The audio format to download the files in.
    /// Supported formats are: flac, wav, aac-hi, mp3-320, aiff-lossless, vorbis, mp3-v0, alac
    #[clap(short = 'f', long = "format", validator = validate_audio_format)]
    audio_format: String,

    // TODO: make this auto load cookies.json or cookies.txt in current
    // directory if found, or fallback to extracting from Firefox.
    #[clap(short, long, value_name = "COOKIES_FILE")]
    cookies: String,

    /// Perform a trial run without changing anything on the filesystem.
    #[clap(short = 'n', long = "dry-run", default_value_t = false)]
    dry_run: bool,

    /// Whether the program should ignore any cache files found.
    #[clap(short, long, default_value_t = false)]
    force: bool,

    /// The amount of parallel jobs (threads) to use.
    #[clap(short, long, default_value_t = 4)]
    jobs: u8,

    /// The folder to extract downloaded releases to.
    #[clap(
        short,
        long = "output-folder",
        value_name = "FOLDER",
        default_value = "./"
    )]
    output_folder: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: custom format
    let env = Env::default().filter_or(DEFAULT_FILTER_ENV, "bcdl=info");
    env_logger::init_from_env(env);

    let bandcamp_cookies = cookies::get_bandcamp_cookies(Some("./cookies.json"))?;
    let cookie = cookies::cookies_to_string(&bandcamp_cookies);
    let api = api::Api::new(cookie);
    // Make relative to user path
    let cache = cache::Cache::new(String::from("./test/bandcamp-collection-downloader.cache"));

    let api::BandcampPage {
        download_urls,
        page_name: _,
    } = api.get_download_urls("ovyerus").await?;

    let ids = &["p190890686", "p73637968", "r212538021", "p189790127"].map(String::from);

    let style =
        ProgressStyle::with_template("{bar:10} ({bytes}/{total_bytes}) {wide_msg}").unwrap();
    let audio_format = "mp3-320";

    let cache_content = cache.content()?;
    let items = download_urls
        .iter()
        .filter(|&(x, _)| !cache_content.contains(x))
        // Artificial limit for testing.
        .filter(|&(x, _)| ids.contains(x));

    println!("Trying to download {} releases", items.clone().count());

    for (id, url) in items {
        let item = match api.get_digital_item(&url).await {
            Ok(Some(item)) => item,
            Ok(None) => {
                // warn that item doesnt exist
                warn!("Could not find digital item for {id}");
                skip_err!(cache.add(id, "UNKNOWN"));
                continue;
            }
            Err(_) => continue,
        };

        println!(
            "Trying {id}, {} - {} ({:?})",
            item.title,
            item.artist,
            item.is_single()
        );

        // TODO: set up a MultiProgress & assign bars to it, when threading.
        let pb = ProgressBar::new(0);
        pb.set_style(style.clone());

        let path = item.destination_path("./test");
        skip_err!(fs::create_dir_all(&path));

        skip_err!(api.download_item(&item, &path, audio_format, &pb).await);

        if !cache.content().unwrap().contains(id) {
            skip_err!(cache.add(
                id,
                &format!(
                    "{} ({}) by {}",
                    item.title,
                    item.release_year(),
                    item.artist
                )
            ));
        }
    }

    println!("Finished!");

    Ok(())
}

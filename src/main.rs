mod api;
mod cache;
mod cookies;
mod util;

#[macro_use]
extern crate log;
#[macro_use]
extern crate simple_error;

use env_logger::{Env, DEFAULT_FILTER_ENV};
use indicatif::MultiProgress;
use std::sync::{Arc, Mutex};
use tokio::fs;

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
                warn!("An error: {}; skipped.", e);
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
    let api = Arc::new(api::Api::new(cookie));
    // Make relative to user path
    let cache = Arc::new(Mutex::new(cache::Cache::new(String::from(
        "./test/bandcamp-collection-downloader.cache",
    ))));

    let audio_format = "mp3-320";
    let mcache = cache.lock().unwrap();
    let cache_content = mcache.content()?;
    drop(mcache);

    let download_urls = api.get_download_urls("ovyerus").await?.download_urls;

    let items = download_urls
        .into_iter()
        .filter(|(x, _)| !cache_content.contains(x))
        // Artificial limit for testing.
        .take(5)
        .collect::<Vec<_>>();
    println!("Trying to download {} releases", items.len());

    let jobs = 2;
    let queue = util::WorkQueue::from_vec(items);

    let m = Arc::new(MultiProgress::new());

    tokio_scoped::scope(|scope| {
        for i in 0..jobs {
            let api = api.clone();
            let cache = cache.clone();
            let m = m.clone();
            let queue = queue.clone();

            // somehow re-create thread if it panics
            scope.spawn(async move {
                while let Some((id, url)) = queue.get_work() {
                    m.println(format!("thread {i} taking {id}")).unwrap();
                    let item = match api.get_digital_item(&url).await {
                        Ok(Some(item)) => item,
                        Ok(None) => {
                            let cache = cache.lock().unwrap();
                            // warn that item doesnt exist
                            warn!("Could not find digital item for {id}");
                            skip_err!(cache.add(&id, "UNKNOWN"));
                            continue;
                        }
                        Err(_) => continue,
                    };

                    m.println(format!(
                        "Trying {id}, {} - {} ({:?})",
                        item.title,
                        item.artist,
                        item.is_single(),
                    ))
                    .unwrap();

                    let path = item.destination_path("./test");
                    skip_err!(fs::create_dir_all(&path).await);

                    skip_err!(api.download_item(&item, &path, audio_format, &m).await);

                    let cache = cache.lock().unwrap();
                    if !cache.content().unwrap().contains(&id) {
                        skip_err!(cache.add(
                            &id,
                            &format!(
                                "{} ({}) by {}",
                                item.title,
                                item.release_year(),
                                item.artist
                            )
                        ));
                    }
                }
            });
        }
    });

    println!("Finished!");

    Ok(())
}

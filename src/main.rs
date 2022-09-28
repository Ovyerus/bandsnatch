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
use std::{
    path::Path,
    sync::{Arc, Mutex},
};
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
#[clap(name = "bcdl", version, long_about = None)]
struct Args {
    /// The audio format to download the files in.
    /// Supported formats are: flac, wav, aac-hi, mp3-320, aiff-lossless, vorbis, mp3-v0, alac
    #[clap(short = 'f', long = "format", validator = validate_audio_format, env)]
    audio_format: String,

    // TODO: make this auto load cookies.json or cookies.txt in current
    // directory if found, or fallback to extracting from Firefox.
    #[clap(short, long, value_name = "COOKIES_FILE", env)]
    cookies: Option<String>,

    /// Perform a trial run without changing anything on the filesystem.
    // #[clap(short = 'n', long = "dry-run")]
    // dry_run: bool,

    /// Delete's any found cache file and does a from-scratch download run.
    #[clap(short = 'F', long, env)]
    force: bool,

    /// The amount of parallel jobs (threads) to use.
    #[clap(short, long, default_value_t = 4, env)]
    jobs: u8,

    /// Maximum number of releases to download. Useful for testing.
    #[clap(short = 'n', long)]
    limit: Option<usize>,

    /// The folder to extract downloaded releases to.
    #[clap(
        short,
        long = "output-folder",
        value_name = "FOLDER",
        default_value = "./",
        env
    )]
    output_folder: String,

    /// Name of the user to download releases from (must be logged in through cookies).
    #[clap(env = "BCDL_USER")]
    user: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: custom format
    // TODO: make default based on what release target
    let env = Env::default().filter_or(DEFAULT_FILTER_ENV, "bcdl=info");
    env_logger::init_from_env(env);

    let Args {
        audio_format,
        cookies,
        // TODO: Create a trait with needed IO methods, and use that to create a FakeIO thing to log what's going on
        // dry_run,
        force,
        jobs,
        limit,
        output_folder,
        user,
    } = Args::parse();
    // let cookies_file = cookies.or_else(util::find_valid_cookies_file);
    let cookies_file = cookies;
    let root = Path::new(&output_folder);
    let limit = limit.or(Some(usize::MAX)).unwrap();
    // fs::metadata(root).await?.is_dir();
    match fs::metadata(root).await {
        Ok(d) => d.is_dir(),
        Err(_) => false,
    };
    // TODO: ensure root is a directory

    let bandcamp_cookies = cookies::get_bandcamp_cookies(cookies_file.as_deref())?;
    let cookie = cookies::cookies_to_string(&bandcamp_cookies);
    let api = Arc::new(api::Api::new(cookie));
    let cache = Arc::new(Mutex::new(cache::Cache::new(
        root.join("bandcamp-collection-downloader.cache"),
    )));

    let download_urls = api.get_download_urls(&user).await?.download_urls;
    let items = {
        // Lock gets freed after this block.
        let cache_content = cache.lock().unwrap().content()?;

        download_urls
            .into_iter()
            .filter(|(x, _)| force || !cache_content.contains(x))
            .take(limit)
            .collect::<Vec<_>>()
    };
    println!("Trying to download {} releases", items.len());

    let queue = util::WorkQueue::from_vec(items);
    let m = Arc::new(MultiProgress::new());

    // TODO:  dry_run

    tokio_scoped::scope(|scope| {
        for i in 0..jobs {
            let api = api.clone();
            let cache = cache.clone();
            let m = m.clone();
            let queue = queue.clone();
            let audio_format = audio_format.clone();

            // somehow re-create thread if it panics
            scope.spawn(async move {
                while let Some((id, url)) = queue.get_work() {
                    m.suspend(|| debug!("thread {i} taking {id}"));

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

                    // TODO: intialise progressbar with this, and then pass that + m to download
                    m.println(format!(
                        "Trying {id}, {} - {} ({:?})",
                        item.title,
                        item.artist,
                        item.is_single(),
                    ))
                    .unwrap();

                    let path = item.destination_path(root);
                    skip_err!(fs::create_dir_all(&path).await);

                    // TODO: separate cache for failed downloads.
                    // TODO: retries
                    skip_err!(api.download_item(&item, &path, &audio_format, &m).await);

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

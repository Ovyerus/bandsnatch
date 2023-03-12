use clap::{builder::PossibleValuesParser, Args as ClapArgs};
use indicatif::MultiProgress;
use std::{
    path::Path,
    sync::{Arc, Mutex},
};
use tokio::fs;

use crate::{api, cache, cookies, util};

const FORMATS: &[&str] = &[
    "flac",
    "wav",
    "aac-hi",
    "mp3-320",
    "aiff-lossless",
    "vorbis",
    "mp3-v0",
    "alac",
];

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

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// The audio format to download the files in.
    #[arg(short = 'f', long = "format", value_parser = PossibleValuesParser::new(FORMATS), env = "BS_FORMAT")]
    audio_format: String,

    // TODO: make this auto load cookies.json or cookies.txt in current
    // directory if found, or fallback to extracting from Firefox.
    #[arg(short, long, value_name = "COOKIES_FILE", env = "BS_COOKIES")]
    cookies: Option<String>,

    // Return a list of all tracks to be downloaded, without actually downloading them.
    // #[arg(short = 'n', long = "dry-run")]
    // dry_run: bool,
    /// Ignores any found cache file and instead does a from-scratch download run.
    #[arg(short = 'F', long, env = "BS_FORCE")]
    force: bool,

    /// The amount of parallel jobs (threads) to use.
    #[arg(short, long, default_value_t = 4, env = "BS_JOBS")]
    jobs: u8,

    /// Maximum number of releases to download. Useful for testing.
    #[arg(short = 'n', long, env = "BS_LIMIT")]
    limit: Option<usize>,

    /// The folder to extract downloaded releases to.
    #[arg(
        short,
        long = "output-folder",
        value_name = "FOLDER",
        default_value = "./",
        env = "BS_OUTPUT_FOLDER"
    )]
    output_folder: String,

    /// Name of the user to download releases from (must be logged in through cookies).
    #[clap(env = "BS_USER")]
    user: String,
}

pub async fn command(
    Args {
        audio_format,
        cookies,
        force,
        jobs,
        limit,
        output_folder,
        user,
    }: Args,
) -> Result<(), Box<dyn std::error::Error>> {
    let cookies_file = cookies.map(|p| {
        let expanded = shellexpand::tilde(&p);
        expanded.into_owned()
    });
    let root = shellexpand::tilde(&output_folder);
    let root = Path::new(root.as_ref());
    let limit = limit.unwrap_or(usize::MAX);

    let root_exists = match fs::metadata(root).await {
        Ok(d) => Some(d.is_dir()),
        Err(_) => None,
    };

    match root_exists {
        Some(true) => (),
        Some(false) => {
            error!("Cannot use `output-folder`, as it is not a folder. Please delete it and create as a directory, or try a different path.");
            std::process::exit(1);
        }
        None => fs::create_dir_all(root).await?,
    }

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

                    // skip_err!
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

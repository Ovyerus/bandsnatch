use clap::{builder::PossibleValuesParser, Args as ClapArgs};
use crossbeam_utils::thread;
use indicatif::MultiProgress;
use std::{
    fs,
    path::Path,
    sync::{Arc, Mutex},
};

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

    #[arg(short, long, value_name = "COOKIES_FILE", env = "BS_COOKIES")]
    cookies: Option<String>,

    /// Enables some extra debug output in certain scenarios.
    #[arg(long, env = "BS_DEBUG")]
    debug: bool,

    /// Return a list of all tracks to be downloaded, without actually downloading them.
    #[arg(short = 'd', long = "dry-run")]
    dry_run: bool,

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

pub fn command(
    Args {
        audio_format,
        cookies,
        debug,
        dry_run,
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

    let root_exists = match fs::metadata(root) {
        Ok(d) => Some(d.is_dir()),
        Err(_) => None,
    };

    match root_exists {
        Some(true) => (),
        Some(false) => {
            error!("Cannot use `output-folder`, as it is not a folder. Please delete it and create as a directory, or try a different path.");
            std::process::exit(1);
        }
        None => fs::create_dir_all(root)?,
    }

    let cookies = cookies::get_bandcamp_cookies(cookies_file.as_deref())?;
    let api = Arc::new(api::Api::new(cookies));
    let cache = Arc::new(Mutex::new(cache::Cache::new(
        root.join("bandcamp-collection-downloader.cache"),
    )));

    let download_urls = api.get_download_urls(&user)?.download_urls;
    let items = {
        // Lock gets freed after this block.
        let cache_content = cache.lock().unwrap().content()?;

        download_urls
            .into_iter()
            .filter(|(x, _)| force || !cache_content.contains(x))
            .take(limit)
            .collect::<Vec<_>>()
    };

    if dry_run {
        println!("Fetching information for {} found releases", items.len());
    } else {
        println!("Trying to download {} releases", items.len());
    }

    let queue = util::WorkQueue::from_vec(items);
    let m = Arc::new(MultiProgress::new());
    let dry_run_results = Arc::new(Mutex::new(Vec::<String>::new()));

    // TODO:  dry_run

    thread::scope(|scope| {
        for i in 0..jobs {
            let api = api.clone();
            let cache = cache.clone();
            let m = m.clone();
            let queue = queue.clone();
            let audio_format = audio_format.clone();
            let dry_run_results = dry_run_results.clone();

            // somehow re-create thread if it panics
            scope.spawn(move |_| {
                while let Some((id, url)) = queue.get_work() {
                    m.suspend(|| debug!("thread {i} taking {id}"));

                    // skip_err!
                    let item = match api.get_digital_item(&url, &debug) {
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

                    if dry_run {
                        let results_lock = dry_run_results.lock();
                        if let Ok(mut results) = results_lock {
                            results.push(format!("{id}, {} - {}", item.title, item.artist))
                        } else {
                            panic!("dry_run_results is poisoned!!")
                        }
                        continue;
                    }

                    // TODO: intialise progressbar with this, and then pass that + m to download
                    m.println(format!(
                        "Trying {id}, {} - {} ({:?})",
                        item.title,
                        item.artist,
                        item.is_single(),
                    ))
                    .unwrap();

                    let path = item.destination_path(root);
                    skip_err!(fs::create_dir_all(&path));

                    // TODO: separate cache for failed downloads.
                    // TODO: retries
                    skip_err!(api.download_item(&item, &path, &audio_format, &m));

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
    })
    .unwrap();

    if dry_run {
        println!("{}", dry_run_results.lock().unwrap().join("\n"));
        return Ok(());
    }

    println!("Finished!");

    Ok(())
}

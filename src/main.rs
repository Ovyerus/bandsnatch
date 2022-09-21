// use std::thread;
// use std::time::Duration;
// use std::{cmp::min, fmt::Write};

// use indicatif::{ProgressBar, ProgressState, ProgressStyle};

// fn main() {
//     let mut downloaded = 0;
//     let total_size = 231231231;

//     let pb = ProgressBar::new(total_size);
//     pb.set_style(
//         ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
//             .unwrap()
//             .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
//             .progress_chars("#>-")
//     );

//     while downloaded < total_size {
//         let new = min(downloaded + 223211, total_size);
//         downloaded = new;
//         pb.set_position(new);
//         thread::sleep(Duration::from_millis(12));
//     }

//     pb.finish_with_message("downloaded")
// }

mod api;
mod cookies;

#[macro_use]
extern crate simple_error;

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

#[derive(Parser, Debug)]
#[clap(name = "bcdl", author, version, long_about = None)]
struct Args {
    /// The audio format to download the files in.
    /// Supported formats are: flac, wav, aac-hi, mp3-320, aiff-lossless, vorbis, mp3-v0, alac
    #[clap(short = 'f', long = "format", validator = validate_audio_format)]
    audio_format: String,

    /// The folder to extract downloaded releases to.
    #[clap(
        short,
        long = "output-folder",
        value_name = "FOLDER",
        default_value = "./"
    )]
    output_folder: String,

    /// The amount of parallel jobs (threads) to use.
    #[clap(short, long, default_value_t = 4)]
    jobs: u8,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let args = Args::parse();
    // println!("{:?}", args);
    let bandcamp_cookies = cookies::get_bandcamp_cookies(Some("./cookies.json"))?;
    let cookie = cookies::cookies_to_string(&bandcamp_cookies);
    let api = api::Api::new(cookie);
    let result = api.get_download_urls("ovyerus").await?;
    println!("{:?}", result.download_urls);

    Ok(())
}

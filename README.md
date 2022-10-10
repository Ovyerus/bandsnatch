# bandsnatch

> A CLI batch downloader for your Bandcamp collection.

Bandsnatch is a Rust tool for downloading all of your Bandcamp purchases all at
once in your desired format, and being able to be run multiple times when you
buy new releases.

This project is heavily inspired by Ezwen's
[bandcamp-collection-downloader](https://framagit.org/Ezwen/bandcamp-collection-downloader),
which I used myself before this, specifically existing to help me learn Rust,
but also to add some improvements over it that I've wanted.

## State of the Project

This tool is still currently a work in progress, so bugs and other weirdness may
occur. If anything weird happens or something breaks, please open an issue about
it with information and reproduction steps if possible. Specifically testing use
of this with large collections would be very helpful to see if there's any areas
that I need to improve in.

If you're a developer poking around in the code, please note that this is my
first proper project written using Rust, so code quality may be subpar,
especially in terms of memory usage. If you have any ideas to improve the
project in general I'd love to hear them.

## Usage

The most basic usage is along the lines of `bandsnatch -f <format> <username>`,
as it will try to automatically fetch cookies from a local `cookies.json` or
from Firefox. But if this fails you can provide the `-c` option with a path to a
cookies file to use.

For more advanced usage, you can run `bandsnatch -h` to get output similar to
the following.

```
A CLI batch downloader for your Bandcamp collection

Usage: bandsnatch [OPTIONS] --format <AUDIO_FORMAT> <USER>

Arguments:
  <USER>  Name of the user to download releases from (must be logged in through cookies) [env: BS_USER=]

Options:
  -f, --format <AUDIO_FORMAT>   The audio format to download the files in [env: BS_FORMAT=] [possible values: flac, wav, aac-hi, mp3-320, aiff-lossless, vorbis, mp3-v0, alac]
  -c, --cookies <COOKIES_FILE>  [env: BS_COOKIES=]
  -F, --force                   Ignores any found cache file and instead does a from-scratch download run [env: BS_FORCE=]
  -j, --jobs <JOBS>             The amount of parallel jobs (threads) to use [env: BS_JOBS=] [default: 4]
  -n, --limit <LIMIT>           Maximum number of releases to download. Useful for testing [env: BS_LIMIT=]
  -o, --output-folder <FOLDER>  The folder to extract downloaded releases to [env: BS_OUTPUT_FOLDER=] [default: ./]
  -h, --help                    Print help information
  -V, --version                 Print version information
```

Besides these options, you can also use environment variables with the option
name in `SCREAMING_SNAKE_CASE`, prefixed with `BS_`, so that if set up correctly
you can just run `bandsnatch` and have it automatically download your collection
to the folder you want.

### Exmaple

```
bandsnatch -c ./cookies.json -f flac -o ./Music ovyerus
```

This would download my entire music collection into a local "Music" folder, and
also create a `bandcamp-collection-downloader.cache` in the same directory,
which then gets read on future runs in order to skip items it has already
retrieved.

## Authentication

Because Bandsnatch does not manage logging into Bandcamp itself, you need to
provide it the authentication cookies. For Firefox users, you can extract a
`cookies.json` with the
[Cookie Quick Manager extension](https://addons.mozilla.org/en-US/firefox/addon/cookie-quick-manager/),
and on Chrome, you can use the
[Get cookies.txt extension](https://chrome.google.com/webstore/detail/get-cookiestxt/bgaddhkoddajcdgocldbbfleckgcbcid/),
to extract the cookies in the Netscape format, which Bandsnatch also supports.

If you don't provide the `--cookies` option, Bandsnatch will attempt to
automatically find a file named `cookies.json` or `cookies.txt` in the local
directory and load it. Failing that, if you use Firefox on Windows or Linux,
bandsnatch will try to automatically load the cookies from there if possible.

## Installing

Binary builds of Bandsnatch are available on our
[releases page](https://github.com/Ovyerus/bandsnatch/releases) for Windows, Mac
(both ARM & Intel), and Linux (various architectures).

### Homebrew

`brew install ovyerus/tap/bandsnatch`

### Scoop

```
scoop bucket add ovyerus https://github.com/Ovyerus/bucket
scoop install bandsnatch
```

## AUR (unofficial)

An unofficial AUR package is available from [wale](https://github.com/wale) at
https://aur.archlinux.org/packages/bandsnatch. Either use your favourite AUR
helper, or you can install it manually via the following:

```
git clone https://aur.archlinux.org/bandsnatch.git
cd bandsnatch
makepkg -si
```

### Crate

`cargo install bandsnatch`

### From source

Pull this repository and run `cargo build --release`, and look for the
`bandsnatch` binary in `./target/release/`.

## License

This program is licensed under the MIT license (see [LICENSE](./LICENSE) or
https://opensource.org/licenses/MIT).

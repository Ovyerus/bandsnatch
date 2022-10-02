# bandsnatch

> A CLI batch downloader for your Bandcamp collection.

## State of the Project

This tool is still currently a work in progress, so bugs and other weirdness may
occur. If anything weird happens or something breaks, please open an issue about
it with information and reproduction steps if possible.

If you're a developer poking around in the code, please note that this is my
first proper project written using Rust, so code quality may be subpar,
especially in terms of memory usage. If you have any ideas to improve the
project in general I'd love to hear them.

## Usage

The most basic usage is along the lines of `bandsnatch -f <format> <username>`,
as it will try to automatically fetch cookies from a local `cookies.json` or
from Firefox. But if this fails you can provide the `-c` option with a path to a
cookies file to use.

(NB: you can use `bs` instead of `bandsnatch` as the command name in most cases,
makes it faster to type ;) )

For more advanced usage, you can run `bandsnatch -h` to get output similar to
the following.

```
bandsnatch 0.1.0
A CLI batch downloader for your Bandcamp collection

USAGE:
    bandsnatch [OPTIONS] --format <AUDIO_FORMAT> <USER>

ARGS:
    <USER>    Name of the user to download releases from (must be logged in through cookies)
              [env: BS_USER=]

OPTIONS:
    -c, --cookies <COOKIES_FILE>    [env: BS_COOKIES=]
    -f, --format <AUDIO_FORMAT>     The audio format to download the files in. Supported formats
                                    are: flac, wav, aac-hi, mp3-320, aiff-lossless, vorbis, mp3-v0,
                                    alac [env: BS_FORMAT=]
    -F, --force                     Perform a trial run without changing anything on the filesystem.
                                    Delete's any found cache file and does a from-scratch download
                                    run [env: BS_FORCE=]
    -h, --help                      Print help information
    -j, --jobs <JOBS>               The amount of parallel jobs (threads) to use [env: BS_JOBS=]
                                    [default: 4]
    -n, --limit <LIMIT>             Maximum number of releases to download. Useful for testing [env:
                                    BS_LIMIT=]
    -o, --output-folder <FOLDER>    The folder to extract downloaded releases to [env:
                                    BS_OUTPUT_FOLDER=] [default: ./]
    -V, --version                   Print version information
```

Besides these options, you can also use environment variables with the option
name in `SCREAMING_SNAKE_CASE`, prefixed with `BS_`, so that if set up correctly
you can just run `bs` and have it automatically download your collection to the
folder you want.

### Exmaple

```
bs -c ./cookies.json -f flac -o ./Music ovyerus
```

this

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

Currently, you need to pull this repository and build the binary manually using
`cargo build --release`.

Packages/releases with binaries coming soon.

## License

This program is licensed under the MIT license (see [LICENSE](./LICENSE) or
https://opensource.org/licenses/MIT).

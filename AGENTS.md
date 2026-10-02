# Repository Guidelines

## Project Overview

Bandsnatch is a Rust CLI that downloads a logged-in user's Bandcamp collection in a chosen audio format. Repeated runs use a local cache to avoid downloading the same purchases again. Authentication comes from exported Bandcamp cookies; the tool does not log in for the user.

## Architecture & Data Flow

- `src/main.rs` initializes logging, parses clap subcommands, and dispatches `run` or `debug-collection`. `src/cmds/release.rs` is not wired into the CLI.
- `src/cmds/run.rs` loads cookies, creates an `Api`, obtains collection release IDs/URLs, filters against the cache (unless `--force`), applies `--limit`, and distributes releases to scoped worker threads.
- `src/api/mod.rs` uses blocking `reqwest`: scrape Bandcamp page data, paginate the collection, resolve digital items, then download the requested format. Albums are ZIP-extracted; single tracks are retained as files. `src/api/structs/` contains the serde models and destination-path logic.
- `src/util.rs` provides the shared FIFO work queue, filename sanitization, and progress-aware copying. Workers share the API, queue, cache, and results with `Arc`/`Mutex`; there is no async runtime in the active CLI. Most API requests use a rate-limited retry helper, but collection pagination and `debug-collection` make direct requests.
- `src/cache.rs` reads/appends `<output-folder>/bandcamp-collection-downloader.cache` entries as `id| description`. Preserve collection ID identity: failed downloads must not be cached; successful downloads are cached, while missing items and items with no downloads are deliberately marked skipped. `--dry-run` avoids downloads but can still cache those skip cases.

## Key Directories

- `src/cmds/`: user-facing command arguments and workflows (`run.rs`, `debug_collection.rs`).
- `src/api/`: Bandcamp HTTP/page parsing, downloads, and serde models in `structs/`.
- `test/`: saved download/cache data, **not** an automated test suite; it is ignored by Git.
- `.github/workflows/`: cross-platform build and release CI.

## Development Commands

```sh
cargo build --release             # source build; binary in target/release/
cargo run -- --help               # inspect top-level CLI
cargo run -- run --help           # inspect download options
cargo test                        # standard Rust test harness
cargo fmt --check                 # formatting check
cargo clippy --all-targets        # optional local lint check
nix build                         # CI-style Linux/macOS flake build
```

For a real authenticated run: `cargo run -- run -c ./cookies.json -f flac -o ./Music <username>`. Use `--dry-run --limit 1` for a limited manual check; dry-run still queries Bandcamp and can write skipped-item cache entries. No standalone project scripts were found.

## Code Conventions & Common Patterns

- Rust 2021; modules/functions/fields use `snake_case`, types `PascalCase`. Follow neighboring clap derive `#[arg(..., env = "BS_...")]` and serde model definitions rather than inventing parallel configuration paths.
- Commands and API methods generally return `Result<_, Box<dyn std::error::Error>>`; worker-level `skip_err!` logs a warning and continues. Some malformed input/response paths still use `unwrap`/`expect` or skip errors silently: inspect the actual caller before changing failure behavior.
- I/O is synchronous (`reqwest::blocking`); bounded scoped threads and a mutex-backed queue provide parallelism. Keep lock-protected cache access and the distinction between successful downloads, intentional skips, and failures.
- Cookie loading lives in `src/cookies.rs` (JSON exports or Netscape-style text); output/cache paths are handled by `run.rs`. Avoid committing real cookies, downloaded audio, or generated caches.

## Important Files

`src/main.rs` (entry/dispatch), `src/cmds/run.rs` (main workflow/options), `src/api/mod.rs` (Bandcamp requests/downloads), `src/api/structs/digital_item.rs` (release metadata/path), `src/cache.rs` (cache format), `src/cookies.rs` (authentication), `src/util.rs` (queue/copy), `README.md` (user-facing usage), `CHANGELOG.md` (Keep a Changelog/SemVer history).

## Runtime/Tooling Preferences

Cargo is the package manager; `Cargo.toml` sets minimum Rust 1.82.0 and `rust-toolchain.toml` selects stable. `.envrc` uses the `flake.nix` dev shell; the flake packages Linux/macOS targets, while Windows CI builds with Cargo/MSVC. This is a compiled CLI, not a Node/Bun project.

## Testing & QA

There are currently no Rust test cases or test-specific dependencies; `cargo test` alone gives no behavioral coverage. CI builds binaries but does not run tests, clippy, or rustfmt (the workflow notes lint/format work as TODO; Nix packaging disables checks). For behavioral changes, exercise the changed CLI path with an appropriate cookie-backed run or a focused isolated test; prefer `--dry-run --limit 1` when network/authentication are available. Never treat files in `test/` as a test harness or commit private cookie fixtures.

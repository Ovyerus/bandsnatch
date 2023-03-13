# Changelog

All notable changes to Bandsnatch will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2023-03-13

### Fixed

- Some more fixes for some releases that don't have the exact same data
  structure as others.

## [0.2.0] - 2023-03-12

### Breaking Change

The previous behaviour of running the download job with the base command has
been moved into its own subcommand `run` in order to accommodate some features I
plan to add in the future.

### Added

- `--dry-run` flag to get a list of releases Bandsnatch would try to download,
  without actually downloading them.
- `--debug` flag to get some extra information in certain circumstances (Might
  be changed to `--verbose` in the future if I change my mind).

### Fixed

- Fix problem where some releases could crash a thread with
  `` missing field `download_type`  ``.

### Changed

- New `run` subcommand which replaces the previous functionality of running the
  downloader on the base command.

## [0.1.1] - 2022-10-29

### Added

- Create output folder if it doesn't exist, and warn user if it's a file.

### Fixed

- Replace certain characters in the folder structure which may conflict with
  what filesystems allow (e.g. `:`, `\`, `/`)

### Changed

- Upgrade to `clap` 4.0.

## [0.1.0] - 2022-10-02

Initial public release of Bandsnatch.

[unreleased]: https://github.com/Ovyerus/bandsnatch/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/Ovyerus/bandsnatch/releases/tag/v0.2.0
[0.1.1]: https://github.com/Ovyerus/bandsnatch/releases/tag/v0.1.1
[0.1.0]: https://github.com/Ovyerus/bandsnatch/releases/tag/v0.1.0

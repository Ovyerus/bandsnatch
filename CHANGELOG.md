# Changelog

All notable changes to Bandsnatch will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[unreleased]: https://github.com/Ovyerus/bandsnatch/compare/v0.1.0...HEAD
[0.1.1]: https://github.com/Ovyerus/bandsnatch/releases/tag/v0.1.1
[0.1.0]: https://github.com/Ovyerus/bandsnatch/releases/tag/v0.1.0

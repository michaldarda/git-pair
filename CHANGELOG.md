# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Global roster functionality for saving frequently used co-authors
- Quick add co-authors using aliases from global roster
- `--version` and `--help` command line options
- Environment variable `GIT_PAIR_ROSTER_FILE` for custom roster file location

### Changed
- Improved CLI help output with comprehensive examples
- Enhanced per-branch isolation for multi-threaded test safety

### Fixed
- Removed unnecessary `tempfile` dependency, now using only standard library

### Removed
- Dependency on `tempfile` crate

## [0.1.0] - 2025-09-10

### Added
- Initial release of git-pair
- Per-branch co-author configuration
- Git hook integration for automatic co-author attribution
- Commands: `init`, `add`, `clear`, `status`
- Support for multiple co-authors per branch
- Branch-specific configuration isolation
- Automatic Co-authored-by trailer formatting

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-09-12

### Added
- rustup-like installer and uninstaller scripts

### Fixed
- Git hook management now preserves existing hooks instead of overwriting them
- Integration tests updated to work with modern Git default branch naming
- Temporarily disabled crates.io publishing until token is configured

### Changed
- Consolidated hook installation logic for better maintainability
- Improved hook script generation with multiline strings
- Improved help text generation with multiline strings

## [0.1.0] - 2025-09-10

- Initial release of git-pair
- Global roster functionality for saving frequently used co-authors
- Quick add co-authors using aliases from global roster
- `--version` and `--help` command line options
- Environment variable `GIT_PAIR_ROSTER_FILE` for custom roster file location
- Per-branch co-author configuration
- Git hook integration for automatic co-author attribution
- Commands: `init`, `add`, `clear`, `status`
- Support for multiple co-authors per branch
- Branch-specific configuration isolation
- Automatic Co-authored-by trailer formatting

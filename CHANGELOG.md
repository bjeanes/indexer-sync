# Changelog

All notable changes to this project will be documented in this file.

<small>

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

</small>

## [v0.2.0] - Unreleased

### Added

- Short-hand flags for existing sources and destinations
- `-i`/`--interval` flag to activate long-running mode which syncs every `-i`
  seconds.
- Ability to filter indexers to sync by name
- Configurable log levels. Defaults to level 'info' in release builds and
- 'debug' in debug builds.
- Pre-built binaries for Linux using musl
- Pre-built binaries for other semi-common architectures
- Docker build

### Fixed

### Changed

- Improved log output from commandwith adjustable log levels (controllable with
  `RUST_LOG` for now).

### Deprecated

### Removed

### Known Issues

- Sometimes the syncer doesn't correctly identify an existing indexer in
  Sonarr and attempts to create a new one instead.
  ([#1](https://github.com/bjeanes/indexer-sync/issues/1))
- Code quality is poor. This first release should be considered a proof-of-concept.

## [v0.1.0] - 2020-05-18

### Added

- Basic end-to-end sync of Torrent indexers from Jackett into Sonarr
- Pre-built binaries for Linux
- Pre-built binaries for macOS
- Pre-built binaries for Windows

### Known Issues

- Sometimes the syncer doesn't correctly identify an existing indexer in
  Sonarr and attempts to create a new one instead.
  ([#1](https://github.com/bjeanes/indexer-sync/issues/1))
- Code quality is poor. This first release should be considered a proof-of-concept.

[v0.2.0]: https://github.com/bjeanes/indexer-sync/compare/v0.1.0..HEAD
[v0.1.0]: https://github.com/bjeanes/indexer-sync/tree/v0.1.0

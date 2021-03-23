# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## [Unreleased] - ReleaseDate
## [0.1.1] - 2021-03-23
### Added
- [PR#3](https://github.com/EmbarkStudios/gsutil/pull/3) added support for the `-a` flag on the [`cp`](https://cloud.google.com/storage/docs/gsutil/commands/cp) command, allowing you to specify a predefined ACL for the destination GCS object.

### Fixed
- [PR#3](https://github.com/EmbarkStudios/gsutil/pull/3) fixed a bug on the `cp` command which would duplicate the file name component in the destination GCS object.

## [0.1.0] - 2021-01-19

### Added

- Initial add of `gsutil`, mostly ported from [tame-gcs](https://github.com/EmbarkStudios/tame-gcs) examples.

<!-- next-url -->
[Unreleased]: https://github.com/EmbarkStudios/gsutil/compare/0.1.1...HEAD
[0.1.1]: https://github.com/EmbarkStudios/gsutil/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/EmbarkStudios/gsutil/releases/tag/0.1.0

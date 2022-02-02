<div align="center">

# `🚙 gsutil`

[![Embark](https://img.shields.io/badge/embark-open%20source-blueviolet.svg)](https://embark.dev)
[![Embark](https://img.shields.io/badge/discord-ark-%237289da.svg?logo=discord)](https://discord.gg/dAuKfZS)
[![Crates.io](https://img.shields.io/crates/v/gsutil.svg)](https://crates.io/crates/gsutil)
[![Docs](https://docs.rs/gsutil/badge.svg)](https://docs.rs/gsutil)
[![dependency status](https://deps.rs/repo/github/EmbarkStudios/gsutil/status.svg)](https://deps.rs/repo/github/EmbarkStudios/gsutil)
[![Build status](https://github.com/EmbarkStudios/gsutil/workflows/CI/badge.svg)](https://github.com/EmbarkStudios/gsutil/actions)

A small, incomplete replacement for the official [gsutil](https://cloud.google.com/storage/docs/gsutil).

</div>

## Why?

* You need to do basic GCS operations like uploading some objects for eg. CD, and don't need to do every possible thing you can do with GCS.
* You want a single binary with 0 system dependencies and a minimal footprint (the gcloud/gsutil install is over 100MiB, compressed, in addition to requiring a Python install)

## Why not?

* This binary only supports some operations, listed below, if you need other operations they need to be added, or you must use the official gsutil.

## Supported subcommands

* [cat](src/cat.rs) - [Downloads](https://docs.rs/tame-gcs/latest/tame_gcs/objects/struct.Object.html#method.download) and prints an object to stdout.
* [cp](src/cp.rs) - Either [downloads](https://docs.rs/tame-gcs/latest/tame_gcs/objects/struct.Object.html#method.download) an Object and stores it in a local file, or [uploads](https://docs.rs/tame-gcs/latest/tame_gcs/objects/struct.Object.html#method.insert_multipart) a local file as an Object.
* [ls](src/ls.rs) - [Lists](https://docs.rs/tame-gcs/latest/tame_gcs/objects/struct.Object.html#method.list) Objects.
* [signurl](src/signurl.rs) - [Creates](https://docs.rs/tame-gcs/latest/tame_gcs/signed_url/struct.UrlSigner.html) a signed url for an Object.
* [stat](src/stat.rs) - [Shows](https://docs.rs/tame-gcs/latest/tame_gcs/objects/struct.Object.html#method.get) metadata for an Object.

## Contribution

[![Contributor Covenant](https://img.shields.io/badge/contributor%20covenant-v1.4-ff69b4.svg)](CODE_OF_CONDUCT.md)

We welcome community contributions to this project.

Please read our [Contributor Guide](CONTRIBUTING.md) for more information on how to get started.
Please also read our [Contributor Terms](CONTRIBUTING.md/#Contributor-Terms) before you make any contributions.

Any contribution intentionally submitted for inclusion in an Embark Studios project, shall comply with the Rust standard licensing model (MIT + Apache 2.0) and therefore be dual licensed as described below, without any additional terms or conditions:

### License

This [contribution] is dual licensed under EITHER OF

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

For clarity, "your" refers to Embark or any other licensee/user of the contribution.

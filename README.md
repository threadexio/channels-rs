[crates-io]: https://crates.io/crates/channels
[docs-rs]: https://docs.rs/channels/latest/channels
[github-actions]: https://github.com/threadexio/channels-rs/actions/workflows/ci.yaml

[github-rust]: https://github.com/threadexio/channels-rs
[github-c]: https://github.com/threadexio/channels-c

[license-badge]: https://img.shields.io/github/license/threadexio/channels-rs?style=flat-square&labelColor=0d1117&color=decd87
[version-badge]: https://img.shields.io/crates/v/channels?style=flat-square&logo=rust&labelColor=0d1117&color=decd87
[downloads-badge]: https://img.shields.io/crates/d/channels?style=flat-square&logo=rust&labelColor=0d1117&color=decd87

[tests-badge]: https://img.shields.io/github/actions/workflow/status/threadexio/channels-rs/ci.yaml?style=flat-square&logo=github&label=tests&labelColor=0d1117
[docs-badge]: https://img.shields.io/docsrs/channels?style=flat-square&logo=docs.rs&labelColor=0d1117

<div align="center">
  <img src="https://raw.githubusercontent.com/threadexio/channels-rs/master/.github/images/logo.svg" width="250px">

  <!--
  For testing local changes.
  <img src=".github/images/logo.svg" width="250px">
  -->

  <h3>channels-rs</h3>

  <p>
    A crate that allows for easy and fast communication between processes, threads and systems.
  </p>

  [![license-badge]][crates-io]
  [![version-badge]][crates-io]
  [![downloads-badge]][crates-io]

  [![tests-badge]][github-actions]
  [![docs-badge]][docs-rs]

</div>

[`std::io::Read`]: std::io::Read
[`std::io::Write`]: std::io::Write
[`std::sync::mpsc`]: std::sync::mpsc

[`serde::Serialize`]: serde::Serialize
[`serde::Deserialize`]: serde::Deserialize
[`bincode`]: mod@bincode
[`ciborium`]: mod@ciborium
[`serde_json`]: mod@serde_json
[`hmac`]: mod@hmac
[`sha3`]: mod@sha3
[`flate2`]: mod@flate2
[`crc`]: mod@crc

## Repos

This library is available in the following languages:

- **[Rust][github-rust]**
- [C][github-c] (work in progress)

Sender/Receiver types to be used with _any_ type that implements [`std::io::Read`] and [`std::io::Write`].

This crate is similar to [`std::sync::mpsc`] in terms of the API, and most of the documentation
for that module carries over to this crate.

Don't think of these channels as a replacement for [`std::sync::mpsc`], but as another implementation that works over many different transports.

These channels are meant to be used in combination with network sockets, local sockets, pipes, etc.

The differences are:

- Channels **will** block, unless the underlying stream is set as non-blocking.
- The amount of messages that can be queued up before reading is dependent on the underlying stream.

# Features

- 🚀 Async & Sync API
- 📥 Transparent compression
- 🔑 Transparent authentication with `HMAC-SHA3-512`
- ➕ Transparent checksumming with CRC

# cargo features

- `cbor`: Adds support for sending/receiving any type with [`ciborium`].
- `crc`: Adds support for verifying data with CRC checksums.
- `flate2`: Adds support for compressing data with [`flate2`].
- `hmac`: Adds support for cryptographically signing data with [`hmac`] and [`sha3`].
- `json`: Adds support for sending/receiving any typr with [`serde_json`].
- `statistics`: Capture statistic data like: total bytes sent/received, timestamp of last packet, etc
- `tokio`: Adds support for sending/receiving types asynchronously.

## cargo default features

- `serde`: Adds support for sending/receiving any type which implements [`serde::Serialize`] and [`serde::Deserialize`].
- `bincode`: Adds support for sending/receiving any type with [`bincode`].

# Examples

See: [examples/](https://github.com/threadexio/channels-rs/tree/master/examples)

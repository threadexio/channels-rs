[crates-io]: https://crates.io/crates/channels
[docs-rs]: https://docs.rs/channels/latest/channels
[github-actions]: https://github.com/threadexio/channels-rs/actions/workflows/ci.yaml

[license-badge]: https://img.shields.io/github/license/threadexio/channels-rs?style=flat-square&labelColor=0d1117&color=decd87
[version-badge]: https://img.shields.io/crates/v/channels?style=flat-square&logo=rust&labelColor=0d1117&color=decd87
[downloads-badge]: https://img.shields.io/crates/d/channels?style=flat-square&logo=rust&labelColor=0d1117&color=decd87

[tests-badge]: https://img.shields.io/github/actions/workflow/status/threadexio/channels-rs/ci.yaml?style=flat-square&logo=github&label=tests&labelColor=0d1117
[docs-badge]: https://img.shields.io/docsrs/channels?style=flat-square&logo=docs.rs&labelColor=0d1117

<div align="center">
  <img src="https://raw.githubusercontent.com/threadexio/channels-rs/master/.github/images/logo.svg" width="200px">

  [![license-badge]][crates-io]
  [![version-badge]][crates-io]
  [![downloads-badge]][crates-io]

  [![tests-badge]][github-actions]
  [![docs-badge]][docs-rs]

</div>

[`std::io::Read`]: std::io::Read
[`std::io::Write`]: std::io::Write
[`std::sync::mpsc`]: std::sync::mpsc

[`serde`]: mod@serde
[`bincode`]: mod@bincode

[`Serializer`]: crate::serdes::Serializer
[`Deserializer`]: crate::serdes::Deserializer

# **channels-rs**

**channels** is a crate that allows for easy and fast communication between processes, threads and systems.

Sender/Receiver types to be used with _any_ type that implements [`std::io::Read`] and [`std::io::Write`].

This crate is similar to [`std::sync::mpsc`] in terms of the API, and most of the documentation
for that module carries over to this crate.

Don't think of these channels as a replacement for [`std::sync::mpsc`], but as another implementation that works over many different transports.

These channels are meant to be used in combination with network sockets, local sockets, pipes, etc.

The differences are:

- Channels **will** block, unless the underlying stream is set as non-blocking.
- The amount of messages that can be queued up before reading is dependent on the underlying stream.

**:warning: Warning:** This library does not support transparently encryption or authentication of the data. This functionality must be implemented by a [`Serializer`] and [`Deserializer`].

# Features

- `statistics`: Capture statistic data like: total bytes sent/received, timestamp of last packet, etc

## Default features

- `serde`: Adds support for sending/receiving any type with [`serde`] and [`bincode`]

# Examples

For more complete and complex examples see: [examples/](https://github.com/threadexio/channels-rs/tree/master/examples)

## TCP Echo server

```rust no_run
use std::io;
use std::net::TcpListener;

let listener = TcpListener::bind("0.0.0.0:1337").unwrap();

loop {
    let (stream, _) = listener.accept().unwrap();
    let (mut tx, mut rx) = channels::channel(stream.try_clone().unwrap(), stream);

    let client_data: i32 = rx.recv().unwrap();

    println!("Client sent: {}", client_data);

    tx.send(client_data).unwrap();
}
```

## TCP Echo client

```rust no_run
use std::io;
use std::net::TcpStream;

let stream = TcpStream::connect("127.0.0.1:1337").unwrap();
let (mut tx, mut rx) = channels::channel(stream.try_clone().unwrap(), stream);

tx.send(1337_i32).unwrap();

let received_data = rx.recv().unwrap();

assert_eq!(received_data, 1337_i32);
```

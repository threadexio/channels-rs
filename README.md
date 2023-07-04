[crates-io]: https://crates.io/crates/channels

[license-badge]: https://img.shields.io/crates/l/channels?style=for-the-badge
[version-badge]: https://img.shields.io/crates/v/channels?style=for-the-badge
[downloads-badge]: https://img.shields.io/crates/d/channels?style=for-the-badge

# **channels-rs**

[![license-badge]][crates-io]
[![version-badge]][crates-io]
[![downloads-badge]][crates-io]

**channels** is a crate that allows for easy and fast communication between processes, threads and systems.

try_Sender/Receiver types to be used with _any_ type that implements [`std::io::Read`] and [`std::io::Write`].

This crate is similar to [`std::sync::mpsc`] in terms of the API, and most of the documentation
for that module carries over to this crate.

Don't think of these channels as a replacement for [`std::sync::mpsc`], but as another implementation that works over many different transports.

These channels are meant to be used in combination with network sockets, local sockets, pipes, etc. And can be chained with other adapter types to create complex
and structured packets.

The differences are:

- Channels **will** block, unless the underlying stream is set as non-blocking.
- The amount of messages that can be queued up before reading is dependent on the underlying stream.

# Features

- `statistics`: Capture statistic data like: total bytes sent/received, timestamp of last packet, etc

## Default features

- `serde`: Adds support for try_sending/receiving any type with `serde` and `bincode`

# Examples

For more complete and complex examples see: [examples/](https://github.com/threadexio/channels-rs/tree/master/examples)

## Simple echo server

```rust no_run
use std::io;
use std::net::TcpListener;

let listener = TcpListener::bind("0.0.0.0:1337").unwrap();

loop {
    let (stream, _) = listener.accept().unwrap();
    let (mut tx, mut rx) = channels::channel(stream.try_clone().unwrap(), stream);

    let client_data: i32 = rx.try_recv().unwrap();

    println!("Client sent: {}", client_data);

    tx.try_send(client_data).unwrap();
}
```

## Simple echo client

```rust no_run
use std::io;
use std::net::TcpStream;

let stream = TcpStream::connect("127.0.0.1:1337").unwrap();
let (mut tx, mut rx) = channels::channel(stream.try_clone().unwrap(), stream);

tx.try_send(1337_i32).unwrap();

let received_data = rx.try_recv().unwrap();

assert_eq!(received_data, 1337_i32);
```

## Multi-threaded echo server

```rust no_run
use std::net::TcpListener;

let listener = TcpListener::bind("0.0.0.0:1337").unwrap();

loop {
    let (stream, _) = listener.accept().unwrap();

    std::thread::spawn(move || {
        let (mut tx, mut rx) = channels::channel(stream.try_clone().unwrap(), stream);

        loop {
            let client_data: i32 = rx.try_recv().unwrap();

            println!("Client sent: {}", client_data);

            tx.try_send(client_data).unwrap();
        }
    });
}
```

## try_Send/Recv with 2 threads

```rust no_run
use std::io;
use std::net::TcpStream;

let stream = TcpStream::connect("127.0.0.1:1337").unwrap();
let (mut tx, mut rx) = channels::channel(stream.try_clone().unwrap(), stream);

// Receiving thread
let recv_thread = std::thread::spawn(move || loop {
    println!("Received: {}", rx.try_recv().unwrap());
});

// try_Sending thread
let try_send_thread = std::thread::spawn(move || {
    let mut counter: i32 = 0;
    loop {
        tx.try_send(counter).unwrap();
        counter += 1;
    }
});

recv_thread.join().unwrap();
try_send_thread.join().unwrap();
```

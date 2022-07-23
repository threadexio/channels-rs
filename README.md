[crates-io]: https://crates.io/crates/channels

[license-badge]: https://img.shields.io/crates/l/channels?style=for-the-badge
[version-badge]: https://img.shields.io/crates/v/channels?style=for-the-badge
[downloads-badge]: https://img.shields.io/crates/d/channels?style=for-the-badge


# **channels-rs**

[![license-badge]][crates-io]
[![version-badge]][crates-io]
[![downloads-badge]][crates-io]

**channels** is a crate that allows for easy and fast communication between processes, threads and systems.

Anything you might need can be found over at the documentation @ [docs.rs](https://docs.rs/channels)

# Examples
Simple echo server:
```rust
use std::io;
use std::net::TcpListener;

use channels;

	fn main() -> io::Result<()> {
		let listener = TcpListener::bind("0.0.0.0:1337")?;

		loop {
			let (stream, _) = listener.accept()?;
			let (mut tx, mut rx) = channels::channel::<i32, _>(stream);

			let client_data = rx.recv()?;

			println!("Client sent: {}", client_data);

			tx.send(client_data)?;
		}

 	Ok(())
}
```

Simple echo client:
```rust
use std::io;
use std::net::TcpStream;

fn main() -> io::Result<()> {
		let stream = TcpStream::connect("127.0.0.1:1337")?;
		let (mut tx, mut rx) = channels::channel::<i32, _>(stream);

		tx.send(1337_i32)?;

		let received_data = rx.recv()?;

		assert_eq!(received_data, 1337_i32);

 	Ok(())
}
```

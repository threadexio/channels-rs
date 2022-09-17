//! Sender/Receiver types to be used with _any_ type that implements [`std::io::Read`](`std::io::Read`) and [`std::io::Write`](`std::io::Write`).
//!
//! This crate is similar to [`std::sync::mpsc`](std::sync::mpsc) in terms of the API, and most of the documentation
//! for that module carries over to this crate.
//!
//! An important note, when an object is sent through a [`Sender`](Sender), which was passed into [`channel()`](channel),
//! for other code of the same or another process to use it.
//!
//! Don't think of these channels as a replacement for [`std::sync::mpsc`](std::sync::mpsc), but as another implementation that works over many different transports.
//!
//! These channels are meant to be used in combination with network sockets, local sockets, pipes, etc. And can be chained with other adapter types to create complex
//! and structured packets.
//!
//! The differences are:
//!   - Channels **will** block, unless the underlying stream is set as non-blocking.
//!   - The amount of messages that can be queued up before reading is dependent on the underlying stream.
//!
//! # Features
//!   - [`crc`](crate::crc): Adds data validation with CRC
//!
//! # Examples
//!
//! Simple echo server:
//! ```rust
//! use std::io;
//! use std::net::TcpListener;
//!
//! use channels;
//!
//!	fn main() -> io::Result<()> {
//!		let listener = TcpListener::bind("0.0.0.0:1337")?;
//!
//!		loop {
//!			let (stream, _) = listener.accept()?;
//!			let (mut tx, mut rx) = channels::channel::<i32, _>(stream);
//!
//!			let client_data = rx.recv()?;
//!
//!			println!("Client sent: {}", client_data);
//!
//!			tx.send(client_data)?;
//!		}
//!
//! 	Ok(())
//! }
//! ```
//!
//! Simple echo client:
//! ```rust
//! use std::io;
//! use std::net::TcpStream;
//!
//! fn main() -> io::Result<()> {
//!		let stream = TcpStream::connect("127.0.0.1:1337")?;
//!		let (mut tx, mut rx) = channels::channel::<i32, _>(stream);
//!
//!		tx.send(1337_i32)?;
//!
//!		let received_data = rx.recv()?;
//!
//!		assert_eq!(received_data, 1337_i32);
//!
//! 	Ok(())
//! }
//! ```
//!
//! Multi-threaded echo server:
//! ```rust
//! use std::io;
//! use std::net::TcpListener;
//!
//!	fn main() -> io::Result<()> {
//!		let listener = TcpListener::bind("0.0.0.0:1337")?;
//!
//!		loop {
//!			let (stream, _) = listener.accept()?;
//!
//! 		std::thread::spawn(move || {
//! 			let (mut tx, mut rx) = channels::channel::<i32, _>(stream);
//!
//! 			loop {
//! 				let client_data = rx.recv().unwrap();
//!
//! 				println!("Client sent: {}", client_data);
//!
//!					tx.send(client_data).unwrap();
//! 			}
//! 		});
//!		}
//!
//! 	Ok(())
//! }
//! ```
//!
//! Send/Recv with 2 threads:
//! ```rust
//! use std::io;
//!	use std::net::TcpStream;
//!
//!	fn main() -> io::Result<()> {
//!		let stream = TcpStream::connect("127.0.0.1:1337")?;
//!		let (mut tx, mut rx) = channels::channel::<i32, _>(stream);
//!
//!		// Receiving thread
//!		let recv_thread = std::thread::spawn(move || loop {
//!			println!("Received: {}", rx.recv().unwrap());
//!		});
//!
//!		// Sending thread
//!		let send_thread = std::thread::spawn(move || {
//!			let mut counter: i32 = 0;
//!			loop {
//!				tx.send(counter).unwrap();
//!				counter += 1;
//!			}
//!		});
//!
//!		recv_thread.join().unwrap();
//!		send_thread.join().unwrap();
//!
//!		Ok(())
//!	}
//! ```

mod prelude {
	pub use bincode::Options;

	pub use serde::de::DeserializeOwned;
	pub use serde::{Deserialize, Serialize};

	pub use std::io;
	pub use std::io::{Read, Write};
	pub use std::marker::PhantomData;

	pub use crate::error::*;
	pub use crate::io::Buffer;
}

mod io;
mod shared;

mod packet;

mod error;
pub use error::{Error, Result};

mod sender;
pub use sender::Sender;

mod receiver;
pub use receiver::Receiver;

mod crc;

use prelude::*;

use shared::*;

/// A simple type that combines Read and Write.
pub struct RwAdapter<R: Read, W: Write>(R, W);

impl<R: Read, W: Write> RwAdapter<R, W> {
	pub fn new(reader: R, writer: W) -> Self {
		Self(reader, writer)
	}
}

impl<R: Read, W: Write> Read for RwAdapter<R, W> {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		self.0.read(buf)
	}
}

impl<R: Read, W: Write> Write for RwAdapter<R, W> {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		self.1.write(buf)
	}

	fn flush(&mut self) -> std::io::Result<()> {
		self.1.flush()
	}
}

/// Creates a new channel, returning the sender/receiver. This is the same as [`std::sync::mpsc::channel()`](std::sync::mpsc::channel).
pub fn channel<T: Serialize + DeserializeOwned, Rw: Read + Write>(
	s: Rw,
) -> (Sender<T, Rw>, Receiver<T, Rw>) {
	let shared_stream = Outer::new(Inner::new(s));

	(
		Sender::<T, Rw>::new(shared_stream.clone()),
		Receiver::<T, Rw>::new(shared_stream),
	)
}

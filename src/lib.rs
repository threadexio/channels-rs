//! Sender/Receiver types to be used with _any_ type that implements [`std::io::Read`](`std::io::Read`) and [`std::io::Write`](`std::io::Write`).
//!
//! This crate is similar to [`std::sync::mpsc`](std::sync::mpsc) in terms of the API, and most of the documentation
//! for that module carries over to this crate.
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
//! # Limitations
//!   - At this time only objects with a memory footprint smaller than 65KiB or `u16::MAX` bytes can be sent through the channel. This should be enough for anything you might need to send over.
//!     If you need more then you should seriously rethink if you really need all that data sent over. If yes, then this crate will be of little use to you.
//!
//! # Examples
//!
//! Simple echo server:
//! ```no_run
//! use std::io;
//! use std::net::TcpListener;
//!
//! let listener = TcpListener::bind("0.0.0.0:1337").unwrap();
//!
//! loop {
//!     let (stream, _) = listener.accept().unwrap();
//!     let (mut tx, mut rx) = channels::channel::<i32>(stream.try_clone().unwrap(), stream);
//!
//!     let client_data = rx.recv().unwrap();
//!
//!     println!("Client sent: {}", client_data);
//!
//!     tx.send(client_data).unwrap();
//! }
//! ```
//!
//! Simple echo client:
//! ```no_run
//! use std::io;
//! use std::net::TcpStream;
//!
//! let stream = TcpStream::connect("127.0.0.1:1337").unwrap();
//! let (mut tx, mut rx) = channels::channel::<i32>(stream.try_clone().unwrap(), stream);
//!
//! tx.send(1337_i32).unwrap();
//!
//! let received_data = rx.recv().unwrap();
//!
//! assert_eq!(received_data, 1337_i32);
//! ```
//!
//! Multi-threaded echo server:
//! ```no_run
//! use std::io;
//! use std::net::TcpListener;
//!
//! let listener = TcpListener::bind("0.0.0.0:1337").unwrap();
//!
//! loop {
//!     let (stream, _) = listener.accept().unwrap();
//!
//!     std::thread::spawn(move || {
//!         let (mut tx, mut rx) = channels::channel::<i32>(stream.try_clone().unwrap(), stream);
//!
//!         loop {
//!             let client_data = rx.recv().unwrap();
//!
//!             println!("Client sent: {}", client_data);
//!
//!             tx.send(client_data).unwrap();
//!         }
//!     });
//! }
//! ```
//!
//! Send/Recv with 2 threads:
//! ```no_run
//! use std::io;
//! use std::net::TcpStream;
//!
//! let stream = TcpStream::connect("127.0.0.1:1337").unwrap();
//! let (mut tx, mut rx) = channels::channel::<i32>(stream.try_clone().unwrap(), stream);
//!
//! // Receiving thread
//! let recv_thread = std::thread::spawn(move || loop {
//!     println!("Received: {}", rx.recv().unwrap());
//! });
//!
//! // Sending thread
//! let send_thread = std::thread::spawn(move || {
//!     let mut counter: i32 = 0;
//!     loop {
//!         tx.send(counter).unwrap();
//!         counter += 1;
//!     }
//! });
//!
//! recv_thread.join().unwrap();
//! send_thread.join().unwrap();
//! ```

mod crc;
mod packet;

mod error;
pub use error::{Error, Result};

mod sender;
pub use sender::Sender;

mod receiver;
pub use receiver::Receiver;

mod prelude {
	pub use ::std::{
		io::{self, prelude::*, BufReader, BufWriter},
		marker::PhantomData,
	};

	pub(crate) use crate::{error::*, packet};

	pub use ::serde::{de::DeserializeOwned, Deserialize, Serialize};
}

use prelude::*;

/// A tuple containing a [`Sender`](Sender) and a [`Receiver`](Receiver)
pub type Pair<'r, 'w, T> = (Sender<'w, T>, Receiver<'r, T>);

/// Creates a new channel, returning the [`Sender`](Sender)/[`Receiver`](Receiver).
///
/// # Usage
/// ```no_run
/// use std::net::TcpStream;
///
/// let conn = TcpStream::connect("0.0.0.0:1234").unwrap();
///
/// let (mut tx, mut rx) = channels::channel::<i32>(conn.try_clone().unwrap(), conn);
/// ```
pub fn channel<'r, 'w, T: Serialize + DeserializeOwned>(
	r: impl Read + 'r,
	w: impl Write + 'w,
) -> Pair<'r, 'w, T> {
	(Sender::new(w), Receiver::new(r))
}

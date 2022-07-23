//! Sender/Receiver types to be used with _any_ type that implements [`std::io::Read`](`std::io::Read`) and [`std::io::Write`](`std::io::Write`).
//!
//! This crate is similar to [`std::sync::mpsc`](std::sync::mpsc) in term of the API, and most of the documentation
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
//!   - By default, the [`Sender`](Sender) and [`Receiver`](Receiver) types cannot be sent across threads, unless the `mt` feature is enabled.
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

use std::io::{self, Read, Write};
use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use bincode::Options;

#[cfg(not(feature = "mt"))]
use std::{
	cell::{RefCell, RefMut},
	rc::Rc,
};

#[cfg(feature = "mt")]
use std::sync::{Arc, Mutex, MutexGuard};

const MAX_MESSAGE_SIZE: usize = 2_usize.pow(16);

macro_rules! bincode {
	() => {
		bincode::options()
			.reject_trailing_bytes()
			.with_big_endian()
			.with_fixint_encoding()
			.with_limit(MAX_MESSAGE_SIZE as u64)
	};
}

#[derive(Serialize, Deserialize)]
struct MsgHeader {
	pub len: u16,
}

const MSG_HDR_SIZE: usize = std::mem::size_of::<MsgHeader>();

/// The sending-half of the channel. This is the same as [`std::sync::mpsc::Sender`](std::sync::mpsc::Sender),
/// except for a [few key differences](self).
///
/// See [module-level documentation](self).
pub struct Sender<T: Serialize, W: Write> {
	_p: PhantomData<T>,

	#[cfg(not(feature = "mt"))]
	wtr: Rc<RefCell<W>>,

	#[cfg(feature = "mt")]
	wtr: Arc<Mutex<W>>,
}

impl<T: Serialize, W: Write> Sender<T, W> {
	#[cfg(not(feature = "mt"))]
	pub fn new(wtr: Rc<RefCell<W>>) -> Self {
		Self {
			_p: PhantomData,
			wtr,
		}
	}

	#[cfg(feature = "mt")]
	pub fn new(wtr: Arc<Mutex<W>>) -> Self {
		Self {
			_p: PhantomData,
			wtr,
		}
	}

	#[cfg(not(feature = "mt"))]
	/// Get a mutable reference to the underlying data stream.
	pub unsafe fn inner(&mut self) -> RefMut<'_, W> {
		self.wtr.borrow_mut()
	}

	#[cfg(feature = "mt")]
	/// Get a mutable reference to the underlying data stream.
	pub unsafe fn inner(&mut self) -> MutexGuard<'_, W> {
		self.wtr.lock().unwrap()
	}

	/// Attempts to send an object through the data stream.
	///
	/// The method returns as follows:
	///  - `Ok(())`:		The send operation was successful and the object was sent.
	///	 - `Err(error)`:	This is a normal `send()` error and should be handled appropriately.
	pub fn send(&mut self, data: T) -> io::Result<()> {
		let serialized_data = bincode!()
			.serialize(&data)
			.map_err(|x| io::Error::new(io::ErrorKind::Other, x))?;

		let hdr = MsgHeader {
			len: serialized_data.len() as u16,
		};

		let serialized_header = bincode!()
			.serialize(&hdr)
			.map_err(|x| io::Error::new(io::ErrorKind::Other, x))?;

		#[cfg(not(feature = "mt"))]
		let mut writer = self.wtr.borrow_mut();

		#[cfg(feature = "mt")]
		let mut writer = self.wtr.lock().unwrap();

		writer.write_all(&[serialized_header, serialized_data].concat())?;

		Ok(())
	}
}

#[cfg(feature = "mt")]
unsafe impl<T: Serialize, W: Write> Send for Sender<T, W> {}

/// The receiving-half of the channel. This is the same as [`std::sync::mpsc::Receiver`](std::sync::mpsc::Receiver),
/// except for a [few key differences](self).
///
/// See [module-level documentation](self).
pub struct Receiver<T: DeserializeOwned, R: Read> {
	_p: PhantomData<T>,

	#[cfg(not(feature = "mt"))]
	rdr: Rc<RefCell<R>>,

	#[cfg(feature = "mt")]
	rdr: Arc<Mutex<R>>,

	recv_buf: Box<[u8]>,
	recv_cursor: usize,
	msg_hdr: Option<MsgHeader>,
}

impl<T: DeserializeOwned, R: Read> Receiver<T, R> {
	#[cfg(not(feature = "mt"))]
	pub fn new(rdr: Rc<RefCell<R>>) -> Self {
		Self {
			_p: PhantomData,
			recv_buf: vec![0u8; MAX_MESSAGE_SIZE].into_boxed_slice(),
			recv_cursor: 0,
			msg_hdr: None,
			rdr,
		}
	}

	#[cfg(feature = "mt")]
	pub fn new(rdr: Arc<Mutex<R>>) -> Self {
		Self {
			_p: PhantomData,
			recv_buf: vec![0u8; MAX_MESSAGE_SIZE].into_boxed_slice(),
			recv_cursor: 0,
			msg_hdr: None,
			rdr,
		}
	}

	#[cfg(not(feature = "mt"))]
	/// Get a mutable reference to the underlying data stream.
	pub unsafe fn inner(&mut self) -> RefMut<'_, R> {
		self.rdr.borrow_mut()
	}

	#[cfg(feature = "mt")]
	/// Get a mutable reference to the underlying data stream.
	pub unsafe fn inner(&mut self) -> MutexGuard<'_, R> {
		self.rdr.lock().unwrap()
	}

	/// Attempts to read an object from the sender end.
	///
	/// If the underlying data stream is a blocking socket then `recv()` will block until
	/// an object is available.
	///
	/// If the underlying data stream is a non-blocking socket then `recv()` will return
	/// an error with a kind of `std::io::ErrorKind::WouldBlock` whenever the complete object is not
	/// available.
	///
	/// The method returns as follows:
	///  - `Ok(object)`:	The receive operation was successful and an object was returned.
	///  - `Err(error)`:	If `error.kind()` is `std::io::ErrorKind::WouldBlock` then no object
	/// 					is currently available, but one might become available in the future
	/// 					(This can only happen when the underlying stream is set to non-blocking mode).
	///	 - `Err(error)`:	This is a normal `read()` error and should be handled appropriately.
	pub fn recv(&mut self) -> io::Result<T> {
		#[cfg(not(feature = "mt"))]
		let mut reader = self.rdr.borrow_mut();

		#[cfg(feature = "mt")]
		let mut reader = self.rdr.lock().unwrap();

		// check if we haven't read a message header yet
		if self.msg_hdr.is_none() {
			// continuously read to complete the header, if any error is encountered return immediately
			// when working with non-blocking sockets this code returns WouldBlock if there is no data,
			// this is the desired behavior
			while self.recv_cursor != MSG_HDR_SIZE {
				match reader.read(&mut self.recv_buf[self.recv_cursor..MSG_HDR_SIZE]) {
					Ok(v) => self.recv_cursor += v,
					Err(e) => match e.kind() {
						io::ErrorKind::Interrupted => continue,
						_ => return Err(e),
					},
				};
			}

			self.recv_cursor = 0;
			self.msg_hdr = Some(
				bincode!()
					.deserialize(&self.recv_buf[..MSG_HDR_SIZE])
					.map_err(|x| io::Error::new(io::ErrorKind::Other, x))?,
			);
		}

		if let Some(ref hdr) = self.msg_hdr {
			while self.recv_cursor != hdr.len as usize {
				match reader.read(&mut self.recv_buf[self.recv_cursor..hdr.len as usize]) {
					Ok(v) => self.recv_cursor += v,
					Err(e) => match e.kind() {
						io::ErrorKind::Interrupted => continue,
						_ => return Err(e),
					},
				};
			}

			let data = bincode!()
				.deserialize(&self.recv_buf[..hdr.len as usize])
				.map_err(|x| io::Error::new(io::ErrorKind::Other, x))?;

			self.recv_cursor = 0;
			self.msg_hdr = None;

			return Ok(data);
		}

		return Err(io::Error::new(
			io::ErrorKind::WouldBlock,
			"failed to fill buffer",
		));
	}
}

#[cfg(feature = "mt")]
unsafe impl<T: DeserializeOwned, R: Read> Send for Receiver<T, R> {}

/// Creates a new channel, returning the sender/receiver. This is the same as [`std::sync::mpsc::channel()`](std::sync::mpsc::channel).
pub fn channel<T: Serialize + DeserializeOwned, Rw: Read + Write>(
	s: Rw,
) -> (Sender<T, Rw>, Receiver<T, Rw>) {
	#[cfg(not(feature = "mt"))]
	let shared_stream = Rc::new(RefCell::new(s));

	#[cfg(feature = "mt")]
	let shared_stream = Arc::new(Mutex::new(s));

	(
		Sender::<T, Rw>::new(shared_stream.clone()),
		Receiver::<T, Rw>::new(shared_stream.clone()),
	)
}

#[cfg(test)]
mod tests {
	use super::*;

	use std::thread;

	#[test]
	fn test_tcp() {
		use std::net::{TcpListener, TcpStream};

		let listener = TcpListener::bind("127.0.0.35:8000").unwrap();

		let client_thread = thread::spawn(|| {
			let stream = TcpStream::connect("127.0.0.35:8000").unwrap();

			let (mut tx, mut rx) = channel::<i32, _>(stream);

			tx.send(rx.recv().unwrap()).unwrap();
		});

		let (stream, _) = listener.accept().unwrap();
		let (mut tx, mut rx) = channel::<i32, _>(stream);

		tx.send(42).unwrap();

		assert_eq!(rx.recv().unwrap(), 42);

		client_thread.join().unwrap();
	}

	#[test]
	fn test_tcp_server_packet() {
		use std::net::{TcpListener, TcpStream};

		#[derive(Debug, PartialEq, Serialize, Deserialize)]
		enum Packet {
			P1(i32),
			P2(String),
			P3,
			Stop,
		}

		let listener = TcpListener::bind("127.0.0.35:8001").unwrap();

		let client_thread = thread::spawn(|| {
			let stream = TcpStream::connect("127.0.0.35:8001").unwrap();

			let (mut tx, mut rx) = channel::<Packet, _>(stream);

			tx.send(Packet::P1(32)).unwrap();
			assert_eq!(rx.recv().unwrap(), Packet::P1(32));

			println!("[Client] Computing...");
			std::thread::sleep(std::time::Duration::from_secs(1));

			tx.send(Packet::P2("TEST STRING".to_string())).unwrap();
			assert_eq!(rx.recv().unwrap(), Packet::P2("TEST STRING".to_string()));

			println!("[Client] Computing...");
			std::thread::sleep(std::time::Duration::from_secs(2));

			tx.send(Packet::P3).unwrap();
			assert_eq!(rx.recv().unwrap(), Packet::P3);

			tx.send(Packet::Stop).unwrap();
		});

		let (stream, _) = listener.accept().unwrap();
		stream.set_nonblocking(true).unwrap();

		let (mut tx, mut rx) = channel::<Packet, _>(stream);

		loop {
			match rx.recv() {
				Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
				Err(e) => panic!("Error: {}", e),
				Ok(packet) => match packet {
					Packet::P1(n) => {
						println!("Received: P1({})", n);
						tx.send(Packet::P1(n)).unwrap();
					}
					Packet::P2(s) => {
						println!("Received: P2({})", s);
						tx.send(Packet::P2(s)).unwrap();
					}
					Packet::P3 => {
						println!("Received: P3");
						tx.send(Packet::P3).unwrap();
					}
					Packet::Stop => break,
				},
			};

			println!("[Server] Computing...");
			std::thread::sleep(std::time::Duration::from_millis(500));
		}

		client_thread.join().unwrap();
	}
}

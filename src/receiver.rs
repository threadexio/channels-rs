use core::marker::PhantomData;
use std::io::Read;

use crate::error::{Error, Result};
use crate::packet::{self, Header, Packet};
use crate::storage::Buffer;

/// The receiving-half of the channel. This is the same as [`std::sync::mpsc::Receiver`](std::sync::mpsc::Receiver),
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
pub struct Receiver<'a, T> {
	_p: PhantomData<T>,
	rx: Box<dyn Read + 'a>,
	rx_buffer: Buffer,
	seq_no: u16,
}

impl<'a, T> Receiver<'a, T> {
	/// Creates a new [`Receiver`](Receiver) from `rx`.
	///
	/// It is generally recommended to use [`channels::channel`](crate::channel) instead.
	pub fn new(rx: impl Read + 'a) -> Self {
		Self {
			_p: PhantomData,
			rx: Box::new(rx),
			rx_buffer: Buffer::new(Packet::MAX_SIZE),
			seq_no: 0,
		}
	}

	/// Get a reference to the underlying reader.
	pub fn get(&self) -> &dyn Read {
		self.rx.as_ref()
	}

	/// Get a mutable reference to the underlying reader. Directly reading from the stream is not advised.
	pub fn get_mut(&mut self) -> &mut dyn Read {
		self.rx.as_mut()
	}
}

impl<'a, T: serde::de::DeserializeOwned> Receiver<'a, T> {
	fn read(&mut self, amt: usize) -> Result<usize> {
		loop {
			match self.rx.read(&mut self.rx_buffer.after_mut()[..amt])
			{
				Ok(v) => {
					self.rx_buffer.seek_forward(v);
					return Ok(v);
				},
				Err(e) => {
					use std::io::ErrorKind;
					match e.kind() {
						ErrorKind::Interrupted => continue,
						_ => return Err(Error::Io(e)),
					}
				},
			};
		}
	}

	/// Attempts to read an object from the sender end.
	///
	/// If the underlying data stream is a blocking socket then `recv()` will block until
	/// an object is available.
	///
	/// If the underlying data stream is a non-blocking socket then `recv()` will return
	/// an error with a kind of `std::io::ErrorKind::WouldBlock` whenever the complete object is not
	/// available.
	pub fn recv(&mut self) -> Result<T> {
		while self.rx_buffer.len() < Header::MAX_SIZE {
			self.read(Header::MAX_SIZE - self.rx_buffer.len())?;
		}

		let mut packet =
			Packet::new_unchecked(self.rx_buffer.buffer_mut());
		let mut header = packet.header();

		if header.get_version() != packet::PROTOCOL_VERSION {
			return Err(Error::VersionMismatch);
		}

		{
			let unverified = header.get_header_checksum();
			header.set_header_checksum(0);
			let calculated = header.calculate_header_checksum();

			if unverified != calculated {
				return Err(Error::ChecksumError);
			}
		}

		if header.get_id() != self.seq_no {
			return Err(Error::OutOfOrder);
		}

		let packet_len = header.get_length() as usize;
		if packet_len < Header::MAX_SIZE {
			return Err(Error::SizeLimit);
		}

		while self.rx_buffer.len() < packet_len {
			self.read(packet_len - self.rx_buffer.len())?;
		}

		let packet =
			Packet::new_unchecked(self.rx_buffer.buffer_mut());

		let data: Result<T> = packet::deserialize(
			&packet.payload()[..packet_len - Header::MAX_SIZE],
		);

		self.seq_no = self.seq_no.wrapping_add(1);
		self.rx_buffer.clear();

		data
	}
}

unsafe impl<T> Send for Receiver<'_, T> {}
unsafe impl<T> Sync for Receiver<'_, T> {}

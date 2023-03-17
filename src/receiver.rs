use core::marker::PhantomData;
use std::io::Read;

use crate::error::{Error, Result};
use crate::io::{ReadExt, Reader};
use crate::packet::PacketBuffer;

/// The receiving-half of the channel. This is the same as [`std::sync::mpsc::Receiver`],
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
pub struct Receiver<'a, T> {
	_p: PhantomData<T>,
	rx: Reader<Box<dyn Read + 'a>>,
	buffer: PacketBuffer,
	seq_no: u16,
}

impl<'a, T> Receiver<'a, T> {
	/// Creates a new [`Receiver`] from `rx`.
	pub fn new<R>(rx: R) -> Self
	where
		R: Read + 'a,
	{
		Self {
			_p: PhantomData,
			rx: Reader::new(Box::new(rx)),
			buffer: PacketBuffer::new(),
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

	#[cfg(feature = "statistics")]
	/// Get statistics on this [`Receiver`](Self).
	pub fn stats(&self) -> &crate::stats::RecvStats {
		self.rx.stats()
	}
}

impl<'a, T> Receiver<'a, T> {
	/// Attempts to receive an object from the data stream using
	/// a custom deserialization function.
	///
	/// If the underlying data stream is a blocking socket then `recv()` will block until
	/// an object is available.
	///
	/// If the underlying data stream is a non-blocking socket then `recv()` will return
	/// an error with a kind of `std::io::ErrorKind::WouldBlock` whenever the complete object is not
	/// available.
	///
	/// The first parameter passed to `de_fn` is the buffer with the
	/// serialized data.
	///
	/// `de_fn` must return the deserialized object.
	///
	/// # Example
	/// ```no_run
	/// use channels::Receiver;
	///
	/// let mut rx = Receiver::<i32>::new(std::io::empty());
	///
	/// let number = rx.recv_with(|buf| {
	///     let number = i32::from_be_bytes(buf[..2].try_into().unwrap());
	///     Ok(number)
	/// }).unwrap();
	/// ```
	pub fn recv_with<F>(&mut self, de_fn: F) -> Result<T>
	where
		F: FnOnce(&[u8]) -> Result<T>,
	{
		{
			let buf_len = self.buffer.len();
			if buf_len < PacketBuffer::HEADER_SIZE {
				self.rx.fill_buffer(
					&mut self.buffer,
					PacketBuffer::HEADER_SIZE - buf_len,
				)?;

				if let Err(e) = self.buffer.verify_header() {
					self.buffer.clear();
					return Err(e);
				}

				if self.buffer.get_id() != self.seq_no {
					self.buffer.clear();
					return Err(Error::OutOfOrder);
				}
			}
		}

		let packet_len = self.buffer.get_length() as usize;

		{
			let buf_len = self.buffer.len();
			if buf_len < packet_len {
				self.rx.fill_buffer(
					&mut self.buffer,
					packet_len - buf_len,
				)?;
			}
		}

		self.buffer.clear();
		let payload_len = packet_len - PacketBuffer::HEADER_SIZE;

		let payload_buffer = &self.buffer.payload()[..payload_len];
		let data = de_fn(payload_buffer)?;

		self.seq_no = self.seq_no.wrapping_add(1);

		#[cfg(feature = "statistics")]
		self.rx.stats_mut().update_received_time();

		Ok(data)
	}
}

#[cfg(feature = "serde")]
impl<'a, T: serde::de::DeserializeOwned> Receiver<'a, T> {
	/// Attempts to read an object from the sender end.
	///
	/// If the underlying data stream is a blocking socket then `recv()` will block until
	/// an object is available.
	///
	/// If the underlying data stream is a non-blocking socket then `recv()` will return
	/// an error with a kind of `std::io::ErrorKind::WouldBlock` whenever the complete object is not
	/// available.
	///
	/// # Example
	/// ```no_run
	/// use channels::Receiver;
	///
	/// let mut rx = Receiver::<i32>::new(std::io::empty());
	///
	/// let number = rx.recv().unwrap();
	/// ```
	pub fn recv(&mut self) -> Result<T> {
		self.recv_with(crate::serde::deserialize)
	}
}

unsafe impl<T> Send for Receiver<'_, T> {}
unsafe impl<T> Sync for Receiver<'_, T> {}

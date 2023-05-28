use core::marker::PhantomData;

use crate::error::*;
use crate::io::{self, prelude::*, OwnedBuf, Reader};
use crate::packet::*;

/// The receiving-half of the channel. This is the same as [`std::sync::mpsc::Receiver`],
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
pub struct Receiver<'r, T> {
	_p: PhantomData<T>,
	rx: Reader<'r>,
	pbuf: PacketBuf,
	pid: PacketId,
}

unsafe impl<T> Send for Receiver<'_, T> {}
unsafe impl<T> Sync for Receiver<'_, T> {}

impl<'r, T> Receiver<'r, T> {
	/// Creates a new [`Receiver`] from `rx`.
	pub fn new<R>(rx: R) -> Self
	where
		R: Read + 'r,
	{
		Self {
			_p: PhantomData,
			rx: Reader::new(rx),
			pbuf: PacketBuf::new(),
			pid: PacketId::new(),
		}
	}

	/// Get a reference to the underlying reader.
	pub fn get(&self) -> &dyn Read {
		self.rx.get()
	}

	/// Get a mutable reference to the underlying reader. Directly reading from the stream is not advised.
	pub fn get_mut(&mut self) -> &mut dyn Read {
		self.rx.get_mut()
	}
}

#[cfg(feature = "statistics")]
impl<T> Receiver<'_, T> {
	/// Get statistics on this [`Receiver`](Self).
	pub fn stats(&self) -> &crate::stats::RecvStats {
		self.rx.stats()
	}
}

impl<T> Receiver<'_, T> {
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
		let mut payload = OwnedBuf::new(vec![]);

		loop {
			let chunk_len = self.recv_chunk()?;

			let payload_len = payload.len();
			payload.resize(payload_len + chunk_len, 0);

			payload.write_all(&self.pbuf.payload()[..chunk_len])?;

			if !(self.pbuf.get_flags() & PacketFlags::MORE_DATA) {
				break;
			}
		}

		let data = de_fn(&payload)?;

		Ok(data)
	}

	/// Receives exactly one packet of data, returning
	/// the amount of bytes that the payload consists
	/// of. The received payload is written into the
	/// buffer and must be read before further calls.
	#[must_use = "unused payload size"]
	fn recv_chunk(&mut self) -> Result<usize> {
		if self.pbuf.len() < PacketBuf::HEADER_SIZE {
			io::fill_buffer_to(
				&mut self.pbuf,
				&mut self.rx,
				PacketBuf::HEADER_SIZE,
			)?;

			if let Err(e) = self.pbuf.verify_header() {
				self.pbuf.clear();
				return Err(e);
			}

			if self.pbuf.get_id() != self.pid {
				self.pbuf.clear();
				return Err(Error::OutOfOrder);
			}

			self.pid.next_id();
		}

		let packet_len = usize::from(self.pbuf.get_packet_length());

		if self.pbuf.len() < packet_len {
			io::fill_buffer_to(
				&mut self.pbuf,
				&mut self.rx,
				packet_len,
			)?;
		}

		self.pbuf.clear();

		#[cfg(feature = "statistics")]
		self.rx.stats_mut().update_received_time();

		let payload_len = packet_len - PacketBuf::HEADER_SIZE;
		Ok(payload_len)
	}
}

#[cfg(feature = "serde")]
impl<T> Receiver<'_, T>
where
	T: serde::de::DeserializeOwned,
{
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

use core::marker::PhantomData;

use crate::error::*;
use crate::io::{prelude::*, OwnedBuf, Reader};
use crate::packet::*;

/// The receiving-half of the channel. This is the same as [`std::sync::mpsc::Receiver`],
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
pub struct Receiver<T, R> {
	_p: PhantomData<T>,
	rx: Reader<R>,
	pbuf: PacketBuf,
	pid: PacketId,
}

unsafe impl<T, R> Send for Receiver<T, R> {}
unsafe impl<T, R> Sync for Receiver<T, R> {}

impl<T, R> Receiver<T, R> {
	/// Creates a new [`Receiver`] from `rx`.
	pub fn new(rx: R) -> Self {
		Self {
			_p: PhantomData,
			rx: Reader::new(rx),
			pbuf: PacketBuf::new(),
			pid: PacketId::new(),
		}
	}

	/// Get a reference to the underlying reader.
	pub fn get(&self) -> &R {
		self.rx.get()
	}

	/// Get a mutable reference to the underlying reader. Directly reading from the stream is not advised.
	pub fn get_mut(&mut self) -> &mut R {
		self.rx.get_mut()
	}
}

#[cfg(feature = "statistics")]
impl<T, R> Receiver<T, R> {
	/// Get statistics on this [`Receiver`](Self).
	pub fn stats(&self) -> &crate::stats::RecvStats {
		self.rx.stats()
	}
}

impl<T, R> Receiver<T, R>
where
	R: Read,
{
	/// Attempts to receive an object from the data stream using
	/// a custom deserialization function.
	///
	/// If the underlying data stream is a blocking socket then `try_recv()` will block until
	/// an object is available.
	///
	/// If the underlying data stream is a non-blocking socket then `try_recv()` will return
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
	/// let mut rx = Receiver::new(std::io::empty());
	///
	/// let number = rx.try_recv_with(|buf| {
	///     let number = i32::from_be_bytes(buf[..2].try_into().unwrap());
	///     Ok(number)
	/// }).unwrap();
	/// ```
	pub fn try_recv_with<F>(&mut self, de_fn: F) -> Result<T>
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
		let mut fill_buffer_to =
			|buf: &mut OwnedBuf, limit: usize| -> Result<()> {
				let buf_len = buf.len();
				if buf_len < limit {
					self.rx.fill_buffer(buf, limit - buf_len)?;
				}

				Ok(())
			};

		if self.pbuf.len() < PacketBuf::HEADER_SIZE {
			fill_buffer_to(&mut self.pbuf, PacketBuf::HEADER_SIZE)?;

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
			fill_buffer_to(&mut self.pbuf, packet_len)?;
		}

		self.pbuf.clear();

		#[cfg(feature = "statistics")]
		self.rx.stats_mut().update_received_time();

		let payload_len = packet_len - PacketBuf::HEADER_SIZE;
		Ok(payload_len)
	}
}

#[cfg(feature = "serde")]
impl<T, R> Receiver<T, R>
where
	R: Read,
	T: serde::de::DeserializeOwned,
{
	/// Attempts to read an object from the sender end.
	///
	/// If the underlying data stream is a blocking socket then `try_recv()` will block until
	/// an object is available.
	///
	/// If the underlying data stream is a non-blocking socket then `try_recv()` will return
	/// an error with a kind of `std::io::ErrorKind::WouldBlock` whenever the complete object is not
	/// available.
	///
	/// # Example
	/// ```no_run
	/// use channels::Receiver;
	///
	/// let mut rx = Receiver::new(std::io::empty());
	///
	/// let number: i32 = rx.try_recv().unwrap();
	/// ```
	pub fn try_recv(&mut self) -> Result<T> {
		self.try_recv_with(crate::serde::deserialize)
	}

	/// TODO: docs
	pub async fn recv(&mut self) -> Result<T> {
		std::future::poll_fn(|_cx| {
			crate::error::poll_result(self.try_recv())
		})
		.await
	}
}

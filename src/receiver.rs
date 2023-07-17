use core::marker::PhantomData;

use crate::error::RecvError;
use crate::io::{prelude::*, OwnedBuf, Reader};
use crate::packet::*;

use crate::serdes::{self, Deserializer};

/// The receiving-half of the channel. This is the same as [`std::sync::mpsc::Receiver`],
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
pub struct Receiver<T, R, D>
where
	R: Read,
	D: Deserializer<T>,
{
	_p: PhantomData<T>,
	rx: Reader<R>,
	pbuf: PacketBuf,
	pid: PacketId,

	deserializer: D,
}

#[cfg(feature = "serde")]
impl<T, R> Receiver<T, R, serdes::Bincode>
where
	for<'de> T: serde::Deserialize<'de>,
	R: Read,
{
	/// Creates a new [`Receiver`] from `rx`.
	pub fn new(rx: R) -> Self {
		Self::with_deserializer(rx, serdes::Bincode)
	}
}

impl<T, R, D> Receiver<T, R, D>
where
	R: Read,
	D: Deserializer<T>,
{
	/// Create a mew [`Receiver`] from `rx` that uses `deserializer`.
	pub fn with_deserializer(rx: R, deserializer: D) -> Self {
		Self {
			_p: PhantomData,
			rx: Reader::new(rx),
			pbuf: PacketBuf::new(),
			pid: PacketId::new(),
			deserializer,
		}
	}

	/// Get a reference to the underlying reader.
	pub fn get(&self) -> &R {
		self.rx.get()
	}

	/// Get a mutable reference to the underlying reader. Directly
	/// reading from the stream is not advised.
	pub fn get_mut(&mut self) -> &mut R {
		self.rx.get_mut()
	}

	#[cfg(feature = "statistics")]
	/// Get statistics on this [`Receiver`](Self).
	pub fn stats(&self) -> &crate::stats::RecvStats {
		self.rx.stats()
	}

	/// Attempts to receive an object from the data stream using a
	/// custom deserialization function.
	///
	/// - If the underlying reader is a blocking then `recv()` will
	/// block until an object is available.
	///
	/// - If the underlying reader is non-blocking then `recv()` will
	/// return an error with a kind of `std::io::ErrorKind::WouldBlock`
	/// whenever the complete object is not available.
	///
	/// # Example
	/// ```no_run
	/// use channels::Receiver;
	///
	/// let mut rx = Receiver::new(std::io::empty());
	///
	/// let number: i32 = rx.recv().unwrap();
	/// ```
	pub fn recv(&mut self) -> Result<T, RecvError> {
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

		let data = self
			.deserializer
			.deserialize(&payload)
			.map_err(|x| RecvError::Serde(Box::new(x)))?;

		Ok(data)
	}

	/// Receives exactly one packet of data, returning
	/// the amount of bytes that the payload consists
	/// of. The received payload is written into the
	/// buffer and must be read before further calls.
	#[must_use = "unused payload size"]
	fn recv_chunk(&mut self) -> Result<usize, RecvError> {
		let mut fill_buffer_to = |buf: &mut OwnedBuf,
		                          limit: usize|
		 -> Result<(), RecvError> {
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
				return Err(RecvError::OutOfOrder);
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

	/// Get an iterator over incoming messages. The iterator will
	/// return `None` messages when an error is returned by [`Receiver::recv`].
	///
	/// See: [`Incoming`].
	///
	/// # Example
	/// ```no_run
	/// use channels::Receiver;
	///
	/// let mut rx = Receiver::<i32, _, _>::new(std::io::empty());
	///
	/// for number in rx.incoming() {
	///     println!("Received number: {number}");
	/// }
	/// ```
	pub fn incoming(&mut self) -> Incoming<T, R, D> {
		Incoming(self)
	}
}

/// An iterator over incoming messages of a [`Receiver`]. The iterator
/// will return `None` only when [`Receiver::recv`] returns with an error.
///
/// **NOTE:** If the reader is non-blocking then the iterator will return
/// `None` even in the case where [`Receiver::recv`] would return `WouldBlock`.
pub struct Incoming<'r, T, R, D>(&'r mut Receiver<T, R, D>)
where
	R: Read,
	D: Deserializer<T>;

impl<T, R, D> Iterator for Incoming<'_, T, R, D>
where
	R: Read,
	D: Deserializer<T>,
{
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.recv().ok()
	}
}

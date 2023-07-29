//! Module containing the implementation for [`Receiver`].

use core::any::type_name;
use core::fmt;
use core::marker::PhantomData;

use std::io::{self, Read, Write};

use crate::error::{RecvError, VerifyError};
use crate::io::{BytesMut, Cursor, GrowableBuffer, Reader};
use crate::packet::{header::*, Packet};
use crate::serdes::{self, Deserializer};

/// The receiving-half of the channel. This is the same as [`std::sync::mpsc::Receiver`],
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
pub struct Receiver<T, R, D> {
	_p: PhantomData<T>,
	deserializer: D,

	rx: Reader<R>,
	pbuf: Packet<Box<[u8]>>,
	pid: Id,
}

#[cfg(feature = "serde")]
impl<T, R> Receiver<T, R, serdes::Bincode> {
	/// Creates a new [`Receiver`] from `rx`.
	pub fn new(rx: R) -> Self {
		Self::with_deserializer(rx, serdes::Bincode)
	}
}

impl<T, R, D> Receiver<T, R, D> {
	/// Create a mew [`Receiver`] from `rx` that uses `deserializer`.
	pub fn with_deserializer(rx: R, deserializer: D) -> Self {
		Self {
			_p: PhantomData,
			rx: Reader::new(rx),
			pbuf: Packet::new(
				vec![0u8; Packet::<()>::MAX_SIZE].into_boxed_slice(),
			),
			pid: Id::default(),
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
		&self.rx.stats
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

impl<T, R, D> Receiver<T, R, D>
where
	R: Read,
	D: Deserializer<T>,
{
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
	pub fn recv(&mut self) -> Result<T, RecvError<D::Error>> {
		macro_rules! deserialize_payload {
			($p:expr) => {
				self.deserializer
					.deserialize($p)
					.map_err(RecvError::Serde)
			};
		}

		let header = self.recv_chunk()?;

		// If the first packet we read has the `MORE_DATA` flag set,
		// then we must wait for more data, so initialize a buffer to
		// start writing the payload chunks into.
		//
		// Otherwise use directly the payload from the packet buffer,
		// no need to allocate a GrowableBuffer just to copy data that
		// already exists in another part of memory.
		if header.flags & Flags::MORE_DATA {
			macro_rules! write_payload_to_buf {
				($h:expr, $p:expr) => {{
					let payload_len = $h.length.to_payload_length();

					$p.write_all(
						&self.pbuf.payload_slice()
							[..payload_len.as_usize()],
					)?;
				}};
			}

			let mut payload = GrowableBuffer::new();

			write_payload_to_buf!(header, payload);

			loop {
				let header = self.recv_chunk()?;
				write_payload_to_buf!(header, payload);

				if !(header.flags & Flags::MORE_DATA) {
					break;
				}
			}

			let payload = Cursor::new(payload.into_inner());
			let data = deserialize_payload!(payload)?;

			Ok(data)
		} else {
			let payload_len = header.length.to_payload_length();

			let payload = Cursor::new(
				&self.pbuf.payload_slice()[..payload_len.as_usize()],
			);
			let data = deserialize_payload!(payload)?;

			Ok(data)
		}
	}

	/// Receives exactly one packet of data, returning its header. The
	/// received payload is written into the buffer and must be read
	/// before any further calls.
	#[must_use = "unused payload size"]
	fn recv_chunk(&mut self) -> Result<Header, RecvError<D::Error>> {
		if self.pbuf.position() < Header::SIZE {
			fill_buf_to(&mut self.rx, &mut self.pbuf, Header::SIZE)?;

			let header = match self.pbuf.get_header() {
				Ok(v) => v,
				Err(e) => {
					self.pbuf.set_position(0);
					return Err(RecvError::Verify(e));
				},
			};

			if header.id != self.pid {
				self.pbuf.set_position(0);
				return Err(RecvError::Verify(
					VerifyError::OutOfOrder,
				));
			}
		}

		let header = unsafe {
			// SAFETY: The header has been verified above.
			self.pbuf.get_header_unchecked()
		};

		if self.pbuf.position() < header.length.as_usize() {
			fill_buf_to(
				&mut self.rx,
				&mut self.pbuf,
				header.length.as_usize(),
			)?;
		}

		self.pbuf.set_position(0);

		self.pid = self.pid.next();

		#[cfg(feature = "statistics")]
		self.rx.stats.update_received_time();

		Ok(header)
	}
}

/// Continuously call `read` until `buf` reaches position `limit`.
#[inline]
fn fill_buf_to<R, T>(
	reader: &mut Reader<R>,
	buf: &mut Cursor<T>,
	limit: usize,
) -> io::Result<()>
where
	R: Read,
	T: BytesMut,
{
	use io::ErrorKind;

	while buf.position() < limit {
		let pos = buf.position();

		let i = match reader.read(&mut buf.as_mut_slice()[pos..limit])
		{
			Ok(v) if v == 0 => {
				return Err(ErrorKind::UnexpectedEof.into())
			},
			Ok(v) => v,
			Err(e) if e.kind() == ErrorKind::Interrupted => continue,
			Err(e) => return Err(e),
		};

		buf.advance(i).unwrap();
	}

	Ok(())
}

impl<T, R, D> fmt::Debug for Receiver<T, R, D> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Receiver")
			.field("deserializer", &type_name::<D>())
			.field("rx", &self.rx)
			.field("pbuf", &self.pbuf.position())
			.field("pid", &self.pid)
			.finish()
	}
}

/// An iterator over incoming messages of a [`Receiver`]. The iterator
/// will return `None` only when [`Receiver::recv`] returns with an error.
///
/// **NOTE:** If the reader is non-blocking then the iterator will return
/// `None` even in the case where [`Receiver::recv`] would return `WouldBlock`.
///
/// When the iterator returns `None` it does not always mean that further
/// calls to `next()` will also return `None`. This behavior depends on the
/// underlying reader.
pub struct Incoming<'r, T, R, D>(&'r mut Receiver<T, R, D>);

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

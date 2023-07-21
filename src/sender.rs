//! Module containing the implementation for [`Sender`].

use core::any::type_name;
use core::borrow::Borrow;
use core::fmt;
use core::marker::PhantomData;

use std::io::{self, Read, Write};

use crate::error::SendError;
use crate::io::{BytesRef, Cursor, Writer};
use crate::packet::{Buffer, Flags, Header, Id};
use crate::serdes::{self, Serializer};

/// The sending-half of the channel. This is the same as [`std::sync::mpsc::Sender`],
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
pub struct Sender<T, W, S> {
	_p: PhantomData<T>,
	serializer: S,

	tx: Writer<W>,
	pbuf: Buffer,
	pid: Id,
}

#[cfg(feature = "serde")]
impl<T, W> Sender<T, W, serdes::Bincode> {
	/// Creates a new [`Sender`] from `tx`.
	pub fn new(tx: W) -> Self {
		Self::with_serializer(tx, serdes::Bincode)
	}
}

impl<T, W, S> Sender<T, W, S> {
	/// Create a mew [`Sender`] from `tx` that uses `serializer`.
	pub fn with_serializer(tx: W, serializer: S) -> Self {
		Self {
			_p: PhantomData,
			tx: Writer::new(tx),
			pbuf: Buffer::new(),
			pid: Id::default(),
			serializer,
		}
	}

	/// Get a reference to the underlying reader.
	pub fn get(&self) -> &W {
		self.tx.get()
	}

	/// Get a mutable reference to the underlying reader. Directly reading from the stream is not advised.
	pub fn get_mut(&mut self) -> &mut W {
		self.tx.get_mut()
	}

	#[cfg(feature = "statistics")]
	/// Get statistics on this [`Sender`].
	pub fn stats(&self) -> &crate::stats::SendStats {
		self.tx.stats()
	}
}

impl<T, W, S> Sender<T, W, S>
where
	W: Write,
	S: Serializer<T>,
{
	/// Attempts to send an object through the data stream.
	///
	/// # Example
	/// ```no_run
	/// use channels::Sender;
	///
	/// let mut tx = Sender::new(std::io::sink());
	///
	/// tx.send(42_i32).unwrap();
	/// ```
	pub fn send<D>(&mut self, data: D) -> Result<(), SendError>
	where
		D: Borrow<T>,
	{
		let data = data.borrow();
		let serialized_data = self
			.serializer
			.serialize(data)
			.map_err(|x| SendError::Serde(Box::new(x)))?;

		let mut payload = Cursor::new(serialized_data);

		loop {
			let dst: &mut [u8] = self.pbuf.payload_mut();

			// SAFETY:
			//
			// (1) 0 <= chunk_payload_len <= dst.len()      , `read()` cannot violate this
			// (2) dst.len() <= self.pbuf.len()      , see `payload_mut()`
			// (3) self.pbuf.len() = MAX_PACKET_SIZE , see `Buffer::new()`
			// (4) MAX_PACKET_SIZE = u16::MAX        , see `Buffer`
			//
			// (3), (4) <=> self.pbuf.len() = MAX_PACKET_SIZE = u16::MAX
			//          <=> self.pbuf.len() = u16::MAX (5)
			//
			// (2), (5) <=> dst.len() <= self.pbuf.len()
			//          <=> dst.len() <= u16::MAX (6)
			//
			// (1), (6) <=> 0 <= chunk_payload_len <= dst.len()
			//          <=> 0 <= chunk_payload_len <= u16::MAX
			//
			// So `chunk_payload_len` is safe to convert to `u16`. Thus the following
			// cannot panic.
			let chunk_payload_len = payload.read(dst)? as u16;
			if chunk_payload_len == 0 {
				break;
			}

			let mut packet_flags = Flags::zero();

			if payload.position() < payload.len() {
				packet_flags |= Flags::MORE_DATA;
			}

			// SAFETY:
			//
			// (7) dst.len() = self.pbuf.len() - HEADER_SIZE , see `payload_mut()`
			//
			// (1), (7) <=> 0 <= chunk_payload_len <= dst.len()
			//          <=> 0 <= chunk_payload_len <= self.pbuf.len() - HEADER_SIZE
			//          <=> HEADER_SIZE <= chunk_payload_len + HEADER_SIZE <= self.pbuf.len() (8)
			//
			// (3), (8) <=> HEADER_SIZE <= chunk_payload_len + HEADER_SIZE <= u16::MAX
			//
			// So `chunk_payload_len + HEADER_SIZE` still fits inside a u16, so
			// no overflow can occur.
			let header = Header {
				// SAFETY: HEADER_SIZE can't even come close to not
				//         fitting inside a u16.
				length: (Buffer::HEADER_SIZE as u16)
					+ chunk_payload_len,
				flags: packet_flags,
				id: self.pid,
			};

			self.send_packet(&header)?;

			self.pid = self.pid.next();
		}

		#[cfg(feature = "statistics")]
		self.tx.stats_mut().update_sent_time();

		Ok(())
	}

	/// Prepares a packet with `header`. and sends it. The caller must
	/// write payload to the buffer before calling.
	#[inline]
	#[must_use = "unchecked send result"]
	fn send_packet(
		&mut self,
		header: &Header,
	) -> Result<(), SendError> {
		self.pbuf.finalize(header);
		self.pbuf.clear();

		write_buf_to(
			&mut self.tx,
			&mut self.pbuf,
			header.length as usize,
		)?;

		Ok(())
	}
}

/// Continuously call `write` until `buf` reaches position `limit`.
#[inline]
fn write_buf_to<W, T>(
	writer: &mut Writer<W>,
	buf: &mut Cursor<T>,
	limit: usize,
) -> io::Result<()>
where
	W: Write,
	T: BytesRef,
{
	use io::ErrorKind;

	while buf.position() < limit {
		let i = match writer
			.write(&buf.as_slice()[buf.position()..limit])
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

impl<T, W, S> fmt::Debug for Sender<T, W, S> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Sender")
			.field("serializer", &type_name::<S>())
			.field("tx", &self.tx)
			.field("pbuf", &self.pbuf.position())
			.field("pid", &self.pid)
			.finish()
	}
}

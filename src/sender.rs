//! Module containing the implementation for [`Sender`].

use core::any::type_name;
use core::borrow::Borrow;
use core::fmt;
use core::marker::PhantomData;

use std::io::{self, Read, Write};

use crate::error::SendError;
use crate::io::{BytesRef, Chain, Cursor, GrowableBuffer, Writer};
use crate::packet::{header::*, Packet};
use crate::serdes::{self, Serializer};

/// The sending-half of the channel. This is the same as [`std::sync::mpsc::Sender`],
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
pub struct Sender<T, W, S> {
	_p: PhantomData<T>,
	serializer: S,

	tx: Writer<W>,
	pbuf: Packet<Box<[u8]>>,
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
			pbuf: Packet::new(
				vec![0u8; Packet::<()>::MAX_SIZE].into_boxed_slice(),
			),
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
		&self.tx.stats
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
	#[allow(clippy::missing_panics_doc)]
	pub fn send<D>(
		&mut self,
		data: D,
	) -> Result<(), SendError<S::Error>>
	where
		D: Borrow<T>,
	{
		let data = data.borrow();

		// `payload_writer` first writes directly to the payload buffer
		// of the packet and then uses GrowableBuffer. This way, it
		// eliminates the first data copy and avoids an extra allocation.
		// This also means that any payload from the serializer that is
		// less than the maximum payload size does not require any memory
		// allocations in order to be sent.

		let mut payload_writer = {
			let w1 = Cursor::new(self.pbuf.payload_mut_slice());

			match self.serializer.size_hint(data) {
				// If the size hint from the serializer is larger than
				// the payload in the packet buffer, then preallocate
				// the rest to avoid unnecessary allocations and moves
				// later.
				Some(size) if size > w1.len() => {
					let extra_size = size - w1.len();
					Chain::new(
						w1,
						GrowableBuffer::with_capacity(extra_size),
					)
				},
				_ => Chain::new(w1, GrowableBuffer::new()),
			}
		};

		self.serializer
			.serialize(&mut payload_writer, data)
			.map_err(SendError::Serde)?;

		let (first, mut extra) = payload_writer.into_inner();

		{
			let payload_len =
				PayloadLength::from_usize(first.position()).unwrap();

			let mut header = Header {
				length: payload_len.to_packet_length(),
				flags: Flags::zero(),
				id: self.pid,
			};

			if extra.len() != 0 {
				header.flags |= Flags::MORE_DATA;
			}

			self.send_packet(&header)?;
		}

		extra.set_position(0);
		while !extra.remaining().is_empty() {
			// copy the rest of the payload from `extra` into the packet buffer
			let payload_len = PayloadLength::from_usize(
				extra.read(self.pbuf.payload_mut_slice())?,
			)
			.unwrap();

			let mut header = Header {
				length: payload_len.to_packet_length(),
				flags: Flags::zero(),
				id: self.pid,
			};

			if !extra.remaining().is_empty() {
				header.flags |= Flags::MORE_DATA;
			}

			self.send_packet(&header)?;
		}

		Ok(())
	}

	/// Prepares a packet with `header`. and sends it. The caller must
	/// write payload to the buffer before calling.
	#[must_use = "unchecked send result"]
	fn send_packet(
		&mut self,
		header: &Header,
	) -> Result<(), SendError<S::Error>> {
		self.pbuf.finalize(header);

		self.tx.write_all(
			&self.pbuf.as_slice()[..header.length.as_usize()],
		)?;

		self.pid = self.pid.next();

		#[cfg(feature = "statistics")]
		self.tx.stats.update_sent_time();

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

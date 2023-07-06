use core::borrow::Borrow;
use core::marker::PhantomData;

use crate::error::SendError;
use crate::io::{prelude::*, OwnedBuf, Writer};
use crate::packet::*;

use crate::serdes::{self, Serializer};

/// The sending-half of the channel. This is the same as [`std::sync::mpsc::Sender`],
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
pub struct Sender<T, W, S>
where
	W: Write,
	S: Serializer<T>,
{
	_p: PhantomData<T>,
	tx: Writer<W>,
	pbuf: PacketBuf,
	pid: PacketId,

	serializer: S,
}

unsafe impl<T, W, S> Send for Sender<T, W, S>
where
	W: Write,
	S: Serializer<T>,
{
}

unsafe impl<T, W, S> Sync for Sender<T, W, S>
where
	W: Write,
	S: Serializer<T>,
{
}

#[cfg(feature = "serde")]
impl<T, W> Sender<T, W, serdes::Bincode>
where
	T: serde::Serialize,
	W: Write,
{
	/// Creates a new [`Sender`] from `tx`.
	pub fn new(tx: W) -> Self {
		Self::with_serializer(tx, serdes::Bincode)
	}
}

impl<T, W, S> Sender<T, W, S>
where
	W: Write,
	S: Serializer<T>,
{
	/// Create a mew [`Sender`] from `tx` that uses `serializer`.
	pub fn with_serializer(tx: W, serializer: S) -> Self {
		Self {
			_p: PhantomData,
			tx: Writer::new(tx),
			pbuf: PacketBuf::new(),
			pid: PacketId::new(),
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
		let mut payload =
			OwnedBuf::new(self.serializer.serialize(data)?);

		loop {
			let mut dst = self.pbuf.payload_mut();
			let chunk_len = payload.read(&mut dst)?;
			if chunk_len == 0 {
				break;
			}

			let mut flags = PacketFlags::zero();

			if !payload.after().is_empty() {
				flags |= PacketFlags::MORE_DATA;
			}

			self.pbuf.set_flags(flags);
			self.send_chunk(chunk_len)?;
		}

		Ok(())
	}

	/// Prepares a packet with `payload_len` bytes and sends it.
	/// Caller must write payload to the buffer before calling.
	#[must_use = "unchecked send result"]
	fn send_chunk(
		&mut self,
		payload_len: usize,
	) -> Result<(), SendError> {
		let packet_len = PacketBuf::HEADER_SIZE + payload_len;

		#[allow(clippy::as_conversions)]
		self.pbuf.set_packet_length(packet_len as u16);
		self.pbuf.set_id(self.pid);
		self.pbuf.finalize();

		self.pbuf.clear();
		self.tx.write_buffer(&mut self.pbuf, packet_len)?;
		self.pid.next_id();

		#[cfg(feature = "statistics")]
		self.tx.stats_mut().update_sent_time();

		Ok(())
	}
}

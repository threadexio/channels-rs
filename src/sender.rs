use core::borrow::Borrow;
use core::marker::PhantomData;

use crate::error::*;
use crate::io::{prelude::*, OwnedBuf, Writer};
use crate::packet::*;

/// The sending-half of the channel. This is the same as [`std::sync::mpsc::Sender`],
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
pub struct Sender<T, W> {
	_p: PhantomData<T>,
	tx: Writer<W>,
	pbuf: PacketBuf,
	pid: PacketId,
}

unsafe impl<T, W> Send for Sender<T, W> {}
unsafe impl<T, W> Sync for Sender<T, W> {}

impl<T, W> Sender<T, W> {
	/// Creates a new [`Sender`] from `tx`.
	pub fn new(tx: W) -> Self {
		Self {
			_p: PhantomData,
			tx: Writer::new(tx),
			pbuf: PacketBuf::new(),
			pid: PacketId::new(),
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
}

#[cfg(feature = "statistics")]
impl<T, W> Sender<T, W> {
	/// Get statistics on this [`Sender`].
	pub fn stats(&self) -> &crate::stats::SendStats {
		self.tx.stats()
	}
}

impl<T, W> Sender<T, W>
where
	W: Write,
{
	/// Attempts to try_send an object through the data stream
	/// using a custom serialization function.
	///
	/// The first parameter passed to `ser_fn` is the buffer
	/// where serialized data should be put.
	///
	/// The second parameter passed to `sef_fn` is `data`.
	///
	/// `ser_fn` must return the number of bytes the serialized
	///  data consists of.
	///
	/// **NOTE:** The serialized payload must fit within the provided buffer.
	///
	/// # Example
	/// ```no_run
	/// use channels::Sender;
	///
	/// let mut tx = Sender::new(std::io::sink());
	///
	/// tx.try_send_with(42_i32, |data| {
	///     Ok(data.to_be_bytes().to_vec())
	/// }).unwrap();
	/// ```
	pub fn try_send_with<D, F>(
		&mut self,
		data: D,
		ser_fn: F,
	) -> Result<()>
	where
		D: Borrow<T>,
		F: FnOnce(D) -> Result<Vec<u8>>,
	{
		let mut payload = OwnedBuf::new(ser_fn(data)?);

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
	#[must_use = "unchecked try_send result"]
	fn send_chunk(&mut self, payload_len: usize) -> Result<()> {
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

#[cfg(feature = "serde")]
impl<T, W> Sender<T, W>
where
	W: Write,
	T: serde::ser::Serialize,
{
	/// Attempts to try_send an object through the data stream using `serde`.
	///
	/// # Example
	/// ```no_run
	/// use channels::Sender;
	///
	/// #[derive(serde::Serialize)]
	/// struct Data {
	///     a: i32
	/// }
	///
	/// let mut tx = Sender::new(std::io::sink());
	///
	/// tx.try_send(Data { a: 42 }).unwrap();
	/// ```
	pub fn try_send<D>(&mut self, data: D) -> Result<()>
	where
		D: Borrow<T>,
	{
		let data = data.borrow();
		self.try_send_with(data, crate::serde::serialize)
	}

	/// TODO: docs
	pub async fn send<D>(&mut self, data: D) -> Result<()>
	where
		D: Borrow<T>,
	{
		let data = data.borrow();

		std::future::poll_fn(|_cx| {
			crate::error::poll_result(self.try_send(data))
		})
		.await
	}
}

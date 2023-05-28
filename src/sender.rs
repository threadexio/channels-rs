use core::borrow::Borrow;
use core::marker::PhantomData;
use std::io::{Read, Write};

use crate::error::*;
use crate::io::{self, WriteExt, Writer};
use crate::packet::*;

/// The sending-half of the channel. This is the same as [`std::sync::mpsc::Sender`],
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
pub struct Sender<'w, T> {
	_p: PhantomData<T>,
	tx: Writer<'w>,
	pbuf: PacketBuf,
	pid: PacketId,
}

unsafe impl<T> Send for Sender<'_, T> {}
unsafe impl<T> Sync for Sender<'_, T> {}

impl<'w, T> Sender<'w, T> {
	/// Creates a new [`Sender`] from `tx`.
	pub fn new<W>(tx: W) -> Self
	where
		W: Write + 'w,
	{
		Self {
			_p: PhantomData,
			tx: Writer::new(tx),
			pbuf: PacketBuf::new(),
			pid: PacketId::new(),
		}
	}

	/// Get a reference to the underlying reader.
	pub fn get(&self) -> &dyn Write {
		self.tx.get()
	}

	/// Get a mutable reference to the underlying reader. Directly reading from the stream is not advised.
	pub fn get_mut(&mut self) -> &mut dyn Write {
		self.tx.get_mut()
	}
}

#[cfg(feature = "statistics")]
impl<T> Sender<'_, T> {
	/// Get statistics on this [`Sender`].
	pub fn stats(&self) -> &crate::stats::SendStats {
		self.tx.stats()
	}
}

impl<T> Sender<'_, T> {
	/// Attempts to send an object through the data stream
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
	/// let mut tx = Sender::<i32>::new(std::io::sink());
	///
	/// tx.send_with(42, |data| {
	///     Ok(data.to_be_bytes().to_vec())
	/// }).unwrap();
	/// ```
	pub fn send_with<D, F>(
		&mut self,
		data: D,
		ser_fn: F,
	) -> Result<()>
	where
		D: Borrow<T>,
		F: FnOnce(D) -> Result<Vec<u8>>,
	{
		let mut payload = io::OwnedBuf::new(ser_fn(data)?);

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
	fn send_chunk(&mut self, payload_len: usize) -> Result<()> {
		let packet_len = PacketBuf::HEADER_SIZE + payload_len;
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
impl<T> Sender<'_, T>
where
	T: serde::ser::Serialize,
{
	/// Attempts to send an object through the data stream using `serde`.
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
	/// let mut tx = Sender::<Data>::new(std::io::sink());
	///
	/// tx.send(Data { a: 42 }).unwrap();
	/// ```
	pub fn send<D>(&mut self, data: D) -> Result<()>
	where
		D: Borrow<T>,
	{
		let data = data.borrow();
		self.send_with(data, crate::serde::serialize)
	}
}

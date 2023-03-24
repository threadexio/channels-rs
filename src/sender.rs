use core::borrow::Borrow;
use core::marker::PhantomData;
use core::ops::Range;
use std::io::Write;

use crate::error::{Error, Result};
use crate::io::{WriteExt, Writer};
use crate::packet::{layer::*, PacketBuffer};

/// The sending-half of the channel. This is the same as [`std::sync::mpsc::Sender`],
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
pub struct Sender<'a, T> {
	_p: PhantomData<T>,
	tx: Writer<Box<dyn Write + 'a>>,
	buffer: PacketBuffer,
	layers: Id<()>,
}

impl<'a, T> Sender<'a, T> {
	/// Creates a new [`Sender`] from `tx`.
	pub fn new<W>(tx: W) -> Self
	where
		W: Write + 'a,
	{
		Self {
			_p: PhantomData,
			tx: Writer::new(Box::new(tx)),
			buffer: PacketBuffer::new(),
			layers: Id::new(()),
		}
	}

	/// Get a reference to the underlying reader.
	pub fn get(&self) -> &dyn Write {
		self.tx.as_ref()
	}

	/// Get a mutable reference to the underlying reader. Directly reading from the stream is not advised.
	pub fn get_mut(&mut self) -> &mut dyn Write {
		self.tx.as_mut()
	}

	#[cfg(feature = "statistics")]
	/// Get statistics on this [`Sender`].
	pub fn stats(&self) -> &crate::stats::SendStats {
		self.tx.stats()
	}
}

impl<'a, T> Sender<'a, T> {
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
	/// tx.send_with(42, |buf, data| {
	///     let payload = data.to_be_bytes();
	///
	///     // i32 is 2 bytes in size
	///     buf[..2].copy_from_slice(&payload);
	///     Ok(2)
	/// }).unwrap();
	/// ```
	pub fn send_with<D, F>(
		&mut self,
		data: D,
		ser_fn: F,
	) -> Result<()>
	where
		D: Borrow<T>,
		F: FnOnce(&mut [u8], D) -> Result<usize>,
	{
		let payload_buffer =
			self.layers.payload(self.buffer.payload_mut());
		let payload_len = ser_fn(payload_buffer, data)?;
		if payload_len > payload_buffer.len() {
			return Err(Error::SizeLimit);
		}

		self.buffer.set_version(PacketBuffer::VERSION);

		let lp = payload_len; // Payload length

		let Range { start: sb, .. } = self.buffer.as_ptr_range();
		let sb = sb as usize; // Buffer start

		let payload_buffer =
			self.layers.on_send(self.buffer.payload_mut())?;

		let Range { start: sp, .. } = payload_buffer.as_ptr_range();
		let sp = sp as usize; // Payload start
		let ep = sp + lp; // Payload end

		// sp <= ep (1), payload length is bigger or equal to 0 bytes
		// sb < sp  (2), payload start is always bigger than the buffer start (there are headers preceding it)

		// (2), (1) <=> sb < sp <= ep
		//          <=> sb < ep
		//          <=> ep - sb > 0, no overflows
		let packet_len = ep - sb;
		self.buffer.set_length(packet_len as u16);

		self.buffer.update_header_checksum();
		self.buffer.clear();
		self.tx.write_buffer(&mut self.buffer, packet_len)?;

		#[cfg(feature = "statistics")]
		self.tx.stats_mut().update_sent_time();

		Ok(())
	}
}

#[cfg(feature = "serde")]
impl<'a, T: serde::Serialize> Sender<'a, T> {
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
		self.send_with(data, |buf, data| {
			let payload = crate::serde::serialize(data.borrow())?;

			let payload_len = payload.len();
			if payload_len > buf.len() {
				return Err(Error::SizeLimit);
			}

			buf[..payload_len].copy_from_slice(&payload);
			Ok(payload_len)
		})
	}
}

unsafe impl<T> Send for Sender<'_, T> {}
unsafe impl<T> Sync for Sender<'_, T> {}

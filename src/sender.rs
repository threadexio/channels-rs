use core::borrow::Borrow;
use core::marker::PhantomData;
use std::io::{self, Write};

use crate::error::{Error, Result};
use crate::packet::PacketBuffer;
use crate::storage::Buffer;
use crate::util;

#[cfg(feature = "statistics")]
use crate::stats;

/// The sending-half of the channel. This is the same as [`std::sync::mpsc::Sender`](std::sync::mpsc::Sender),
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
pub struct Sender<'a, T> {
	_p: PhantomData<T>,
	tx: Box<dyn Write + 'a>,
	buffer: PacketBuffer,
	seq_no: u16,

	#[cfg(feature = "statistics")]
	stats: stats::SendStats,
}

impl<'a, T> Sender<'a, T> {
	/// Creates a new [`Sender`](Sender) from `reader`.
	///
	/// It is generally recommended to use [`channels::channel`](crate::channel) instead.
	pub fn new<W>(tx: W) -> Self
	where
		W: Write + 'a,
	{
		Self {
			_p: PhantomData,
			tx: Box::new(tx),
			//tx_buffer: Packet::new(),
			buffer: PacketBuffer::new(),
			seq_no: 0,

			#[cfg(feature = "statistics")]
			stats: stats::SendStats::new(),
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
	/// Get statistics on this [`Sender`](Self).
	pub fn stats(&self) -> &stats::SendStats {
		&self.stats
	}
}

impl<'a, T: serde::Serialize> Sender<'a, T> {
	/// Attempts to send an object through the data stream.
	pub fn send<D>(&mut self, data: D) -> Result<()>
	where
		D: Borrow<T>,
	{
		#[inline(never)]
		#[track_caller]
		fn write_all<W, F>(
			buffer: &mut Buffer,
			writer: &mut W,
			limit: usize,
			mut write_cb: F,
		) -> Result<()>
		where
			W: Write,
			F: FnMut(usize),
		{
			let mut bytes_sent: usize = 0;
			while bytes_sent < limit {
				let remaining = limit - bytes_sent;

				let i = match writer
					.write(&buffer.after()[..remaining])
				{
					Ok(v) if v == 0 => {
						return Err(Error::Io(
							io::ErrorKind::UnexpectedEof.into(),
						))
					},
					Ok(v) => v,
					Err(e)
						if e.kind() == io::ErrorKind::Interrupted =>
					{
						continue
					},
					Err(e) => return Err(Error::Io(e)),
				};

				bytes_sent += i;
				buffer.seek_forward(i);

				write_cb(i);
			}

			Ok(())
		}

		let payload = util::serialize(data.borrow())?;
		let payload_len: u16 =
			payload.len().try_into().map_err(|_| Error::SizeLimit)?;

		self.buffer.payload_mut()[..payload_len as usize]
			.copy_from_slice(&payload);
		drop(payload);

		self.buffer.set_version(PacketBuffer::VERSION);

		let packet_len =
			PacketBuffer::HEADER_SIZE + payload_len as usize;
		self.buffer.set_length(packet_len as u16);
		self.buffer.set_id(self.seq_no);
		self.buffer.recalculate_header_checksum();

		self.buffer.reset();
		write_all(
			self.buffer.buffer_mut(),
			&mut self.tx,
			packet_len,
			|_x: usize| {
				#[cfg(feature = "statistics")]
				self.stats.add_sent(_x);
			},
		)?;

		self.seq_no = self.seq_no.wrapping_add(1);

		#[cfg(feature = "statistics")]
		self.stats.update_sent_time();

		Ok(())
	}
}

unsafe impl<T> Send for Sender<'_, T> {}
unsafe impl<T> Sync for Sender<'_, T> {}

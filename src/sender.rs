use core::borrow::Borrow;
use core::marker::PhantomData;
use std::io::{self, Write};

use crate::error::{Error, Result};
use crate::io::Writer;
use crate::packet::PacketBuffer;
use crate::storage::Buffer;
use crate::util;

/// The sending-half of the channel. This is the same as [`std::sync::mpsc::Sender`],
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
pub struct Sender<'a, T> {
	_p: PhantomData<T>,
	tx: Writer<Box<dyn Write + 'a>>,
	buffer: PacketBuffer,
	seq_no: u16,
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
			seq_no: 0,
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

impl<'a, T: serde::Serialize> Sender<'a, T> {
	/// Attempts to send an object through the data stream.
	pub fn send<D>(&mut self, data: D) -> Result<()>
	where
		D: Borrow<T>,
	{
		#[inline(never)]
		#[track_caller]
		fn write_all<W>(
			buffer: &mut Buffer,
			writer: &mut W,
			limit: usize,
		) -> Result<()>
		where
			W: Write,
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
		)?;

		self.seq_no = self.seq_no.wrapping_add(1);

		#[cfg(feature = "statistics")]
		self.tx.stats_mut().update_sent_time();

		Ok(())
	}
}

unsafe impl<T> Send for Sender<'_, T> {}
unsafe impl<T> Sync for Sender<'_, T> {}

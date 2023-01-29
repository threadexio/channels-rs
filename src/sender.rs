use core::borrow::Borrow;
use core::marker::PhantomData;
use std::io::Write;

use crate::error::{Error, Result};
use crate::packet::Packet;
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
	tx_buffer: Packet,
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
			tx_buffer: Packet::new(),
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
		let payload = util::serialize(data.borrow())?;
		let payload_len: u16 =
			payload.len().try_into().map_err(|_| Error::SizeLimit)?;

		let packet = &mut self.tx_buffer;

		packet.set_payload_length(payload_len)?;
		packet.payload_mut()[..payload_len as usize]
			.copy_from_slice(&payload);
		drop(payload);

		self.tx.write_all(packet.finalize_with(|packet| {
			packet.set_id(self.seq_no);
		}))?;

		#[cfg(feature = "statistics")]
		{
			self.stats.update_sent_time();
			self.stats.add_sent(packet.get_length() as usize);
		}

		self.seq_no = self.seq_no.wrapping_add(1);
		Ok(())
	}
}

unsafe impl<T> Send for Sender<'_, T> {}
unsafe impl<T> Sync for Sender<'_, T> {}

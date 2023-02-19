use core::marker::PhantomData;
use std::io::Read;

use crate::error::{Error, Result};
use crate::packet::Packet;
use crate::storage::ReadExt;
use crate::util;

#[cfg(feature = "statistics")]
use crate::stats;

/// The receiving-half of the channel. This is the same as [`std::sync::mpsc::Receiver`](std::sync::mpsc::Receiver),
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
pub struct Receiver<'a, T> {
	_p: PhantomData<T>,
	rx: Box<dyn Read + 'a>,
	rx_buffer: Packet,
	seq_no: u16,

	#[cfg(feature = "statistics")]
	stats: stats::RecvStats,
}

impl<'a, T> Receiver<'a, T> {
	/// Creates a new [`Receiver`](Receiver) from `rx`.
	///
	/// It is generally recommended to use [`channels::channel`](crate::channel) instead.
	pub fn new<R>(rx: R) -> Self
	where
		R: Read + 'a,
	{
		Self {
			_p: PhantomData,
			rx: Box::new(rx),
			rx_buffer: Packet::new(),
			seq_no: 0,

			#[cfg(feature = "statistics")]
			stats: stats::RecvStats::new(),
		}
	}

	/// Get a reference to the underlying reader.
	pub fn get(&self) -> &dyn Read {
		self.rx.as_ref()
	}

	/// Get a mutable reference to the underlying reader. Directly reading from the stream is not advised.
	pub fn get_mut(&mut self) -> &mut dyn Read {
		self.rx.as_mut()
	}

	#[cfg(feature = "statistics")]
	/// Get statistics on this [`Receiver`](Self).
	pub fn stats(&self) -> &stats::RecvStats {
		&self.stats
	}
}

impl<'a, T: serde::de::DeserializeOwned> Receiver<'a, T> {
	/// Attempts to read an object from the sender end.
	///
	/// If the underlying data stream is a blocking socket then `recv()` will block until
	/// an object is available.
	///
	/// If the underlying data stream is a non-blocking socket then `recv()` will return
	/// an error with a kind of `std::io::ErrorKind::WouldBlock` whenever the complete object is not
	/// available.
	pub fn recv(&mut self) -> Result<T> {
		let _i = self.rx.fill_buf_to(
			self.rx_buffer.buffer(),
			Packet::MAX_HEADER_SIZE.into(),
		)?;

		#[cfg(feature = "statistics")]
		self.stats.add_received(_i);

		if let Err(e) = self.rx_buffer.verify_with(|packet| {
			if packet.get_id() != self.seq_no {
				return Err(Error::OutOfOrder);
			}

			Ok(())
		}) {
			self.rx_buffer.buffer().clear();
			return Err(e);
		}

		let packet_len = self.rx_buffer.get_length().into();

		let _i = self
			.rx
			.fill_buf_to(self.rx_buffer.buffer(), packet_len)?;

		#[cfg(feature = "statistics")]
		self.stats.add_received(_i);

		let data: Result<T> = util::deserialize(
			&self.rx_buffer.payload()[..packet_len
				.saturating_sub(Packet::MAX_HEADER_SIZE.into())],
		);

		#[cfg(feature = "statistics")]
		self.stats.update_received_time();

		self.seq_no = self.seq_no.wrapping_add(1);
		self.rx_buffer.buffer().clear();

		data
	}
}

unsafe impl<T> Send for Receiver<'_, T> {}
unsafe impl<T> Sync for Receiver<'_, T> {}

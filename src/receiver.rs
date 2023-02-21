use core::marker::PhantomData;
use std::io::{self, Read};

use crate::error::{Error, Result};
use crate::packet::PacketBuffer;
use crate::storage::Buffer;
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
	//rx_buffer: Packet,
	buffer: PacketBuffer,
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
			//rx_buffer: Packet::new(),
			buffer: PacketBuffer::new(),
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
		#[inline(never)]
		#[track_caller]
		fn fill_buf<R, F>(
			buffer: &mut Buffer,
			reader: &mut R,
			limit: usize,
			mut read_cb: F,
		) -> Result<()>
		where
			R: Read,
			F: FnMut(usize),
		{
			let mut bytes_read: usize = 0;
			while limit > bytes_read {
				let remaining = limit - bytes_read;

				let i = match reader
					.read(&mut buffer.after_mut()[..remaining])
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

				buffer.seek_forward(i);
				bytes_read += i;

				read_cb(i);
			}

			Ok(())
		}

		let mut read_cb = |_x: usize| {
			#[cfg(feature = "statistics")]
			self.stats.add_received(_x);
		};

		let buf_len = self.buffer.buffer().len();
		if buf_len < PacketBuffer::HEADER_SIZE {
			fill_buf(
				self.buffer.buffer_mut(),
				&mut self.rx,
				PacketBuffer::HEADER_SIZE - buf_len,
				&mut read_cb,
			)?;

			debug_assert_eq!(
				self.buffer.buffer().len(),
				PacketBuffer::HEADER_SIZE
			);

			if let Err(e) = self.buffer.verify(self.seq_no) {
				self.buffer.reset();
				return Err(e);
			}
		}

		let packet_len = self.buffer.get_length() as usize;
		let buf_len = self.buffer.buffer().len();
		if buf_len < packet_len {
			fill_buf(
				self.buffer.buffer_mut(),
				&mut self.rx,
				packet_len - buf_len,
				&mut read_cb,
			)?;

			debug_assert_eq!(self.buffer.buffer().len(), packet_len);
		}

		self.buffer.reset();
		let payload_len = packet_len - PacketBuffer::HEADER_SIZE;
		let data = util::deserialize::<T>(
			&self.buffer.payload()[..payload_len],
		)?;

		self.seq_no = self.seq_no.wrapping_add(1);

		#[cfg(feature = "statistics")]
		self.stats.update_received_time();

		Ok(data)
	}
}

unsafe impl<T> Send for Receiver<'_, T> {}
unsafe impl<T> Sync for Receiver<'_, T> {}

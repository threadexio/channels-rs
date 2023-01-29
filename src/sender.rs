use core::marker::PhantomData;
use std::io::Write;

use crate::error::{Error, Result};
use crate::packet::{self, Header, Packet};
use crate::storage::Buffer;

/// The sending-half of the channel. This is the same as [`std::sync::mpsc::Sender`](std::sync::mpsc::Sender),
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
pub struct Sender<'a, T> {
	_p: PhantomData<T>,
	tx: Box<dyn Write + 'a>,
	tx_buffer: Buffer,
	seq_no: u16,
}

impl<'a, T> Sender<'a, T> {
	/// Creates a new [`Sender`](Sender) from `reader`.
	///
	/// It is generally recommended to use [`channels::channel`](crate::channel) instead.
	pub fn new(tx: impl Write + 'a) -> Self {
		Self {
			_p: PhantomData,
			tx: Box::new(tx),
			tx_buffer: Buffer::new(Packet::MAX_SIZE),
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
}

impl<'a, T: serde::Serialize> Sender<'a, T> {
	/// Attempts to send an object through the data stream.
	pub fn send(&mut self, data: T) -> Result<()> {
		let payload = packet::serialize(&data)?;
		let payload_len = payload.len();

		if payload_len > Packet::MAX_SIZE - Header::MAX_SIZE {
			return Err(Error::SizeLimit);
		}

		let mut packet =
			Packet::new_unchecked(self.tx_buffer.buffer_mut());

		packet.payload_mut()[..payload_len].copy_from_slice(&payload);
		drop(payload);

		let mut header = packet.header();

		header.set_version(packet::PROTOCOL_VERSION);
		header.set_id(self.seq_no);

		// the cast to u16 is safe because we validated it above
		// when checking for Error::SizeLimit
		let packet_len = Header::MAX_SIZE + payload_len;
		header.set_length(packet_len as u16);
		header.set_header_checksum(0);

		{
			let checksum = header.calculate_header_checksum();
			header.set_header_checksum(checksum);
		}

		self.tx.write_all(&packet.packet()[..packet_len])?;

		self.seq_no = self.seq_no.wrapping_add(1);

		Ok(())
	}
}

unsafe impl<T> Send for Sender<'_, T> {}
unsafe impl<T> Sync for Sender<'_, T> {}

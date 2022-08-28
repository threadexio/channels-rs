use crate::prelude::*;

use crate::packet::{self, Header};
use crate::shared::*;

/// The sending-half of the channel. This is the same as [`std::sync::mpsc::Sender`](std::sync::mpsc::Sender),
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
pub struct Sender<T: Serialize, W: Write> {
	_p: PhantomData<T>,

	writer: Shared<W>,

	#[cfg(feature = "crc")]
	pub crc: crate::crc::Crc,
}

impl<T: Serialize, W: Write> Sender<T, W> {
	pub(crate) fn new(writer: Shared<W>) -> Self {
		Self {
			_p: PhantomData,
			writer,

			#[cfg(feature = "crc")]
			crc: Default::default(),
		}
	}

	/// Get a handle to the underlying stream. Directly writing to the stream is not advised.
	pub fn inner(&self) -> &mut W {
		self.writer.get()
	}

	/// Attempts to send an object through the data stream.
	pub fn send(&mut self, data: T) -> Result<()> {
		let data_length = packet::serialized_size(&data)?;

		if data_length > packet::MAX_PAYLOAD_SIZE.into() {
			return Err(Error::DataTooLarge);
		}

		let data = packet::serialize(&data)?;

		let mut header = Header::default();

		header.payload_len = data_length as u16;

		#[cfg(feature = "crc")]
		{
			header.payload_checksum = self.crc.checksum16(&data);
		}

		let header = packet::serialize(&header)?;

		let writer = self.writer.get();

		writer.write_all(&[header, data].concat())?;

		Ok(())
	}
}

unsafe impl<T: Serialize, W: Write> Send for Sender<T, W> {}

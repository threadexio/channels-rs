use crate::prelude::*;

use crate::packet::{self, Header};
use crate::shared::*;

/// The sending-half of the channel. This is the same as [`std::sync::mpsc::Sender`](std::sync::mpsc::Sender),
/// except for a [few key differences](self).
///
/// See [module-level documentation](self).
pub struct Sender<T: Serialize, W: Write> {
	_p: PhantomData<T>,

	writer: Shared<W>,
}

impl<T: Serialize, W: Write> Sender<T, W> {
	pub(crate) fn new(writer: Shared<W>) -> Self {
		Self {
			_p: PhantomData,
			writer,
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

		let header = packet::serialize(&Header {
			protocol_version: packet::PROTOCOL_VERSION,
			payload_len: data_length as u16,

			#[cfg(not(feature = "crc"))]
			payload_checksum: 0,

			#[cfg(feature = "crc")]
			payload_checksum: crate::crc::checksum32(&data),
		})?;

		let writer = self.writer.get();

		writer.write_all(&[header, data].concat())?;

		Ok(())
	}
}

unsafe impl<T: Serialize, W: Write> Send for Sender<T, W> {}

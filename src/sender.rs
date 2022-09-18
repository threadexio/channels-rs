use crate::prelude::*;

use crate::packet::{self, Header, PROTOCOL_VERSION};
use crate::shared::*;

/// The sending-half of the channel. This is the same as [`std::sync::mpsc::Sender`](std::sync::mpsc::Sender),
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
pub struct Sender<T: Serialize, W: Write> {
	_p: PhantomData<T>,

	writer: Shared<W>,

	send_buf: Buffer,

	pub crc: crate::crc::Crc,
}

impl<T: Serialize, W: Write> Sender<T, W> {
	pub(crate) fn new(writer: Shared<W>) -> Self {
		Self {
			_p: PhantomData,
			writer,

			send_buf: Buffer::with_size(Header::SIZE),

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

		let mut hdr = Header::new(&mut self.send_buf);

		let mut digest = self.crc.crc16.digest();

		digest.update(hdr.set_protocol_version(PROTOCOL_VERSION));
		digest.update(hdr.set_header_checksum(0));
		digest.update(hdr.set_payload_len(data.len() as u16));

		if cfg!(feature = "crc") {
			digest.update(hdr.set_payload_checksum(self.crc.crc16.checksum(&data)));
		} else {
			digest.update(hdr.set_payload_checksum(0));
		}

		hdr.set_header_checksum(digest.finalize());

		let writer = self.writer.get();

		writer.write_all(&[hdr.get(), &data].concat())?;

		Ok(())
	}
}

unsafe impl<T: Serialize, W: Write> Send for Sender<T, W> {}

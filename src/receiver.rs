use crate::prelude::*;

use crate::io::Buffer;
use crate::shared::*;

use crate::packet::{self, Header};

/// The receiving-half of the channel. This is the same as [`std::sync::mpsc::Receiver`](std::sync::mpsc::Receiver),
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
pub struct Receiver<T: DeserializeOwned, R: Read> {
	reader: Shared<R>,
	_p: PhantomData<T>,

	payload_buf: Buffer,

	header_buf: Buffer,
	header_read: bool,

	pub crc: crate::crc::Crc,
}

impl<T: DeserializeOwned, R: Read> Receiver<T, R> {
	pub(crate) fn new(reader: Shared<R>) -> Self {
		Self {
			_p: PhantomData,
			reader,

			payload_buf: Buffer::with_size(
				packet::MAX_PAYLOAD_SIZE as usize,
			),

			header_buf: Buffer::with_size(Header::SIZE),
			header_read: false,

			crc: Default::default(),
		}
	}

	/// Get a handle to the underlying stream. Directly reading from the stream is not advised.
	pub fn inner(&self) -> &mut R {
		self.reader.get()
	}

	/// Attempts to read an object from the sender end.
	///
	/// If the underlying data stream is a blocking socket then `recv()` will block until
	/// an object is available.
	///
	/// If the underlying data stream is a non-blocking socket then `recv()` will return
	/// an error with a kind of `std::io::ErrorKind::WouldBlock` whenever the complete object is not
	/// available.
	pub fn recv(&mut self) -> Result<T> {
		let reader = self.reader.get();

		if !self.header_read {
			self.header_buf
				.from_reader(&mut *reader, Header::SIZE)?;
			self.header_buf.set_pos(0);

			let mut hdr = Header::new(self.header_buf.inner_mut());

			// verify protocol
			if hdr.get_protocol_version() != packet::PROTOCOL_VERSION
			{
				return Err(Error::VersionMismatch);
			}

			let hdr_checksum = hdr.get_header_checksum();
			hdr.set_header_checksum(0);

			if hdr_checksum != self.crc.crc16.checksum(hdr.get()) {
				return Err(Error::ChecksumError);
			}

			self.header_read = true;
		}

		if self.header_read {
			let hdr = Header::new(self.header_buf.inner_mut());

			let payload_len = hdr.get_payload_len();

			if payload_len > packet::MAX_PAYLOAD_SIZE {
				return Err(Error::SizeLimit);
			}

			self.payload_buf
				.from_reader(&mut *reader, payload_len as usize)?;

			let serialized_data =
				&self.payload_buf.inner()[0..payload_len as usize];

			if cfg!(feature = "crc") {
				if self.crc.crc16.checksum(&serialized_data)
					!= hdr.get_payload_checksum()
				{
					return Err(Error::ChecksumError);
				}
			}

			let data = packet::deserialize(serialized_data)?;

			// reset state
			self.payload_buf.set_pos(0);
			self.header_read = false;

			return Ok(data);
		}

		unreachable!()
	}
}

unsafe impl<T: DeserializeOwned, R: Read> Send for Receiver<T, R> {}

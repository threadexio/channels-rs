use crate::prelude::*;

use crate::io::Buffer;
use crate::shared::*;

use crate::packet::{self, Header};

/// The receiving-half of the channel. This is the same as [`std::sync::mpsc::Receiver`](std::sync::mpsc::Receiver),
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
pub struct Receiver<T: DeserializeOwned, R: Read> {
	_p: PhantomData<T>,

	reader: Shared<R>,

	recv_buf: Buffer,
	msg_header: Option<Header>,

	#[cfg(feature = "crc")]
	pub crc: crate::crc::Crc,
}

impl<T: DeserializeOwned, R: Read> Receiver<T, R> {
	pub(crate) fn new(reader: Shared<R>) -> Self {
		Self {
			_p: PhantomData,
			recv_buf: Buffer::with_size(packet::MAX_PACKET_SIZE as usize),
			msg_header: None,
			reader,

			#[cfg(feature = "crc")]
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

		if self.msg_header.is_none() {
			self.recv_buf.from_reader(&mut *reader, Header::SIZE)?;

			let header: Header = packet::deserialize(&self.recv_buf.buffer()[..Header::SIZE])?;

			// This is here to reset the state in case we don't pass the checks bellow.
			self.recv_buf
				.set_pos(0)
				.expect("set_pos(0) failed. it should not have failed");

			if header.protocol_version != packet::PROTOCOL_VERSION {
				return Err(Error::VersionMismatch);
			}

			self.msg_header = Some(header);
		}

		if let Some(header) = &self.msg_header {
			self.recv_buf
				.from_reader(&mut *reader, header.payload_len as usize)?;

			let serialized_data = &self.recv_buf.buffer()[..header.payload_len as usize];

			#[cfg(feature = "crc")]
			if self.crc.checksum16(&serialized_data) != header.payload_checksum {
				return Err(Error::ChecksumError);
			}

			let data = packet::deserialize(serialized_data)?;

			// reset state
			self.recv_buf.set_pos(0)?;
			self.msg_header = None;

			return Ok(data);
		}

		unreachable!()
	}
}

unsafe impl<T: DeserializeOwned, R: Read> Send for Receiver<T, R> {}

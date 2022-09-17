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
	header_read: bool,

	pub crc: crate::crc::Crc,
}

impl<T: DeserializeOwned, R: Read> Receiver<T, R> {
	pub(crate) fn new(reader: Shared<R>) -> Self {
		Self {
			_p: PhantomData,
			reader,
			recv_buf: Buffer::with_size(packet::MAX_PACKET_SIZE as usize),
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
			self.recv_buf.from_reader(&mut *reader, Header::SIZE)?;
			self.recv_buf.set_pos(0)?;

			#[allow(unused_mut)]
			let mut hdr = Header::new(&mut self.recv_buf[..Header::SIZE]);

			// verify protocol
			if hdr.protocol_version() != packet::PROTOCOL_VERSION {
				return Err(Error::VersionMismatch);
			}

			// verify header
			{
				let hdr_chksum = hdr.header_checksum();
				hdr.set_header_checksum(0);

				if hdr_chksum != self.crc.crc16.checksum(hdr.get()) {
					return Err(Error::ChecksumError);
				}
			}

			self.header_read = true;
			self.recv_buf.set_pos(Header::SIZE)?;
		}

		if self.header_read {
			let mut hdr_buf = self.recv_buf[..Header::SIZE].to_vec();
			let hdr = Header::new(&mut hdr_buf);

			if hdr.payload_len() > packet::MAX_PAYLOAD_SIZE {
				return Err(Error::DataTooLarge);
			}

			let payload_start = self.recv_buf.pos();
			self.recv_buf
				.from_reader(&mut *reader, hdr.payload_len() as usize)?;

			let serialized_data =
				&self.recv_buf[payload_start..(payload_start + hdr.payload_len() as usize)];

			#[cfg(feature = "crc")]
			if self.crc.crc16.checksum(&serialized_data) != hdr.payload_checksum() {
				return Err(Error::ChecksumError);
			}

			let data = packet::deserialize(serialized_data)?;

			// reset state
			self.recv_buf.set_pos(0)?;
			self.header_read = false;

			return Ok(data);
		}

		unreachable!()
	}
}

unsafe impl<T: DeserializeOwned, R: Read> Send for Receiver<T, R> {}

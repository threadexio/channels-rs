use crate::util::{read_offset, write_offset};

use crate::error::*;
use crate::storage::Buffer;

pub struct Packet {
	inner: Buffer,
}

impl Packet {
	pub const PROTOCOL_VERSION: u16 = 0x1;

	pub const MAX_SIZE: u16 = 0xffff; // u16::MAX
	pub const MAX_HEADER_SIZE: u16 = 8;

	pub const MAX_PAYLOAD_SIZE: u16 =
		Self::MAX_SIZE - Self::MAX_HEADER_SIZE;

	pub fn new() -> Self {
		Self { inner: Buffer::new(Self::MAX_SIZE.into()) }
	}

	/// Get a mutable reference to the underlying buffer.
	pub fn buffer(&mut self) -> &mut Buffer {
		&mut self.inner
	}

	/// Get a slice to the entire underlying buffer.
	pub fn packet(&self) -> &[u8] {
		self.inner.buffer()
	}

	/// Get a slice to the entire header.
	pub fn header(&self) -> &[u8] {
		&self.inner.buffer()[..Self::MAX_HEADER_SIZE.into()]
	}

	#[allow(dead_code)]
	/// Get a mutable slice to the entire header.
	fn header_mut(&mut self) -> &mut [u8] {
		&mut self.inner.buffer_mut()[..Self::MAX_HEADER_SIZE.into()]
	}

	/// Get a slice to the entire payload.
	pub fn payload(&self) -> &[u8] {
		&self.inner.buffer()[Self::MAX_HEADER_SIZE.into()..]
	}

	/// Get a mutable slice to the entire payload.
	pub fn payload_mut(&mut self) -> &mut [u8] {
		&mut self.inner.buffer_mut()[Self::MAX_HEADER_SIZE.into()..]
	}
}

impl Packet {
	/// Get the protocol version.
	fn get_version(&self) -> u16 {
		u16::from_be(read_offset(&self.inner, 0))
	}

	/// Set the protocol version.
	fn set_version(&mut self, version: u16) {
		write_offset(&mut self.inner, 0, u16::to_be(version));
	}

	/// Get the packet id.
	pub fn get_id(&self) -> u16 {
		u16::from_be(read_offset(&self.inner, 2))
	}

	/// Set the packet id.
	pub fn set_id(&mut self, id: u16) {
		write_offset(&mut self.inner, 2, u16::to_be(id))
	}

	/// Get the length of the whole packet.
	pub fn get_length(&self) -> u16 {
		u16::from_be(read_offset(&self.inner, 4))
	}

	fn set_length(&mut self, length: u16) {
		write_offset(&mut self.inner, 4, u16::to_be(length));
	}

	/// Get the current checksum as is from the header.
	fn get_header_checksum(&self) -> u16 {
		u16::from_be(read_offset(&self.inner, 6))
	}

	/// Set the current checksum.
	fn set_header_checksum(&mut self, checksum: u16) {
		write_offset(&mut self.inner, 6, u16::to_be(checksum));
	}

	/// Calculate a new checksum.
	fn calculate_header_checksum(&self) -> u16 {
		crate::crc::checksum(self.header())
	}
}

impl Packet {
	/// Get the length of the payload.
	pub fn get_payload_length(&self) -> Result<u16> {
		let packet_len = self.get_length();

		if packet_len < Self::MAX_HEADER_SIZE {
			Err(Error::SizeLimit)
		} else {
			Ok(packet_len - Self::MAX_HEADER_SIZE)
		}
	}

	/// Set the length of the payload.
	pub fn set_payload_length(&mut self, length: u16) -> Result<()> {
		if length > Self::MAX_PAYLOAD_SIZE {
			return Err(Error::SizeLimit);
		} else {
			self.set_length(Self::MAX_HEADER_SIZE + length);
		}

		Ok(())
	}

	pub fn finalize_with<F>(&mut self, f: F) -> &[u8]
	where
		F: FnOnce(&mut Self),
	{
		f(self);

		self.set_version(Self::PROTOCOL_VERSION);

		{
			self.set_header_checksum(0);
			let c = self.calculate_header_checksum();
			self.set_header_checksum(c);
		}

		let packet_len = self.get_length().into();
		&self.packet()[..packet_len]
	}

	pub fn verify_with<F>(&mut self, f: F) -> Result<()>
	where
		F: FnOnce(&mut Self) -> Result<()>,
	{
		if self.get_version() != Self::PROTOCOL_VERSION {
			return Err(Error::VersionMismatch);
		}

		{
			let unverified = self.get_header_checksum();
			self.set_header_checksum(0);
			let calculated = self.calculate_header_checksum();

			if unverified != calculated {
				return Err(Error::ChecksumError);
			}
		}

		// check that the packet length is valid
		// packet_len >= Header Size
		self.get_payload_length()?;

		f(self)?;

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use rand::RngCore;

	use super::*;

	#[test]
	fn correct_endianness() {
		let mut packet = Packet::new();

		packet.set_version(1);
		assert_eq!(packet.get_version(), 1);

		packet.set_id(2);
		assert_eq!(packet.get_id(), 2);

		packet.set_length(3);
		assert_eq!(packet.get_length(), 3);

		packet.set_header_checksum(4);
		assert_eq!(packet.get_header_checksum(), 4);

		assert_eq!(packet.header(), &[0, 1, 0, 2, 0, 3, 0, 4]);
	}

	#[test]
	fn checksum_algorithm() {
		let mut rng = rand::thread_rng();
		let mut packet = Packet::new();

		rng.fill_bytes(packet.header_mut());
		let c1 = packet.calculate_header_checksum();
		let c2 = packet.calculate_header_checksum();
		assert_eq!(c1, c2);

		rng.fill_bytes(packet.header_mut());
		let c3 = packet.calculate_header_checksum();
		assert_ne!(c1, c3);
	}
}

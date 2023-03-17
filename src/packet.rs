use crate::error::*;
use crate::io::Buffer;
use crate::util::{read_offset, write_offset};

pub struct PacketBuffer {
	buffer: Buffer,
}

#[allow(dead_code)]
impl PacketBuffer {
	pub fn new() -> Self {
		Self { buffer: Buffer::new(Self::MAX_PACKET_SIZE) }
	}

	pub fn reset(&mut self) {
		self.buffer.clear();
	}

	pub fn buffer(&self) -> &Buffer {
		&self.buffer
	}

	pub fn buffer_mut(&mut self) -> &mut Buffer {
		&mut self.buffer
	}

	pub fn header(&self) -> &[u8] {
		&self.buffer[..Self::HEADER_SIZE]
	}

	pub fn header_mut(&mut self) -> &mut [u8] {
		&mut self.buffer[..Self::HEADER_SIZE]
	}

	pub fn payload(&self) -> &[u8] {
		&self.buffer[Self::HEADER_SIZE..]
	}

	pub fn payload_mut(&mut self) -> &mut [u8] {
		&mut self.buffer[Self::HEADER_SIZE..]
	}
}

#[allow(dead_code)]
impl PacketBuffer {
	pub const VERSION: u16 = 0x1;
	pub const HEADER_SIZE: usize = 8;
	pub const MAX_PACKET_SIZE: usize = 0xffff;
	pub const MAX_PAYLOAD_SIZE: usize =
		Self::MAX_PACKET_SIZE - Self::HEADER_SIZE;

	pub fn get_version(&self) -> u16 {
		u16::from_be(read_offset::<u16>(&self.buffer, 0))
	}

	pub fn set_version(&mut self, version: u16) {
		write_offset(&mut self.buffer, 0, u16::to_be(version))
	}

	pub fn get_id(&self) -> u16 {
		u16::from_be(read_offset(&self.buffer, 2))
	}

	pub fn set_id(&mut self, id: u16) {
		write_offset(&mut self.buffer, 2, u16::to_be(id))
	}

	pub fn get_length(&self) -> u16 {
		u16::from_be(read_offset(&self.buffer, 4))
	}

	pub fn set_length(&mut self, length: u16) {
		write_offset(&mut self.buffer, 4, u16::to_be(length));
	}

	pub fn get_header_checksum(&self) -> u16 {
		u16::from_be(read_offset(&self.buffer, 6))
	}

	fn set_header_checksum(&mut self, checksum: u16) {
		write_offset(&mut self.buffer, 6, u16::to_be(checksum));
	}

	pub(self) fn calculate_header_checksum(&self) -> u16 {
		crate::crc::checksum(self.header())
	}

	pub fn recalculate_header_checksum(&mut self) {
		self.set_header_checksum(0);
		let c = crate::crc::checksum(self.header());
		self.set_header_checksum(c);
	}
}

impl PacketBuffer {
	pub fn verify(&mut self, seq_no: u16) -> Result<()> {
		if self.get_version() != PacketBuffer::VERSION {
			return Err(Error::VersionMismatch);
		}

		let unverified = self.get_header_checksum();
		self.set_header_checksum(0);
		let calculated = crate::crc::checksum(self.header());

		if unverified != calculated {
			return Err(Error::ChecksumError);
		}

		if (self.get_length() as usize) < PacketBuffer::HEADER_SIZE {
			return Err(Error::SizeLimit);
		}

		if self.get_id() != seq_no {
			return Err(Error::OutOfOrder);
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use rand::RngCore;

	use super::*;

	#[test]
	fn correct_endianness() {
		let mut packet = PacketBuffer::new();

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
		let mut packet = PacketBuffer::new();

		rng.fill_bytes(packet.header_mut());
		let c1 = packet.calculate_header_checksum();
		let c2 = packet.calculate_header_checksum();
		assert_eq!(c1, c2);

		rng.fill_bytes(packet.header_mut());
		let c3 = packet.calculate_header_checksum();
		assert_ne!(c1, c3);
	}
}

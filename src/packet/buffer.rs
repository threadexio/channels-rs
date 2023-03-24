#![allow(dead_code)]

use core::ops::{Deref, DerefMut};

use crate::error::*;
use crate::io::Buffer;
use crate::util::{read_offset, write_offset};

pub struct PacketBuffer {
	buffer: Buffer,
}

impl Deref for PacketBuffer {
	type Target = Buffer;

	fn deref(&self) -> &Self::Target {
		&self.buffer
	}
}

impl DerefMut for PacketBuffer {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.buffer
	}
}

impl PacketBuffer {
	pub fn new() -> Self {
		Self { buffer: Buffer::new(Self::MAX_PACKET_SIZE) }
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

impl PacketBuffer {
	pub const VERSION: u16 = 0x1;
	pub const HEADER_SIZE: usize = 6;
	pub const MAX_PACKET_SIZE: usize = 0xffff;

	pub fn get_version(&self) -> u16 {
		u16::from_be(read_offset::<u16>(&self.buffer, 0))
	}

	pub fn set_version(&mut self, version: u16) {
		write_offset(&mut self.buffer, 0, u16::to_be(version))
	}

	pub fn get_length(&self) -> u16 {
		u16::from_be(read_offset::<u16>(&self.buffer, 2))
	}

	pub fn set_length(&mut self, length: u16) {
		write_offset(&mut self.buffer, 2, u16::to_be(length))
	}

	pub fn get_header_checksum(&self) -> u16 {
		u16::from_be(read_offset::<u16>(&self.buffer, 4))
	}

	pub fn set_header_checksum(&mut self, checksum: u16) {
		write_offset(&mut self.buffer, 4, u16::to_be(checksum))
	}

	pub fn calculate_header_checksum(&mut self) -> u16 {
		self.set_header_checksum(0);
		crate::crc::checksum(self.header())
	}

	pub fn update_header_checksum(&mut self) {
		let c = self.calculate_header_checksum();
		self.set_header_checksum(c);
	}

	pub fn verify_header(&mut self) -> Result<()> {
		if self.get_version() != Self::VERSION {
			return Err(Error::VersionMismatch);
		}

		{
			let unverified = self.get_header_checksum();
			let calculated = self.calculate_header_checksum();

			if unverified != calculated {
				return Err(Error::ChecksumError);
			}
		}

		if (self.get_length() as usize) < Self::HEADER_SIZE {
			return Err(Error::SizeLimit);
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn correct_endianness() {
		let mut packet = PacketBuffer::new();

		packet.set_version(1);
		assert_eq!(packet.get_version(), 1);

		packet.set_length(3);
		assert_eq!(packet.get_length(), 3);

		packet.set_header_checksum(4);
		assert_eq!(packet.get_header_checksum(), 4);

		assert_eq!(packet.header(), &[0, 1, 0, 3, 0, 4]);
	}
}

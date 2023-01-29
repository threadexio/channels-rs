use super::util::*;

pub struct Header<'a>(&'a mut [u8]);

impl<'a> Header<'a> {
	pub const MAX_SIZE: usize = 8;

	pub fn check(buf: &[u8]) -> bool {
		buf.len() >= Self::MAX_SIZE
	}

	pub fn new(buf: &'a mut [u8]) -> Option<Self> {
		if Self::check(buf) {
			Some(Self(buf))
		} else {
			None
		}
	}

	pub fn new_unchecked(buf: &'a mut [u8]) -> Self {
		Self(buf)
	}

	pub fn header(&self) -> &[u8] {
		self.0
	}

	pub fn header_mut(&mut self) -> &mut [u8] {
		self.0
	}

	/// Get the protocol version.
	pub fn get_version(&self) -> u16 {
		u16::from_be(read_offset(self.0, 0))
	}

	/// Set the protocol version.
	pub fn set_version(&mut self, version: u16) {
		write_offset(self.0, 0, u16::to_be(version));
	}

	/// Get the packet id.
	pub fn get_id(&self) -> u16 {
		u16::from_be(read_offset(self.0, 2))
	}

	/// Set the packet id.
	pub fn set_id(&mut self, id: u16) {
		write_offset(self.0, 2, u16::to_be(id))
	}

	/// Get the length of the whole packet.
	pub fn get_length(&self) -> u16 {
		u16::from_be(read_offset(self.0, 4))
	}

	/// Set the length of the whole packet.
	pub fn set_length(&mut self, length: u16) {
		write_offset(self.0, 4, u16::to_be(length));
	}

	/// Get the current checksum as is from the header.
	pub fn get_header_checksum(&self) -> u16 {
		u16::from_be(read_offset(self.0, 6))
	}

	/// Set the current checksum.
	pub fn set_header_checksum(&mut self, checksum: u16) {
		write_offset(self.0, 6, u16::to_be(checksum));
	}

	/// Calculate a new checksum.
	pub fn calculate_header_checksum(&self) -> u16 {
		crate::crc::checksum(self.0)
	}
}

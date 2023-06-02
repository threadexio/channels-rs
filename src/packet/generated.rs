/*
 * Automatically generated from `tools/header.py`. Do not edit!
 *
 * Header spec: spec/header.json
 */
impl PacketBuf {
	unsafe fn unsafe_get_version(&self) -> u16 {
		let x = read_offset(self.as_slice(), 0);
		u16::from_be(x)
	}

	unsafe fn unsafe_set_version(&mut self, value: u16) {
		write_offset(self.as_mut_slice(), 0, u16::to_be(value));
	}

	unsafe fn unsafe_get_packet_length(&self) -> u16 {
		let x = read_offset(self.as_slice(), 2);
		u16::from_be(x)
	}

	unsafe fn unsafe_set_packet_length(&mut self, value: u16) {
		write_offset(self.as_mut_slice(), 2, u16::to_be(value));
	}

	unsafe fn unsafe_get_header_checksum(&self) -> u16 {
		let x = read_offset(self.as_slice(), 4);
		x
	}

	unsafe fn unsafe_set_header_checksum(&mut self, value: u16) {
		write_offset(self.as_mut_slice(), 4, value);
	}

	unsafe fn unsafe_get_flags(&self) -> u8 {
		let x = read_offset(self.as_slice(), 6);
		x
	}

	unsafe fn unsafe_set_flags(&mut self, value: u8) {
		write_offset(self.as_mut_slice(), 6, value);
	}

	unsafe fn unsafe_get_packet_id(&self) -> u8 {
		let x = read_offset(self.as_slice(), 7);
		x
	}

	unsafe fn unsafe_set_packet_id(&mut self, value: u8) {
		write_offset(self.as_mut_slice(), 7, value);
	}

	pub const HEADER_HASH: u16 = 0xfd3f;
	pub const HEADER_SIZE: usize = 8;
}

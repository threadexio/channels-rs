mod util;
pub use util::{deserialize, serialize};

mod header;
pub use header::Header;

pub struct Packet<'a>(&'a mut [u8]);

#[allow(dead_code)]
impl<'a> Packet<'a> {
	pub const MAX_SIZE: usize = 0xffff; // u16::MAX

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

	pub fn packet(&self) -> &[u8] {
		self.0
	}

	pub fn packet_mut(&mut self) -> &mut [u8] {
		self.0
	}

	pub fn header(&mut self) -> Header {
		Header::new_unchecked(&mut self.0[..Header::MAX_SIZE])
	}

	pub fn payload(&self) -> &[u8] {
		&self.0[Header::MAX_SIZE..]
	}

	pub fn payload_mut(&mut self) -> &mut [u8] {
		&mut self.0[Header::MAX_SIZE..]
	}
}

pub const PROTOCOL_VERSION: u16 = 0x1;

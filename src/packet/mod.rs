use core::ops::{Deref, DerefMut};

use crate::error::RecvError;
use crate::io::Cursor;
use crate::util::flags;

flags! {
	#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
	pub struct Flags(u8) {
		const MORE_DATA = 0b_1000_0000;
	}
}

#[derive(
	Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Id(pub u8);

impl Id {
	pub fn next(self) -> Self {
		Self(self.0.wrapping_add(1))
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Checksum(pub u16);

impl Checksum {
	pub fn calculate(buf: &[u8]) -> Self {
		let checksum = unsafe {
			let mut addr = buf.as_ptr().cast::<u16>();
			let mut left = buf.len();
			let mut sum: u32 = 0;

			while left >= 2 {
				sum += u32::from(*addr);
				addr = addr.add(1);
				left -= 2;
			}

			if left == 1 {
				let addr = addr.cast::<u8>();
				sum += u32::from(*addr);
			}

			loop {
				let upper = sum >> 16;
				if upper == 0 {
					break;
				}

				sum = upper + (sum & 0xffff);
			}

			!(sum as u16)
		};

		Self(checksum)
	}

	pub fn verify(buf: &[u8]) -> bool {
		Self::calculate(buf) == Self(0)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
	pub length: u16,
	pub flags: Flags,
	pub id: Id,
}

pub struct Buffer {
	inner: Cursor<Box<[u8]>>,
}

impl Buffer {
	// SAFETY:
	// If there is ever a target with a usize smaller than u16,
	// then this will fail to compile. We can assume for now that
	// usize >= u16, so casting from u16 to usize can be done
	// without any checks.
	const MAX_PACKET_SIZE: usize = 0xffff;

	pub fn new() -> Self {
		Self {
			inner: Cursor::new(
				vec![0u8; Self::MAX_PACKET_SIZE].into_boxed_slice(),
			),
		}
	}
}

impl Buffer {
	pub fn header(&self) -> Header {
		unsafe {
			Header {
				length: self.unsafe_get_packet_length(),
				flags: Flags(self.unsafe_get_flags()),
				id: Id(self.unsafe_get_packet_id()),
			}
		}
	}

	fn header_slice(&self) -> &[u8] {
		&self.as_slice()[..Self::HEADER_SIZE]
	}

	pub fn payload(&self) -> &[u8] {
		&self.as_slice()[Self::HEADER_SIZE..]
	}

	pub fn payload_mut(&mut self) -> &mut [u8] {
		&mut self.as_mut_slice()[Self::HEADER_SIZE..]
	}
}

impl Buffer {
	pub fn finalize(&mut self, h: &Header) {
		unsafe {
			self.unsafe_set_version(Self::HEADER_HASH);

			self.unsafe_set_packet_length(h.length);
			self.unsafe_set_flags(h.flags.0);
			self.unsafe_set_packet_id(h.id.0);

			self.unsafe_set_header_checksum(0);
			let checksum = Checksum::calculate(self.header_slice());
			self.unsafe_set_header_checksum(checksum.0);
		}
	}

	pub fn verify(&self) -> Result<Header, RecvError> {
		unsafe {
			if self.unsafe_get_version() != Self::HEADER_HASH {
				return Err(RecvError::VersionMismatch);
			}

			if !Checksum::verify(self.header_slice()) {
				return Err(RecvError::ChecksumError);
			}

			let header = self.header();

			if (header.length as usize) < Self::HEADER_SIZE {
				return Err(RecvError::InvalidHeader);
			}

			Ok(header)
		}
	}
}

impl Deref for Buffer {
	type Target = Cursor<Box<[u8]>>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl DerefMut for Buffer {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

use crate::mem::*;
include!("generated.rs");

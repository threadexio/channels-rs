use core::ops;

use crate::error::RecvError;
use crate::io;
use crate::mem::{read_offset, write_offset};

use super::types::*;

pub struct PacketBuf {
	inner: io::OwnedBuf,
}

impl ops::Deref for PacketBuf {
	type Target = io::OwnedBuf;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl ops::DerefMut for PacketBuf {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

impl PacketBuf {
	const MAX_PACKET_SIZE: usize = 0xffff; // u16::MAX
	pub const MAX_PAYLOAD_SIZE: usize =
		Self::MAX_PACKET_SIZE - Self::HEADER_SIZE;

	pub fn new() -> Self {
		Self {
			inner: io::OwnedBuf::new(vec![
				0u8;
				Self::MAX_PACKET_SIZE
			]),
		}
	}

	/// Get the entire packet as a raw [`u8`] slice.
	pub fn as_slice(&self) -> &[u8] {
		self.inner.as_slice()
	}

	/// Get the entire packet as a raw [`u8`] slice.
	pub fn as_mut_slice(&mut self) -> &mut [u8] {
		self.inner.as_mut_slice()
	}

	/// Get an [`io::BorrowedBuf`] of the entire header.
	///
	/// Each returned buffer does **NOT** maintain the same
	/// state. This means that each buffer has its own cursor
	/// position.
	pub fn header(&self) -> io::BorrowedBuf {
		io::BorrowedBuf::new(&self.as_slice()[..Self::HEADER_SIZE])
	}

	/// Get an [`io::BorrowedMutBuf`] of the entire header.
	///
	/// Each returned buffer does **NOT** maintain the same
	/// state. This means that each buffer has its own cursor
	/// position.
	pub fn header_mut(&mut self) -> io::BorrowedMutBuf {
		io::BorrowedMutBuf::new(
			&mut self.as_mut_slice()[..Self::HEADER_SIZE],
		)
	}

	/// Get an [`io::BorrowedBuf`] of the entire payload.
	///
	/// Each returned buffer does **NOT** maintain the same
	/// state. This means that each buffer has its own cursor
	/// position.
	pub fn payload(&self) -> io::BorrowedBuf {
		io::BorrowedBuf::new(&self.as_slice()[Self::HEADER_SIZE..])
	}

	/// Get an [`io::BorrowedMutBuf`] of the entire payload.
	///
	/// Each returned buffer does **NOT** maintain the same
	/// state. This means that each buffer has its own cursor
	/// position.
	pub fn payload_mut(&mut self) -> io::BorrowedMutBuf {
		io::BorrowedMutBuf::new(
			&mut self.as_mut_slice()[Self::HEADER_SIZE..],
		)
	}
}

// The header is entirely generated from a custom JSON file.
// See `tools/header.py` and `spec/header.json` for more.
include!("./generated.rs");

impl PacketBuf {
	fn calculate_header_checksum(&self) -> u16 {
		let header = *self.header();
		debug_assert!(
			header.len() & 0b1 == 0,
			"the packet header must have even length"
		);

		#[allow(clippy::as_conversions, clippy::cast_lossless)]
		unsafe {
			let mut addr = header.as_ptr().cast::<u16>();
			let mut left = header.len();
			let mut sum: u32 = 0;

			while left >= 2 {
				sum += (*addr) as u32;
				addr = addr.add(1);
				left -= 2;
			}

			loop {
				let upper = sum >> 16;
				if upper == 0 {
					break;
				}

				sum = upper + (sum & 0xffff);
			}

			!(sum as u16)
		}
	}

	fn update_header_checksum(&mut self) {
		unsafe { self.unsafe_set_header_checksum(0) }
		let new_checksum = self.calculate_header_checksum();
		unsafe { self.unsafe_set_header_checksum(new_checksum) }
	}
}

impl PacketBuf {
	pub fn get_packet_length(&self) -> u16 {
		unsafe { self.unsafe_get_packet_length() }
	}

	pub fn set_packet_length(&mut self, length: u16) {
		unsafe { self.unsafe_set_packet_length(length) }
	}
}

impl PacketBuf {
	pub fn get_id(&self) -> PacketId {
		unsafe { PacketId::from(self.unsafe_get_packet_id()) }
	}

	pub fn set_id(&mut self, id: PacketId) {
		unsafe { self.unsafe_set_packet_id(id.into()) }
	}
}

impl PacketBuf {
	pub fn get_flags(&self) -> PacketFlags {
		unsafe { PacketFlags::new_unchecked(self.unsafe_get_flags()) }
	}

	pub fn set_flags(&mut self, flags: PacketFlags) {
		unsafe { self.unsafe_set_flags(flags.into()) }
	}
}

impl PacketBuf {
	pub fn finalize(&mut self) {
		unsafe { self.unsafe_set_version(Self::HEADER_HASH) }
		self.update_header_checksum();
	}

	pub fn verify_header(&mut self) -> Result<(), RecvError> {
		// check version
		if unsafe { self.unsafe_get_version() } != Self::HEADER_HASH {
			return Err(RecvError::VersionMismatch);
		}

		// verify header checksum
		if self.calculate_header_checksum() != 0 {
			return Err(RecvError::ChecksumError);
		}

		Ok(())
	}
}

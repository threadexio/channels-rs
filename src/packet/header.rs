use crate::error::VerifyError;
use crate::util::flags;

use super::consts::*;

#[derive(
	Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct PacketLength(u16);

impl PacketLength {
	pub fn from_u16(l: u16) -> Option<Self> {
		Self::from_usize(usize::from(l))
	}

	pub fn from_usize(l: usize) -> Option<Self> {
		if (HEADER_SIZE..=MAX_PACKET_SIZE).contains(&l) {
			// SAFETY: HEADER_SIZE <= l <= MAX_PACKET_SIZE
			//     <=> HEADER_SIZE <= l <= u16::MAX
			Some(Self(l as u16))
		} else {
			None
		}
	}

	pub fn as_u16(&self) -> u16 {
		self.0
	}

	pub fn as_usize(&self) -> usize {
		usize::from(self.0)
	}

	pub fn to_payload_length(self) -> PayloadLength {
		// SAFETY: HEADER_SIZE <= self.0 <= MAX_PACKET_SIZE
		//     <=> 0 <= self.0 - HEADER_SIZE <= MAX_PACKET_SIZE - HEADER_SIZE
		//     <=> 0 <= self.0 - HEADER_SIZE <= MAX_PAYLOAD_SIZE
		PayloadLength(self.0 - (HEADER_SIZE as u16))
	}
}

#[derive(
	Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct PayloadLength(u16);

impl PayloadLength {
	pub fn from_u16(l: u16) -> Option<Self> {
		Self::from_usize(usize::from(l))
	}

	pub fn from_usize(l: usize) -> Option<Self> {
		if l <= MAX_PAYLOAD_SIZE {
			// SAFETY: MAX_PAYLOAD_SIZE <= u16::MAX
			Some(Self(l as u16))
		} else {
			None
		}
	}

	pub fn as_u16(&self) -> u16 {
		self.0
	}

	pub fn as_usize(&self) -> usize {
		usize::from(self.0)
	}

	pub fn to_packet_length(self) -> PacketLength {
		// SAFETY: self.0 <= MAX_PAYLOAD_SIZE
		//     <=> self.0 <= MAX_PACKET_SIZE - HEADER_SIZE
		//     <=> HEADER_SIZE + self.0 <= MAX_PACKET_SIZE
		//     <=> HEADER_SIZE + self.0 <= u16::MAX
		PacketLength((HEADER_SIZE as u16) + self.0)
	}
}

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
	pub length: PacketLength,
	pub flags: Flags,
	pub id: Id,
}

impl Header {
	pub const SIZE: usize = private::HEADER_SIZE;

	/// Write the header to `buf`.
	///
	/// This function also calculates the checksum for the header.
	///
	/// # Panics
	///
	/// If `buf.len() < HEADER_SIZE`.
	pub fn write_to(&self, buf: &mut [u8]) {
		assert!(
			buf.len() >= Self::SIZE,
			"packet header buf must be >= {}",
			Self::SIZE
		);

		let buf = &mut buf[..Self::SIZE];

		unsafe {
			private::unsafe_set_version(buf, private::HEADER_HASH);
			private::unsafe_set_packet_length(buf, self.length.0);
			private::unsafe_set_flags(buf, self.flags.0);
			private::unsafe_set_packet_id(buf, self.id.0);

			private::unsafe_set_header_checksum(buf, 0);
			let checksum = Checksum::calculate(buf);
			private::unsafe_set_header_checksum(buf, checksum.0);
		}
	}

	/// Read the header from `buf` without performing any checks.
	///
	/// # Panics
	///
	/// If `buf.len() < HEADER_SIZE`.
	pub unsafe fn read_from_unchecked(buf: &[u8]) -> Self {
		assert!(
			buf.len() >= Self::SIZE,
			"packet header buf must be >= {}",
			Self::SIZE
		);

		let buf = &buf[..Self::SIZE];

		unsafe {
			Self {
				length: PacketLength(
					private::unsafe_get_packet_length(buf),
				),
				flags: Flags(private::unsafe_get_flags(buf)),
				id: Id(private::unsafe_get_packet_id(buf)),
			}
		}
	}

	/// Read the header from `buf`.
	///
	/// **NOTE:** This method does not verify the `id` field.
	///
	/// # Panics
	///
	/// If `buf.len() < HEADER_SIZE`.
	pub fn read_from(buf: &[u8]) -> Result<Header, VerifyError> {
		assert!(
			buf.len() >= Self::SIZE,
			"packet header buf must be >= {}",
			Self::SIZE
		);

		let buf = &buf[..Self::SIZE];

		unsafe {
			if private::unsafe_get_version(buf)
				!= private::HEADER_HASH
			{
				return Err(VerifyError::VersionMismatch);
			}

			if Checksum::calculate(buf).0 != 0 {
				return Err(VerifyError::VersionMismatch);
			}

			if usize::from(private::unsafe_get_packet_length(buf))
				< Self::SIZE
			{
				return Err(VerifyError::InvalidHeader);
			}
		}

		Ok(unsafe { Self::read_from_unchecked(buf) })
	}
}

#[allow(clippy::all)]
mod private {
	use crate::mem::*;
	include!("generated.rs");
}

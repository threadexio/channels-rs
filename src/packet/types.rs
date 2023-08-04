use super::consts::*;
use crate::util::flags;

/// The length of one packet.
///
/// The following holds true for this type:
///
/// - `HEADER_SIZE <= l <= MAX_PACKET_SIZE`
#[derive(
	Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct PacketLength(u16);

impl PacketLength {
	/// Create a new payload length from `l`.
	pub fn from_u16(l: u16) -> Option<Self> {
		Self::from_usize(usize::from(l))
	}

	/// Create a new payload length from `l`.
	///
	/// `l` must be in the range `HEADER_SIZE..=MAX_PACKET_SIZE`.
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

/// The length of the payload inside one packet.
///
/// The following holds true for this type:
///
/// - `0 <= l <= MAX_PAYLOAD_SIZE`
#[derive(
	Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct PayloadLength(u16);

impl PayloadLength {
	/// Create a new payload length from `l`.
	///
	/// `l` must be `<= MAX_PAYLOAD_SIZE`.
	pub fn from_u16(l: u16) -> Option<Self> {
		Self::from_usize(usize::from(l))
	}

	/// Create a new payload length from `l`.
	///
	/// `l` must be `<= MAX_PAYLOAD_SIZE`.
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

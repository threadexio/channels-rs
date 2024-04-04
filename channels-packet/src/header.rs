//! A safe API to work with packet headers.

use core::fmt;

use crate::{
	checksum::{checksum, verify},
	consts::{HEADER_SIZE_USIZE, PROTOCOL_VERSION},
	flags::Flags,
	id::{Id, IdSequence},
	num::{u16be, PacketLength},
	raw::{RawHeader, RawHeaderInner},
};

/// Packet header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
	/// Header length field.
	pub length: PacketLength,
	/// Header flags field.
	pub flags: Flags,
	/// Packet ID field.
	pub id: Id,
}

impl Header {
	/// The size of the header in bytes.
	///
	/// This is not the same as [`core::mem::size_of`].
	pub const SIZE: usize = HEADER_SIZE_USIZE;
}

/// Possible errors while reading a header.
///
/// This is the error type returned by [`Header::try_from_bytes`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerifyError {
	/// The `version` field of the header is not supported.
	VersionMismatch,
	/// The `checksum` field of the header is invalid.
	InvalidChecksum,
	/// The `length` field of the header is invalid.
	InvalidLength,
	/// The `id` field is not equal to the next expected ID.
	OutOfOrder,
}

impl fmt::Display for VerifyError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::VersionMismatch => f.write_str("version mismatch"),
			Self::InvalidChecksum => f.write_str("invalid checksum"),
			Self::InvalidLength => f.write_str("bad packet length"),
			Self::OutOfOrder => f.write_str("out of order"),
		}
	}
}

/// Specify whether to calculate/verify the header checksum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WithChecksum {
	/// Calculate/Verify the checksum.
	Yes,
	/// Don't calculate/verify the checksum.
	No,
}

/// Specify whether to verify the ID field of the header.
#[derive(Debug)]
pub enum VerifyId<'a> {
	/// Verify the ID.
	Yes(&'a mut IdSequence),
	/// Don't verify the ID.
	No,
}

impl Header {
	/// Read a header from `bytes`.
	///
	/// **Note:** Use [`slice_to_array{_mut}`] if you have a slice.
	///
	/// You can use [`slice_to_array`] to convert a slice to a reference to
	/// an array.
	///
	/// [`slice_to_array`]: crate::util::slice_to_array
	pub fn try_from_bytes(
		bytes: [u8; Header::SIZE],
		with_checksum: WithChecksum,
		verify_id: VerifyId,
	) -> Result<Header, VerifyError> {
		use VerifyError as E;

		let raw = RawHeader { bytes };

		let version: u16 = unsafe { raw.header.version.into() };

		if version != PROTOCOL_VERSION {
			return Err(E::VersionMismatch);
		}

		match with_checksum {
			WithChecksum::No => {},
			WithChecksum::Yes => {
				if !verify(unsafe { &raw.bytes }) {
					return Err(E::InvalidChecksum);
				}
			},
		}

		let length =
			PacketLength::new(unsafe { raw.header.length.into() })
				.ok_or(E::InvalidLength)?;

		let flags =
			Flags::from_bits_retain(unsafe { raw.header.flags });

		let id = match (verify_id, Id::from(unsafe { raw.header.id }))
		{
			(VerifyId::No, id) => id,
			(VerifyId::Yes(seq), id) => {
				if id == seq.peek() {
					let _ = seq.advance();
					id
				} else {
					return Err(E::OutOfOrder);
				}
			},
		};

		Ok(Header { length, flags, id })
	}

	/// Convert the header to its bytes.
	#[must_use]
	pub fn to_bytes(
		&self,
		with_checksum: WithChecksum,
	) -> [u8; Self::SIZE] {
		let mut raw = RawHeader {
			header: RawHeaderInner {
				version: u16be::from(PROTOCOL_VERSION),
				length: u16be::from(self.length.as_u16()),
				checksum: 0.into(),
				flags: self.flags.bits(),
				id: self.id.as_u8(),
			},
		};

		if with_checksum == WithChecksum::Yes {
			unsafe {
				let checksum = checksum(&raw.bytes);
				raw.header.checksum = checksum.into();
			}
		}

		unsafe { raw.bytes }
	}
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
	use super::*;

	#[test]
	fn test_header_write() {
		assert_eq!(
			Header {
				length: PacketLength::new(1234).unwrap(),
				flags: Flags::empty(),
				id: Id::from(42),
			}
			.to_bytes(WithChecksum::Yes),
			[0xfd, 0x3f, 0x04, 0xd2, 0xfd, 0xc3, 0x00, 0x2a]
		);

		assert_eq!(
			Header {
				length: PacketLength::new(42).unwrap(),
				flags: Flags::MORE_DATA,
				id: Id::from(0),
			}
			.to_bytes(WithChecksum::Yes),
			[0xfd, 0x3f, 0x00, 0x2a, 0x82, 0x95, 0x80, 0x00]
		);
	}

	#[test]
	fn test_header_read() {
		let mut seq = IdSequence::new();

		assert_eq!(
			Header::try_from_bytes(
				[0xfd, 0x3f, 0x04, 0xd2, 0xfd, 0xed, 0x00, 0x00],
				WithChecksum::Yes,
				VerifyId::Yes(&mut seq)
			),
			Ok(Header {
				length: PacketLength::new(1234).unwrap(),
				flags: Flags::empty(),
				id: Id::from(0),
			})
		);

		assert_eq!(
			Header::try_from_bytes(
				[0xfd, 0x3f, 0x00, 0x2a, 0x82, 0x94, 0x80, 0x01],
				WithChecksum::Yes,
				VerifyId::Yes(&mut seq)
			),
			Ok(Header {
				length: PacketLength::new(42).unwrap(),
				flags: Flags::MORE_DATA,
				id: Id::from(1),
			})
		);
	}

	#[test]
	fn test_header_read_no_checksum() {
		let mut seq = IdSequence::new();

		assert_eq!(
			Header::try_from_bytes(
				[0xfd, 0x3f, 0x04, 0xd2, 0xff, 0xff, 0x00, 0x00],
				WithChecksum::No,
				VerifyId::Yes(&mut seq)
			),
			Ok(Header {
				length: PacketLength::new(1234).unwrap(),
				flags: Flags::empty(),
				id: Id::from(0),
			})
		);

		assert_eq!(
			Header::try_from_bytes(
				[0xfd, 0x3f, 0x00, 0x2a, 0xff, 0xff, 0x80, 0x01],
				WithChecksum::No,
				VerifyId::Yes(&mut seq)
			),
			Ok(Header {
				length: PacketLength::new(42).unwrap(),
				flags: Flags::MORE_DATA,
				id: Id::from(1),
			})
		);
	}

	#[test]
	fn test_header_read_no_id() {
		assert_eq!(
			Header::try_from_bytes(
				[0xfd, 0x3f, 0x04, 0xd2, 0xfd, 0xed, 0x00, 0x00],
				WithChecksum::Yes,
				VerifyId::No
			),
			Ok(Header {
				length: PacketLength::new(1234).unwrap(),
				flags: Flags::empty(),
				id: Id::from(0),
			})
		);

		assert_eq!(
			Header::try_from_bytes(
				[0xfd, 0x3f, 0x00, 0x2a, 0x82, 0x94, 0x80, 0x01],
				WithChecksum::Yes,
				VerifyId::No
			),
			Ok(Header {
				length: PacketLength::new(42).unwrap(),
				flags: Flags::MORE_DATA,
				id: Id::from(1),
			})
		);
	}

	#[test]
	fn test_header_read_bad_version() {
		let mut seq = IdSequence::new();

		assert_eq!(
			Header::try_from_bytes(
				[0xfe, 0x3f, 0x04, 0xd2, 0xfd, 0xed, 0x00, 0x00],
				WithChecksum::Yes,
				VerifyId::Yes(&mut seq)
			),
			Err(VerifyError::VersionMismatch)
		);
	}

	#[test]
	fn test_header_read_bad_checksum() {
		let mut seq = IdSequence::new();

		assert_eq!(
			Header::try_from_bytes(
				[0xfd, 0x3f, 0x04, 0xd2, 0xff, 0xed, 0x00, 0x00],
				WithChecksum::Yes,
				VerifyId::Yes(&mut seq)
			),
			Err(VerifyError::InvalidChecksum)
		);
	}

	#[test]
	fn test_header_read_bad_length() {
		let mut seq = IdSequence::new();

		assert_eq!(
			Header::try_from_bytes(
				[0xfd, 0x3f, 0x00, 0x03, 0xfd, 0xed, 0x00, 0x00],
				WithChecksum::No,
				VerifyId::Yes(&mut seq)
			),
			Err(VerifyError::InvalidLength)
		);
	}

	#[test]
	fn test_header_read_bad_id() {
		let mut seq = IdSequence::new();

		assert_eq!(
			Header::try_from_bytes(
				[0xfd, 0x3f, 0x04, 0xd2, 0xff, 0xff, 0x00, 0x04],
				WithChecksum::No,
				VerifyId::Yes(&mut seq)
			),
			Err(VerifyError::OutOfOrder)
		);

		assert_eq!(seq.peek().as_u8(), 0);
	}
}

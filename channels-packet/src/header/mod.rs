//! TODO: docs

use core::borrow::Borrow;
use core::fmt;
use core::ops::{Deref, Range};

use crate::num::{u2, u6};

mod checksum;
mod seq;

pub use self::checksum::Checksum;
pub use self::seq::FrameNumSequence;

const VERSION_MASK: u64 = 0x0000_0000_0000_00ff;
const VERSION_SHIFT: u32 = 0;

const FLAGS_MASK: u64 = 0x0000_0000_0000_0300;
const FLAGS_SHIFT: u32 = 8;

const FRAME_NUM_MASK: u64 = 0x0000_0000_0000_fc00;
const FRAME_NUM_SHIFT: u32 = 10;

const CHECKSUM_FIELD: Range<usize> = 2..4;

const DATA_LEN_MASK: u64 = 0xffff_ffff_0000_0000;
const DATA_LEN_SHIFT: u32 = 32;

/// TODO: docs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
	/// TODO: docs
	pub frame_num: u6,
	/// TODO: docs
	pub data_len: u32,
}

impl Header {
	const VERSION: u8 = 0x42;

	/// TODO: docs
	pub const SIZE: usize = 8;

	/// TODO: docs
	#[inline]
	pub const fn builder() -> Builder {
		Builder { data_len: 0, frame_num: u6::new_truncate(0) }
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	#[allow(clippy::cast_lossless)]
	pub fn to_bytes(self) -> HeaderBytes {
		let x = ((Header::VERSION as u64) << VERSION_SHIFT) // version
			| ((0u8 as u64) << FLAGS_SHIFT) // flags
			| ((self.frame_num.get() as u64) << FRAME_NUM_SHIFT) // frame_num
			| ((self.data_len as u64) << DATA_LEN_SHIFT); // data_len

		let mut data = u64::to_le_bytes(x);

		let checksum = Checksum::checksum(&data);
		data[CHECKSUM_FIELD].copy_from_slice(&checksum.to_ne_bytes());

		HeaderBytes { data }
	}
}

/// TODO: docs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeaderError {
	/// TODO: docs
	InvalidChecksum,
	/// TODO: docs
	VersionMismatch,
}

impl fmt::Display for HeaderError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match *self {
			Self::InvalidChecksum => f.write_str("invalid checksum"),
			Self::VersionMismatch => f.write_str("version mismatch"),
		}
	}
}

#[cfg(feature = "std")]
impl std::error::Error for HeaderError {}

impl Header {
	/// TODO: docs
	#[must_use = "unused parse result"]
	#[allow(
		clippy::missing_panics_doc,
		clippy::cast_possible_truncation
	)]
	pub fn try_parse(
		bytes: &[u8],
	) -> Result<Option<Self>, HeaderError> {
		let Some(hdr_bytes) = bytes.get(..Self::SIZE) else {
			return Ok(None);
		};

		let hdr_bytes: &[u8; Self::SIZE] = hdr_bytes
			.try_into()
			.expect("cast header slice to array failed");

		let hdr = u64::from_le_bytes(*hdr_bytes);

		let version = ((hdr & VERSION_MASK) >> VERSION_SHIFT) as u8;

		let _flags = u2::new_truncate(
			((hdr & FLAGS_MASK) >> FLAGS_SHIFT) as u8,
		);

		let frame_num = u6::new_truncate(
			((hdr & FRAME_NUM_MASK) >> FRAME_NUM_SHIFT) as u8,
		);

		let data_len = u32::from_le(
			((hdr & DATA_LEN_MASK) >> DATA_LEN_SHIFT) as u32,
		);

		if version != Self::VERSION {
			return Err(HeaderError::VersionMismatch);
		}

		if Checksum::checksum(hdr_bytes) != 0 {
			return Err(HeaderError::InvalidChecksum);
		}

		Ok(Some(Self { frame_num, data_len }))
	}
}

/// TODO: docs
#[allow(missing_debug_implementations)]
#[must_use = "builders don't do anything unless you build them"]
pub struct Builder {
	frame_num: u6,
	data_len: u32,
}

impl Builder {
	/// TODO: docs
	#[inline]
	pub const fn frame_num(mut self, frame_num: u6) -> Self {
		self.frame_num = frame_num;
		self
	}

	/// TODO: docs
	#[inline]
	pub fn frame_num_from_seq(
		self,
		seq: &mut FrameNumSequence,
	) -> Self {
		self.frame_num(seq.advance())
	}

	/// TODO: docs
	#[inline]
	pub const fn data_len(mut self, data_len: u32) -> Self {
		self.data_len = data_len;
		self
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn data_len_from_slice(self, data: &[u8]) -> Option<Self> {
		data.len()
			.try_into()
			.map(|data_len| self.data_len(data_len))
			.ok()
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub const fn build(self) -> Header {
		let Self { frame_num, data_len } = self;
		Header { frame_num, data_len }
	}
}

/// TODO: docs
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct HeaderBytes {
	data: [u8; Header::SIZE],
}

impl HeaderBytes {
	const fn as_slice(&self) -> &[u8] {
		self.data.as_slice()
	}
}

impl Borrow<[u8]> for HeaderBytes {
	#[inline]
	fn borrow(&self) -> &[u8] {
		self.as_slice()
	}
}

impl AsRef<[u8]> for HeaderBytes {
	#[inline]
	fn as_ref(&self) -> &[u8] {
		self.as_slice()
	}
}

impl Deref for HeaderBytes {
	type Target = [u8];

	#[inline]
	fn deref(&self) -> &Self::Target {
		self.as_slice()
	}
}

impl fmt::Debug for HeaderBytes {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_list().entries(self.as_slice()).finish()
	}
}

#[cfg(test)]
#[allow(clippy::unusual_byte_groupings)]
mod tests {
	use super::*;

	struct Vector {
		header: Header,
		bytes: [u8; Header::SIZE],
	}

	#[rustfmt::skip]
    static TEST_VECTORS: &[Vector] = &[
        Vector {
            header: Header {
                data_len: 0,
                frame_num: u6::new_truncate(32),
            },
            bytes: [0x42, 0b100000_00, 0xbd, 0x7f, 0, 0, 0, 0],
        },
        Vector {
            header: Header {
                data_len: 42,
                frame_num: u6::new_truncate(34),
            },
            bytes: [0x42, 0b100010_00, 0x93, 0x77, 42, 0, 0, 0],
        },
        Vector {
            header: Header {
                data_len: 0xffff,
                frame_num: u6::new_truncate(23),
            },
            bytes: [0x42, 0b010111_00, 0xbd, 0xa3, 0xff, 0xff, 0, 0],
        },
        Vector {
            header: Header {
                data_len: 0x0001_0000,
                frame_num: u6::new_truncate(5),
            },
            bytes: [0x42, 0b000101_00, 0xbc, 0xeb, 0x00, 0x00, 0x01, 0x00],
        },
        Vector {
            header: Header {
                data_len: 0xffff_ffff,
                frame_num: u6::new_truncate(14),
            },
            bytes: [0x42, 0b001110_00, 0xbd, 0xc7, 0xff, 0xff, 0xff, 0xff],
        },
    ];

	macro_rules! tests {
        ($($test_vector_idx:expr => $test_encode:ident, $test_decode:ident,)*) => {
            $(
                #[test]
                fn $test_encode() {
                    let Vector { header, bytes } = TEST_VECTORS[$test_vector_idx];
                    let buf = header.to_bytes();
                    assert_eq!(buf.as_ref(), bytes);
                }

                #[test]
                fn $test_decode() {
                    let Vector { header, bytes } = TEST_VECTORS[$test_vector_idx];
                    let x = Header::try_parse(&bytes).unwrap().unwrap();
                    assert_eq!(header, x);
                }
            )*
        };
    }

	tests! {
		0 => test_header_encode_no_payload,           test_header_decode_no_payload,
		1 => test_header_encode_small_payload,        test_header_decode_small_payload,
		2 => test_header_encode_small_medium_payload, test_header_decode_small_medium_payload,
		3 => test_header_encode_medium_payload,       test_header_decode_medium_payload,
		4 => test_header_encode_medium_large_payload, test_header_decode_medium_large_payload,
	}

	#[test]
	fn test_header_decode_invalid_version() {
		let bytes: &[u8] =
			&[0x43, 0b000000_00, 0x00, 0x00, 0, 0, 0, 0];
		assert_eq!(
			Header::try_parse(bytes),
			Err(HeaderError::VersionMismatch)
		);
	}

	#[test]
	fn test_header_decode_invalid_checksum() {
		let bytes: &[u8] =
			&[0x42, 0b000100_01, 0xCC, 0xCC, 0x23, 0x00, 0x00, 0x00];
		assert_eq!(
			Header::try_parse(bytes),
			Err(HeaderError::InvalidChecksum)
		);
	}

	#[test]
	fn test_header_decode_not_enough() {
		const HEADERS: &[&[u8]] = &[
			&[],
			&[0x42],
			&[0x42, 0b010101_01],
			&[0x42, 0b010101_01, 0xCC],
			&[0x42, 0b010101_01, 0xCC, 0xCC],
			&[0x42, 0b010101_01, 0xCC, 0xCC, 0x00],
			&[0x42, 0b010101_11, 0xCC, 0xCC, 0x00, 0x00],
			&[0x42, 0b010101_11, 0xCC, 0xCC, 0x00, 0x00, 0x00],
		];

		HEADERS.iter().copied().for_each(|bytes| {
			assert_eq!(Header::try_parse(bytes), Ok(None), "fail");
		});
	}
}

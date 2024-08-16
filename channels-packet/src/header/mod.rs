//! TODO: docs

use core::borrow::Borrow;
use core::fmt;
use core::ops::Deref;
use core::slice;

use crate::num::{u2, u48, u6};

mod checksum;
mod seq;

pub use self::checksum::Checksum;
pub use self::seq::FrameNumSequence;

const VERSION: u8 = 0x42;
const MAX_HEADER_LEN: usize = 10;

/// TODO: docs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
	/// TODO: docs
	pub frame_num: u6,
	/// TODO: docs
	pub data_len: u48,
}

impl Header {
	/// TODO: docs
	#[inline]
	pub const fn builder() -> Builder {
		Builder {
			data_len: u48::new_truncate(0),
			frame_num: u6::new_truncate(0),
		}
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn to_bytes(self) -> HeaderBytes {
		HeaderBytes::from(self)
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn length(&self) -> usize {
		4_usize.saturating_add(
			(words_needed(self.data_len).get() * 2) as usize,
		)
	}
}

/// TODO: docs
#[allow(missing_debug_implementations)]
#[must_use = "builders don't do anything unless you build them"]
pub struct Builder {
	frame_num: u6,
	data_len: u48,
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
	pub const fn data_len(mut self, data_len: u48) -> Self {
		self.data_len = data_len;
		self
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn data_len_from_slice(self, data: &[u8]) -> Option<Self> {
		u48::new(data.len() as u64)
			.map(|data_len| self.data_len(data_len))
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeaderError {
	/// TODO: docs
	InvalidChecksum,
	/// TODO: docs
	NotEnough,
	/// TODO: docs
	VersionMismatch,
}

impl fmt::Display for HeaderError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match *self {
			Self::InvalidChecksum => f.write_str("invalid checksum"),
			Self::NotEnough => f.write_str("not enough data"),
			Self::VersionMismatch => f.write_str("version mismatch"),
		}
	}
}

#[cfg(feature = "std")]
impl std::error::Error for HeaderError {}

impl TryFrom<&[u8]> for Header {
	type Error = HeaderError;

	fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
		if bytes.len() < 4 {
			return Err(HeaderError::NotEnough);
		}

		let version = bytes[0];
		if version != VERSION {
			return Err(HeaderError::VersionMismatch);
		}

		let octet1 = bytes[1];
		let frame_num = u6::new_truncate(octet1 >> 2);
		let len_words = u2::new_truncate(octet1);

		let len_bytes = (len_words.get() * 2) as usize;
		let header_len = 4 + len_bytes;

		if bytes.len() < header_len {
			return Err(HeaderError::NotEnough);
		}

		if Checksum::checksum(&bytes[..header_len]) != 0 {
			return Err(HeaderError::InvalidChecksum);
		}

		let data_len = read_u48_from_slice(&bytes[4..4 + len_bytes]);

		Ok(Self { frame_num, data_len })
	}
}

/// TODO: docs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HeaderBytes {
	data: [u8; MAX_HEADER_LEN],
	len: u8,
}

impl From<Header> for HeaderBytes {
	#[allow(clippy::cast_possible_truncation)]
	fn from(hdr: Header) -> Self {
		let mut data = [0u8; MAX_HEADER_LEN];

		let len_words = words_needed(hdr.data_len);
		let len_bytes = (len_words.get() * 2) as usize;
		let header_len = 4 + len_bytes;

		data[0] = VERSION;
		data[1] = (hdr.frame_num.get() << 2) | len_words.get();

		write_u48_to_slice(hdr.data_len, &mut data[4..10]);

		unsafe {
			let checksum = Checksum::checksum(&data[..header_len]);
			data.as_mut_ptr()
				.byte_add(2)
				.cast::<u16>()
				.write_unaligned(checksum);
		}

		// TODO: safety
		Self { data, len: header_len as u8 }
	}
}

impl HeaderBytes {
	const fn as_slice(&self) -> &[u8] {
		let ptr = self.data.as_ptr();
		unsafe { slice::from_raw_parts(ptr, self.len as usize) }
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

/// Read a variable sized `u48` value from `bytes` in little-endian order.
fn read_u48_from_slice(bytes: &[u8]) -> u48 {
	assert!(
		bytes.len() <= 6,
		"data_len must not be larger than 6 bytes"
	);

	let mut x = [0u8; 8];
	x[..bytes.len()].copy_from_slice(bytes);
	let x = u64::from_le_bytes(x);

	u48::new(x).expect("data_len should fit inside a u48")
}

// Write `x` to `out` as a variable sized `u48` in little-endian order.
fn write_u48_to_slice(x: u48, out: &mut [u8]) {
	assert!(out.len() >= 6, "out must not be smaller than 6 bytes");

	let x = u64::to_le_bytes(x.get());
	out[..6].copy_from_slice(&x[..6]);
}

const fn words_needed(len: u48) -> u2 {
	let mut len = len.get();
	let mut x = 0;

	while len != 0 {
		x += 1;
		len >>= 16;
	}

	u2::new_truncate(x)
}

#[cfg(test)]
#[allow(clippy::unusual_byte_groupings)]
mod tests {
	use super::*;

	struct Vector {
		header: Header,
		bytes: &'static [u8],
	}

	#[rustfmt::skip]
    static TEST_VECTORS: &[Vector] = &[
        Vector {
            header: Header {
                data_len: u48::new_truncate(0),
                frame_num: u6::new_truncate(32),
            },
            bytes: &[0x42, 0b100000_00, 0xbd, 0x7f],
        },
        Vector {
            header: Header {
                data_len: u48::new_truncate(42),
                frame_num: u6::new_truncate(34),
            },
            bytes: &[0x42, 0b100010_01, 0x93, 0x76, 42, 00],
        },
        Vector {
            header: Header {
                data_len: u48::new_truncate(0xffff),
                frame_num: u6::new_truncate(23),
            },
            bytes: &[0x42, 0b010111_01, 0xbd, 0xa2, 0xff, 0xff],
        },
        Vector {
            header: Header {
                data_len: u48::new_truncate(0x0001_0000),
                frame_num: u6::new_truncate(5),
            },
            bytes: &[0x42, 0b000101_10, 0xbc, 0xe9, 0x00, 0x00, 0x01, 0x00],
        },
        Vector {
            header: Header {
                data_len: u48::new_truncate(0xffff_ffff),
                frame_num: u6::new_truncate(14),
            },
            bytes: &[0x42, 0b001110_10, 0xbd, 0xc5, 0xff, 0xff, 0xff, 0xff],
        },
        Vector {
            header: Header {
                data_len: u48::new_truncate(0x0001_0000_0000),
                frame_num: u6::new_truncate(0),
            },
            bytes: &[0x42, 0b000000_11, 0xbc, 0xfc, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00],
        },
        Vector {
            header: Header {
                data_len: u48::new_truncate(0xffff_ffff_ffff),
                frame_num: u6::new_truncate(27),
            },
            bytes: &[0x42, 0b011011_11, 0xbd, 0x90, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
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
                    let x = Header::try_from(bytes).unwrap();
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
		5 => test_header_encode_large_payload,        test_header_decode_large_payload,
		6 => test_header_encode_largest_payload,      test_header_decode_largest_payload,
	}

	#[test]
	fn test_header_decode_invalid_version() {
		let bytes: &[u8] = &[0x43, 0b000000_00, 0x00, 0x00];
		assert_eq!(
			Header::try_from(bytes),
			Err(HeaderError::VersionMismatch)
		);
	}

	#[test]
	fn test_header_decode_invalid_checksum() {
		let bytes: &[u8] =
			&[0x42, 0b000100_01, 0xCC, 0xCC, 0x23, 0x00];
		assert_eq!(
			Header::try_from(bytes),
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
			&[
				0x42,
				0b010101_11,
				0xCC,
				0xCC,
				0x00,
				0x00,
				0x00,
				0x00,
				0x00,
			],
		];

		HEADERS.iter().copied().for_each(|bytes| {
			assert_eq!(
				Header::try_from(bytes),
				Err(HeaderError::NotEnough),
				"fail"
			);
		});
	}
}

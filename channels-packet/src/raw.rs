//! Raw types to parse packet headers.

use core::fmt;

use crate::{consts::HEADER_SIZE_USIZE, util::static_assert};

pub use crate::num::u16be;

/// A raw header.
#[repr(C, align(8))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawHeaderInner {
	/// Version field.
	pub version: u16be,
	/// Length field.
	pub length: u16be,
	/// Checksum field.
	pub checksum: u16be,
	/// Flags field.
	pub flags: u8,
	/// Id field.
	pub id: u8,
}

/// A raw header whose raw bytes can be accessed as an array.
#[repr(C, align(8))]
#[derive(Clone, Copy)]
pub union RawHeader {
	/// The header.
	pub header: RawHeaderInner,
	/// The raw bytes of the header.
	pub bytes: [u8; HEADER_SIZE_USIZE],
}

impl fmt::Debug for RawHeader {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("RawHeader")
			.field("header", unsafe { &self.header })
			.field("bytes", unsafe { &self.bytes })
			.finish()
	}
}

static_assert!(
	core::mem::size_of::<RawHeader>() == HEADER_SIZE_USIZE
);

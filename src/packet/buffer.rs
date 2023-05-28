use core::mem;
use core::ops;

use crate::error::*;
use crate::io;
use crate::util::{read_offset, write_offset};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PacketId(u8);

impl PacketId {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn next_id(&mut self) -> &mut Self {
		self.0 = self.0.wrapping_add(1);
		self
	}
}

impl From<u8> for PacketId {
	fn from(value: u8) -> Self {
		Self(value)
	}
}

impl From<PacketId> for u8 {
	fn from(value: PacketId) -> Self {
		value.0
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PacketFlags(u8);

impl PacketFlags {
	pub const MORE_DATA: Self = Self(0b_1000_0000);

	pub fn zero() -> Self {
		Self(0)
	}

	pub unsafe fn new_unchecked(f: u8) -> Self {
		Self(f)
	}
}

impl From<PacketFlags> for u8 {
	fn from(value: PacketFlags) -> Self {
		value.0
	}
}

impl ops::BitAnd for PacketFlags {
	type Output = bool;

	fn bitand(self, rhs: Self) -> Self::Output {
		self.0 & rhs.0 != 0
	}
}

impl ops::BitOr for PacketFlags {
	type Output = Self;

	fn bitor(self, rhs: Self) -> Self::Output {
		Self(self.0 | rhs.0)
	}
}

impl ops::BitOrAssign for PacketFlags {
	fn bitor_assign(&mut self, rhs: Self) {
		self.0 |= rhs.0;
	}
}

impl ops::BitXor for PacketFlags {
	type Output = Self;

	fn bitxor(self, rhs: Self) -> Self::Output {
		Self(self.0 & !rhs.0)
	}
}

impl ops::BitXorAssign for PacketFlags {
	fn bitxor_assign(&mut self, rhs: Self) {
		self.0 &= !rhs.0;
	}
}

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
	pub const MAX_PACKET_SIZE: usize = 0xffff; // u16::MAX
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

macro_rules! packet_fields {
	($packet_struct_name:ident $(
		{
			type: $field_type:ty,
			offset: $field_byte_offset:literal,
			$(get: { fn: $field_getter_fn_ident:ident, $(vis: $field_getter_vis:vis,)? $(map: $field_get_map_fn:expr,)? },)?
			$(set: { fn: $field_setter_fn_ident:ident, $(vis: $field_setter_vis:vis,)? $(map: $field_set_map_fn:expr,)? },)?
		}
	)*) => {
		impl $packet_struct_name {
			pub const HEADER_SIZE: usize = 0 $( + mem::size_of::<$field_type>())*;

			$(
				$(
					$($field_getter_vis)? unsafe fn $field_getter_fn_ident(&self) -> $field_type {
						debug_assert!($field_byte_offset + mem::size_of::<$field_type>() <= Self::HEADER_SIZE);

						let x = read_offset(self.inner.as_ref(), $field_byte_offset);
						$(let x = $field_get_map_fn(x);)?
						x
					}
				)?
			)*

			$(
				$(
					$($field_setter_vis)? unsafe fn $field_setter_fn_ident(&mut self, value: $field_type) {
						debug_assert!($field_byte_offset + mem::size_of::<$field_type>() <= Self::HEADER_SIZE);

						let x = value;
						$(let x = $field_set_map_fn(x);)?
						write_offset(self.inner.as_mut(), $field_byte_offset, x);
					}
				)?
			)*
		}
	};
}

packet_fields! { PacketBuf
	{ // Version
		type: u16,
		offset: 0,
		get: { fn: unsafe_get_version, map: u16::from_be, },
		set: { fn: unsafe_set_version, map: u16::to_be, },
	}
	{ // Packet Length
		type: u16,
		offset: 2,
		get: { fn: unsafe_get_packet_length, map: u16::from_be, },
		set: { fn: unsafe_set_packet_length, map: u16::to_be, },
	}
	{ // Header Checksum
		type: u16,
		offset: 4,
		get: { fn: unsafe_get_header_checksum, map: u16::from_be, },
		set: { fn: unsafe_set_header_checksum, map: u16::to_be, },
	}
	{ // Packet flags
		type: u8,
		offset: 6,
		get: { fn: unsafe_get_flags, map: u8::from_be, },
		set: { fn: unsafe_set_flags, map: u8::to_be, },
	}
	{ // Packet ID
		type: u8,
		offset: 7,
		get: { fn: unsafe_get_packet_id, map: u8::from_be, },
		set: { fn: unsafe_set_packet_id, map: u8::to_be, },
	}
}

impl PacketBuf {
	fn calculate_header_checksum(&mut self) -> u16 {
		unsafe { self.unsafe_set_header_checksum(0) }
		crate::crc::checksum(*self.header())
	}

	fn get_header_checksum(&self) -> u16 {
		unsafe { self.unsafe_get_header_checksum() }
	}

	fn update_header_checksum(&mut self) {
		let new_checksum = self.calculate_header_checksum();
		unsafe { self.unsafe_set_header_checksum(new_checksum) }
	}
}

impl PacketBuf {}

impl PacketBuf {
	const VERSION: u16 = 0x1;

	pub fn get_packet_length(&self) -> u16 {
		unsafe { self.unsafe_get_packet_length() }
	}

	pub fn set_packet_length(&mut self, length: u16) {
		unsafe { self.unsafe_set_packet_length(length) }
	}

	pub fn get_id(&self) -> PacketId {
		unsafe { PacketId::from(self.unsafe_get_packet_id()) }
	}

	pub fn set_id(&mut self, id: PacketId) {
		unsafe { self.unsafe_set_packet_id(id.into()) }
	}

	pub fn get_flags(&self) -> PacketFlags {
		unsafe { PacketFlags::new_unchecked(self.unsafe_get_flags()) }
	}

	pub fn set_flags(&mut self, flags: PacketFlags) {
		unsafe { self.unsafe_set_flags(flags.into()) }
	}

	pub fn finalize(&mut self) {
		unsafe { self.unsafe_set_version(Self::VERSION) }
		self.update_header_checksum();
	}

	pub fn verify_header(&mut self) -> Result<()> {
		// check version
		if unsafe { self.unsafe_get_version() } != Self::VERSION {
			return Err(Error::VersionMismatch);
		}

		// verify header checksum
		{
			let unverified = self.get_header_checksum();
			let calculated = self.calculate_header_checksum();
			if unverified != calculated {
				return Err(Error::ChecksumError);
			}
		}

		Ok(())
	}
}

//! Utilities to work with packet headers.

use core::mem::{size_of, transmute};
use core::num::Wrapping;
use core::ops;
use core::ptr;

use crate::util::static_assert;

macro_rules! impl_bounded_num_type {
	(
		Self = $Self:path,
		Repr = $Repr:ty,
		Max = $max:expr,
		Min = $min:expr,
	) => {
		/// Minimum value of this type.
		pub const MIN: Self = Self($min as $Repr);

		/// Maximum value of this type.
		pub const MAX: Self = Self($max as $Repr);

		/// # Safety
		///
		/// The caller must ensure `x` is contained in the range: `MIN..=MAX`.
		#[inline]
		#[track_caller]
		pub const unsafe fn new_unchecked(x: $Repr) -> Self {
			assert!(Self::MIN.0 <= x && x <= Self::MAX.0);
			Self(x)
		}

		/// The safe version of [`new_unchecked`].
		///
		/// Returns `None` only if `x` is not contained in the range: `MIN..=MAX`.
		#[inline]
		pub const fn new(x: $Repr) -> Option<Self> {
			if (Self::MIN.0 <= x) && (x <= Self::MAX.0) {
				Some(unsafe { Self::new_unchecked(x) })
			} else {
				None
			}
		}

		/// Returns the `MAX` if `x` is greater than `MAX` and `MIN` if `x` is
		/// less than `MIN`.
		#[inline]
		pub const fn new_saturating(mut x: $Repr) -> Self {
			if x < Self::MIN.0 {
				x = Self::MIN.0;
			} else if x > Self::MAX.0 {
				x = Self::MAX.0;
			}

			unsafe { Self::new_unchecked(x) }
		}
	};
}

static_assert!(
	u16::MAX as u64 <= usize::MAX as u64,
	"cannot build on platforms where usize is smaller than u16"
);

/// A numeric type that represents any valid packet length.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PacketLength(u16);

impl PacketLength {
	impl_bounded_num_type! {
		Self = PacketLength,
		Repr = u16,
		Max = u16::MAX,
		Min = Header::SIZE_U16,
	}

	/// Get the length as a [`usize`].
	#[inline]
	pub const fn as_usize(&self) -> usize {
		// SAFETY: usize is always greater or equal to u16
		self.0 as usize
	}

	/// Get the length as a [`u16`].
	#[inline]
	pub const fn as_u16(&self) -> u16 {
		self.0
	}

	/// Convert this packet length to a payload length.
	#[inline]
	pub const fn to_payload_length(&self) -> PayloadLength {
		unsafe {
			// SAFETY: PacketLength::MIN <= self <= PacketLength::MAX
			//     <=> Header::SIZE_U16 <= self <= u16::MAX
			//     <=> 0 <= self - Header::SIZE_U16 <= u16::MAX - Header::SIZE_U16
			//     <=> PayloadLength::MIN <= self - Header::SIZE_U16 <= PayloadLength::MAX
			static_assert!(PayloadLength::MIN.0 == 0);
			static_assert!(
				PayloadLength::MAX.0 == u16::MAX - Header::SIZE_U16
			);

			PayloadLength::new_unchecked(self.0 - Header::SIZE_U16)
		}
	}
}

/// A numeric type that represents any valid payload length.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PayloadLength(u16);

impl PayloadLength {
	impl_bounded_num_type! {
		Self = PayloadLength,
		Repr = u16,
		Max = PacketLength::MAX.as_u16() - Header::SIZE_U16,
		Min = 0,
	}

	/// Get the length as a [`usize`].
	#[inline]
	pub const fn as_usize(&self) -> usize {
		// SAFETY: usize is always greater or equal to u16
		self.0 as usize
	}

	/// Get the length as a [`u16`].
	#[inline]
	pub const fn as_u16(&self) -> u16 {
		self.0
	}

	/// Convert this payload length to a packet length.
	#[inline]
	pub const fn to_packet_length(&self) -> PacketLength {
		unsafe {
			// SAFETY: PayloadLength::MIN <= self <= PayloadLength::MAX
			//     <=> 0 <= self <= PacketLength::MAX - Header::SIZE_U16
			//     <=> Header::SIZE_U16 <= self + Header::SIZE_U16 <= PacketLength::MAX
			//     <=> PacketLength::MIN <= self + Header::SIZE_U16 <= PacketLength::MAX
			static_assert!(PacketLength::MIN.0 == Header::SIZE_U16);
			static_assert!(PacketLength::MAX.0 == u16::MAX);

			PacketLength::new_unchecked(self.0 + Header::SIZE_U16)
		}
	}
}

/// Header flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Flags(u8);

impl Flags {
	/// More data flag.
	pub const MORE_DATA: Self = Self(1 << 7);
}

impl Flags {
	/// Create an empty [`Flags`] structure.
	#[inline]
	pub const fn zero() -> Self {
		Self(0)
	}

	/// Check whether all bits of `flags` are currently set.
	#[inline]
	pub const fn is_set(&self, flags: Self) -> bool {
		(self.0 & flags.0) ^ flags.0 == 0
	}

	/// Set all bits of `flags`.
	#[inline]
	pub fn set(&mut self, flags: Self) {
		self.0 |= flags.0;
	}

	/// Unset all bits of `flags`.
	#[inline]
	pub fn unset(&mut self, flags: Self) {
		self.0 &= !flags.0;
	}

	/// Conditionally set `flags` if _f_ returns true.
	#[inline]
	pub fn set_if<F>(mut self, flags: Self, f: F) -> Self
	where
		F: FnOnce(Self) -> bool,
	{
		if f(self) {
			self.set(flags);
		}

		self
	}
}

impl ops::BitAnd for Flags {
	type Output = bool;

	fn bitand(self, rhs: Self) -> Self::Output {
		self.is_set(rhs)
	}
}

impl ops::BitOr for Flags {
	type Output = Self;

	fn bitor(mut self, rhs: Self) -> Self::Output {
		self.set(rhs);
		self
	}
}

impl ops::BitOrAssign for Flags {
	fn bitor_assign(&mut self, rhs: Self) {
		self.set(rhs);
	}
}

/// Packet ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Id(u8);

/// A never-ending iterator of [packet IDs](Id).
#[derive(Debug, Clone)]
pub struct IdGenerator {
	current: Wrapping<u8>,
}

impl IdGenerator {
	/// Create a new [`IdGenerator`].
	pub const fn new() -> Self {
		Self { current: Wrapping(0) }
	}

	pub(self) fn current(&self) -> Id {
		Id(self.current.0)
	}

	/// Get the next [`Id`].
	#[must_use = "unused generated id"]
	pub fn next_id(&mut self) -> Id {
		let ret = self.current();
		self.current += 1;
		ret
	}
}

impl Default for IdGenerator {
	fn default() -> Self {
		Self::new()
	}
}

impl Iterator for IdGenerator {
	type Item = Id;

	fn next(&mut self) -> Option<Self::Item> {
		Some(self.next_id())
	}
}

/// Header checksum.
#[derive(Debug, Clone)]
pub struct Checksum {
	state: u32,
}

impl Checksum {
	/// Create a new empty checksum.
	pub const fn new() -> Self {
		Self { state: 0 }
	}

	/// Update the checksum from `buf`.
	pub fn update(&mut self, data: &[u8]) {
		let mut ptr: *const u16 = data.as_ptr().cast();
		let mut left = data.len();

		while left > 1 {
			unsafe {
				self.state += u32::from(ptr::read_unaligned(ptr));
				ptr = ptr.add(1);
				left -= 2;
			}
		}

		if left == 1 {
			unsafe {
				self.state +=
					u32::from(ptr::read_unaligned(ptr.cast::<u8>()));
			}
		}
	}

	/// Same as [`Checksum::update`] but for use with the builder pattern.
	pub fn chain_update(mut self, data: &[u8]) -> Self {
		self.update(data);
		self
	}

	/// Finalize the checksum.
	pub fn finalize(mut self) -> u16 {
		while (self.state >> 16) != 0 {
			self.state = (self.state >> 16) + (self.state & 0xffff);
		}

		!self.state as u16
	}

	/// Calculate the checksum of `data`.
	///
	/// Equivalent to: `Checksum::new().chain_update(data).finalize()`.
	pub fn checksum(data: &[u8]) -> u16 {
		Self::new().chain_update(data).finalize()
	}
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
struct u16be(u16);

impl u16be {
	pub const fn get(&self) -> u16 {
		u16::from_be(self.0)
	}

	pub fn set(&mut self, x: u16) {
		self.0 = x.to_be();
	}
}

#[repr(C)]
struct RawHeader {
	pub version: u16be,
	pub length: u16be,
	pub checksum: u16,
	pub flags: u8,
	pub id: u8,
}

static_assert!(size_of::<RawHeader>() == Header::SIZE);

impl RawHeader {
	pub fn new_ref(buf: &[u8; Header::SIZE]) -> &Self {
		unsafe { transmute(buf) }
	}

	pub fn new_mut(buf: &mut [u8; Header::SIZE]) -> &mut Self {
		unsafe { transmute(buf) }
	}

	pub fn as_bytes(&self) -> &[u8; Header::SIZE] {
		unsafe { &*ptr::addr_of!(*self).cast() }
	}
}

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

/// Possible errors while reading a header.
///
/// This is the error type returned by [`Header::read_from`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeaderReadError {
	/// The `version` field of the header is not supported.
	VersionMismatch,
	/// The `checksum` field of the header is invalid.
	InvalidChecksum,
	/// The `length` field of the header is invalid.
	InvalidLength,
	/// The `id` field is not equal to the next expected ID.
	OutOfOrder,
}

impl Header {
	pub(crate) const SIZE_U16: u16 = 8;

	/// The size of the header in bytes.
	///
	/// This is not the same as [`core::mem::size_of`].
	pub const SIZE: usize = Self::SIZE_U16 as usize;

	const VERSION: u16 = 0xfd3f;

	/// Convert the header to its raw format.
	pub fn to_bytes(&self) -> [u8; Self::SIZE] {
		let mut bytes = [0u8; Self::SIZE];

		{
			let raw = RawHeader::new_mut(&mut bytes);

			raw.version.set(Self::VERSION);
			raw.length.set(self.length.as_u16());
			raw.checksum = 0;
			raw.flags = self.flags.0;
			raw.id = self.id.0;

			let checksum = Checksum::checksum(raw.as_bytes());
			raw.checksum = checksum;
		}

		bytes
	}

	/// Write the buffer to `buf`.
	///
	/// You can use [`slice_to_array_mut`] to convert a slice to a reference to
	/// an array.
	///
	/// [`slice_to_array_mut`]: crate::util::slice_to_array_mut
	pub fn write_to(&self, buf: &mut [u8; Self::SIZE]) {
		*buf = self.to_bytes();
	}

	/// Read a header from `buf`.
	///
	/// **Note:** Use [`slice_to_array{_mut}`] if you have a slice.
	///
	/// You can use [`slice_to_array`] to convert a slice to a reference to
	/// an array.
	///
	/// [`slice_to_array`]: crate::util::slice_to_array
	pub fn read_from(
		buf: &[u8; Self::SIZE],
		gen: &mut IdGenerator,
	) -> Result<Self, HeaderReadError> {
		use HeaderReadError as E;

		let raw = RawHeader::new_ref(buf);

		if raw.version.get() != Self::VERSION {
			return Err(E::VersionMismatch);
		}

		if Checksum::checksum(raw.as_bytes()) != 0 {
			return Err(E::InvalidChecksum);
		}

		let length = PacketLength::new(raw.length.get())
			.ok_or(E::InvalidLength)?;

		let id = Id(raw.id);
		if gen.current() != id {
			return Err(E::OutOfOrder);
		} else {
			let _ = gen.next_id();
		}

		let flags = Flags(raw.flags);

		Ok(Self { length, flags, id })
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_checksum_impl() {
		fn test_case(data: &[u8], expected: u16) {
			let calculated = Checksum::checksum(data).to_be();
			assert_eq!(
				expected, calculated,
				"{expected:#x?} != {calculated:#x?}"
			);
		}

		test_case(
			&[
				0x45, 0x00, 0x00, 0x97, 0x8b, 0x64, 0x40, 0x00, 0x40,
				0x06, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x94, 0x01, 0x01,
				0x01, 0x01,
			],
			0xa267,
		);

		test_case(
			&[
				0x45, 0x00, 0x02, 0x20, 0x54, 0x74, 0x40, 0x00, 0x37,
				0x06, 0x00, 0x00, 0x01, 0x01, 0x01, 0x01, 0x0a, 0x00,
				0x00, 0x94,
			],
			0xe0ce,
		);

		test_case(
			&[
				0x45, 0x00, 0x00, 0xb3, 0x9b, 0xe9, 0x40, 0x00, 0xff,
				0x11, 0xf3, 0xc5, 0x0a, 0x00, 0x00, 0x8f, 0xe0, 0x00,
				0x00, 0xfb,
			],
			0x0000,
		);

		test_case(
			&[
				0x45, 0x00, 0x00, 0x73, 0x7e, 0x9b, 0x40, 0x00, 0x35,
				0x06, 0x4f, 0x1a, 0x03, 0x4a, 0x69, 0xf2, 0x0a, 0x00,
				0x00, 0x94,
			],
			0x0000,
		);
	}

	#[test]
	fn test_header_write() {
		assert_eq!(
			Header {
				length: PacketLength::new(1234).unwrap(),
				flags: Flags::zero(),
				id: Id(42),
			}
			.to_bytes(),
			[0xfd, 0x3f, 0x04, 0xd2, 0xfd, 0xc3, 0x00, 0x2a]
		);

		assert_eq!(
			Header {
				length: PacketLength::new(42).unwrap(),
				flags: Flags::MORE_DATA,
				id: Id(0),
			}
			.to_bytes(),
			[0xfd, 0x3f, 0x00, 0x2a, 0x82, 0x95, 0x80, 0x00]
		);
	}

	#[test]
	fn test_header_read() {
		let mut gen = IdGenerator::new();

		assert_eq!(
			Header::read_from(
				&[0xfd, 0x3f, 0x04, 0xd2, 0xfd, 0xed, 0x00, 0x00],
				&mut gen
			)
			.unwrap(),
			Header {
				length: PacketLength::new(1234).unwrap(),
				flags: Flags::zero(),
				id: Id(0),
			}
		);

		assert_eq!(
			Header::read_from(
				&[0xfd, 0x3f, 0x00, 0x2a, 0x82, 0x94, 0x80, 0x01],
				&mut gen
			)
			.unwrap(),
			Header {
				length: PacketLength::new(42).unwrap(),
				flags: Flags::MORE_DATA,
				id: Id(1),
			}
		);
	}
}

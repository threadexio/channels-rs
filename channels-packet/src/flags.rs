use core::ops::{BitAnd, BitOr, BitOrAssign};

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
	#[must_use]
	pub const fn zero() -> Self {
		Self(0)
	}

	/// Check whether all bits of `flags` are currently set.
	#[inline]
	#[must_use]
	pub const fn is_set(self, flags: Self) -> bool {
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
	#[must_use]
	pub fn set_if<F>(mut self, flags: Self, f: F) -> Self
	where
		F: FnOnce(Self) -> bool,
	{
		if f(self) {
			self.set(flags);
		}

		self
	}

	/// Get the raw bits.
	#[inline]
	#[must_use]
	pub fn bits(self) -> u8 {
		self.0
	}
}

impl Default for Flags {
	fn default() -> Self {
		Self::zero()
	}
}

impl From<Flags> for u8 {
	fn from(value: Flags) -> Self {
		value.bits()
	}
}

impl From<u8> for Flags {
	fn from(value: u8) -> Self {
		Self(value)
	}
}

impl BitAnd for Flags {
	type Output = bool;

	fn bitand(self, rhs: Self) -> Self::Output {
		self.is_set(rhs)
	}
}

impl BitOr for Flags {
	type Output = Self;

	fn bitor(mut self, rhs: Self) -> Self::Output {
		self.set(rhs);
		self
	}
}

impl BitOrAssign for Flags {
	fn bitor_assign(&mut self, rhs: Self) {
		self.set(rhs);
	}
}

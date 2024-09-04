use core::fmt;

/// Frame flags.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Flags(u8);

impl fmt::Debug for Flags {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_set().finish()
	}
}

impl Flags {
	/// Create a new set of frame flags with no flag set.
	#[inline]
	#[must_use]
	pub const fn empty() -> Self {
		Self(0)
	}

	/// Create a new set of frame flags from `bits`.
	///
	/// The upper 6 bits of `bits` are discarded.
	#[inline]
	#[must_use]
	pub const fn from_bits(bits: u8) -> Self {
		Self(bits & 0b11)
	}

	/// Get the bit representation of the flags.
	#[inline]
	#[must_use]
	pub const fn bits(self) -> u8 {
		self.0
	}

	/// Check whether any flag in `other` is set in `self`.
	///
	/// If `other` contains only one flag, this method behaves identical to [`is_all_set()`].
	///
	/// [`is_all_set()`]: Self::is_all_set
	#[inline]
	#[must_use]
	pub const fn is_any_set(self, other: Self) -> bool {
		self.0 & other.0 != 0
	}

	/// Check whether all flags in `other` are set in `self`.
	///
	/// If `other` contains only one flag, this method behaves identical to [`is_any_set()`].
	///
	/// [`is_any_set()`]: Self::is_any_set
	#[inline]
	#[must_use]
	pub const fn is_all_set(self, other: Self) -> bool {
		(self.0 & other.0) ^ other.0 == 0
	}

	/// Set the flags of `other` in `self`.
	#[inline]
	pub fn set(&mut self, other: Self) {
		self.0 |= other.0;
	}

	/// Set the flags of `other` in `self` if the predicate _f_ returns `true`.
	#[inline]
	pub fn set_if<F>(&mut self, other: Self, f: F)
	where
		F: FnOnce() -> bool,
	{
		if f() {
			self.set(other);
		}
	}

	/// Unset the flags of `other` in `self`.
	#[inline]
	pub fn unset(&mut self, other: Self) {
		self.0 &= !other.0;
	}

	/// Unset the flags of `other` in `self` if the predicate _f_ returns `true`.
	#[inline]
	pub fn unset_if<F>(&mut self, other: Self, f: F)
	where
		F: FnOnce() -> bool,
	{
		if f() {
			self.unset(other);
		}
	}

	/// Set or unset the flags of `other` in `self` depending on `set`.
	///
	/// If `set` is `true`, this method will set the flags of `other` in `self`. If `set`
	/// is `false`, it will unset them.
	#[inline]
	pub fn set_or_unset(&mut self, other: Self, set: bool) {
		if set {
			self.set(other);
		} else {
			self.unset(other);
		}
	}

	/// Set or unset the flags of `other` in `self` based on a predicate _f_.
	#[inline]
	pub fn set_or_unset_with<F>(&mut self, other: Self, f: F)
	where
		F: FnOnce() -> bool,
	{
		self.set_or_unset(other, f());
	}
}

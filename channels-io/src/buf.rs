use core::ops::{Deref, DerefMut};

use crate::util::{Bytes, BytesMut};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IoSliceGeneric<T> {
	data: T,
	pos: usize,
}

impl<T> IoSliceGeneric<T> {
	/// Create a new [`IoSliceGeneric`] from `data`.
	pub const fn new(data: T) -> Self {
		Self { data, pos: 0 }
	}

	/// Get a reference to the inner type.
	pub fn inner_ref(&self) -> &T {
		&self.data
	}

	/// Get a mutable reference to the inner type.
	pub fn inner_mut(&mut self) -> &mut T {
		&mut self.data
	}

	/// Destruct the slice into its inner type.
	pub fn into_inner(self) -> T {
		self.data
	}
}

impl<T: Bytes> IoSliceGeneric<T> {
	/// Return the slice before the cursor.
	pub fn filled(&self) -> &[u8] {
		&self.data.as_bytes()[..self.pos]
	}

	/// Return the slice after the cursor.
	pub fn unfilled(&self) -> &[u8] {
		&self.data.as_bytes()[self.pos..]
	}
}

impl<T: BytesMut> IoSliceGeneric<T> {
	/// Return the slice before the cursor.
	pub fn filled_mut(&mut self) -> &mut [u8] {
		&mut self.data.as_mut_bytes()[..self.pos]
	}

	/// Return the slice after the cursor.
	pub fn unfilled_mut(&mut self) -> &mut [u8] {
		&mut self.data.as_mut_bytes()[self.pos..]
	}
}

impl<T: Bytes> IoSliceGeneric<T> {
	/// Advance the slice by `n` bytes.
	///
	/// # Safety
	///
	/// `n` must be less or equal to the length of the slice.
	///
	/// # Panics
	///
	/// Panics if `n` is greater than the length of the slice.
	pub fn advance(&mut self, n: usize) {
		assert!(n <= self.len());
		self.pos += n;
	}
}

impl<T: Bytes> Deref for IoSliceGeneric<T> {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		self.unfilled()
	}
}

impl<T: BytesMut> DerefMut for IoSliceGeneric<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.unfilled_mut()
	}
}

/// An owned byte buffer that tracks how many bytes are filled.
pub type IoSlice<T> = IoSliceGeneric<T>;

/// A slice that tracks how many bytes are filled.
pub type IoSliceRef<'a> = IoSliceGeneric<&'a [u8]>;

/// A mutable slice that tracks how many bytes are filled.
pub type IoSliceMut<'a> = IoSliceGeneric<&'a mut [u8]>;

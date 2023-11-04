use crate::util::{Bytes, BytesMut};

/// A trait for immutable buffers.
pub trait Buf {
	/// Get the amount of remaining bytes in the buffer.
	fn remaining(&self) -> usize;
	/// Get a slice to the remaining bytes.
	fn unfilled(&self) -> &[u8];
	/// Advance the internal cursor of the buffer by `n` bytes.
	fn advance(&mut self, n: usize);

	/// Returns whether the buffer has any more remaining data.
	///
	/// Equivalent to: `self.remaining() != 0`.
	fn has_remaining(&self) -> bool {
		self.remaining() != 0
	}
}

impl<T: Buf + ?Sized> Buf for &mut T {
	fn remaining(&self) -> usize {
		(**self).remaining()
	}

	fn unfilled(&self) -> &[u8] {
		(**self).unfilled()
	}

	fn advance(&mut self, n: usize) {
		(**self).advance(n)
	}

	fn has_remaining(&self) -> bool {
		(**self).has_remaining()
	}
}

impl Buf for &[u8] {
	fn remaining(&self) -> usize {
		self.len()
	}

	fn unfilled(&self) -> &[u8] {
		self
	}

	fn advance(&mut self, n: usize) {
		*self = &self[n..];
	}
}

/// A trait for mutable buffers.
pub trait BufMut {
	/// Get the amount of remaining bytes in the buffer.
	fn remaining_mut(&self) -> usize;
	/// Get a slice to the remaining bytes.
	fn unfilled_mut(&mut self) -> &mut [u8];
	/// Advance the internal cursor of the buffer by `n` bytes.
	fn advance_mut(&mut self, n: usize);

	/// Returns whether the buffer has any more remaining data.
	///
	/// Equivalent to: `self.remaining() != 0`.
	fn has_remaining_mut(&self) -> bool {
		self.remaining_mut() != 0
	}
}

impl<T: BufMut + ?Sized> BufMut for &mut T {
	fn remaining_mut(&self) -> usize {
		(**self).remaining_mut()
	}

	fn unfilled_mut(&mut self) -> &mut [u8] {
		(**self).unfilled_mut()
	}

	fn advance_mut(&mut self, n: usize) {
		(**self).advance_mut(n)
	}

	fn has_remaining_mut(&self) -> bool {
		(**self).has_remaining_mut()
	}
}

impl BufMut for &mut [u8] {
	fn remaining_mut(&self) -> usize {
		self.len()
	}

	fn unfilled_mut(&mut self) -> &mut [u8] {
		self
	}

	fn advance_mut(&mut self, n: usize) {
		let b = core::mem::take(self);
		*self = &mut b[n..];
	}
}

/// An owned byte buffer that tracks how many bytes are filled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IoSlice<T> {
	data: T,
	pos: usize,
}

impl<T> IoSlice<T> {
	/// Create a new [`IoSlice`] from `data`.
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

	/// Set the absolute position of the internal cursor.
	///
	/// # Safety
	///
	/// `pos` must not be greater than the total length of the slice.
	///
	/// # Panics
	///
	/// Panics if `pos` is greater than the total length of the slice.
	pub unsafe fn set_filled(&mut self, pos: usize)
	where
		T: Bytes,
	{
		assert!(self.pos <= self.data.as_bytes().len());
		self.pos = pos;
	}
}

impl<T: Bytes> Buf for IoSlice<T> {
	fn remaining(&self) -> usize {
		self.data.as_bytes().len() - self.pos
	}

	fn unfilled(&self) -> &[u8] {
		&self.data.as_bytes()[self.pos..]
	}

	fn advance(&mut self, n: usize) {
		assert!(n <= self.remaining());
		self.pos += n;
	}
}

impl<T: BytesMut> BufMut for IoSlice<T> {
	fn remaining_mut(&self) -> usize {
		self.data.as_bytes().len() - self.pos
	}

	fn unfilled_mut(&mut self) -> &mut [u8] {
		&mut self.data.as_mut_bytes()[self.pos..]
	}

	fn advance_mut(&mut self, n: usize) {
		assert!(n <= self.remaining_mut());
		self.pos += n;
	}
}

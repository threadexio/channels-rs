use core::cmp::min;

use crate::buf::BufMut;

/// The `limit` adapter.
///
/// See: [`BufMut::limit`].
#[derive(Debug, Clone, Copy)]
pub struct Limit<T> {
	inner: T,
	left: usize,
}

impl<T> Limit<T> {
	pub(super) fn new(inner: T, left: usize) -> Self {
		Self { inner, left }
	}

	/// Get the number of bytes logically left in the buffer.
	///
	/// This method is not the same as [`BufMut::remaining_mut()`]. It returns the maximum
	/// number of bytes the underlying buffer is allowed to have left. The returned value
	/// might be greater than the actual amount of bytes the underlying buffer has left.
	#[inline]
	pub fn left(&self) -> usize {
		self.left
	}

	/// Set the maximum amount of bytes the underlying buffer is allowed to have.
	#[inline]
	pub fn set_left(&mut self, left: usize) {
		self.left = left;
	}

	/// Get a reference to the underlying buffer.
	#[inline]
	pub fn get(&self) -> &T {
		&self.inner
	}

	/// Get a mutable reference to the underlying buffer.
	#[inline]
	pub fn get_mut(&mut self) -> &mut T {
		&mut self.inner
	}

	/// Destruct the adapter and get back the underlying buffer.
	#[inline]
	pub fn into_inner(self) -> T {
		self.inner
	}
}

impl<T: BufMut> BufMut for Limit<T> {
	fn remaining_mut(&self) -> usize {
		min(self.inner.remaining_mut(), self.left)
	}

	fn chunk_mut(&mut self) -> &mut [u8] {
		let s = self.inner.chunk_mut();

		if s.len() > self.left {
			&mut s[..self.left]
		} else {
			s
		}
	}

	fn advance_mut(&mut self, n: usize) {
		assert!(
			n <= self.left,
			"n must not be greater than the amount of bytes left"
		);

		self.left -= n;
	}
}

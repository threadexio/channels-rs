use core::cmp::min;

use crate::buf::Buf;

/// The `take` adapter.
///
/// See: [`Buf::take`].
#[derive(Debug, Clone, Copy)]
pub struct Take<T> {
	inner: T,
	left: usize,
}

impl<T> Take<T> {
	pub(super) fn new(inner: T, left: usize) -> Self {
		Self { inner, left }
	}

	/// Get the number of bytes logically left in the buffer.
	///
	/// This method is not the same as [`Buf::remaining()`]. It returns the maximum number
	/// of bytes the underlying buffer is allowed to have left. The returned value might be
	/// greater than the actual amount of bytes the underlying buffer has left.
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

impl<T: Buf> Buf for Take<T> {
	fn remaining(&self) -> usize {
		min(self.inner.remaining(), self.left)
	}

	fn chunk(&self) -> &[u8] {
		let s = self.inner.chunk();

		if s.len() > self.left {
			&s[..self.left]
		} else {
			s
		}
	}

	fn advance(&mut self, n: usize) {
		assert!(
			n <= self.left,
			"n must not be greater than the amount of bytes left"
		);

		self.left -= n;
	}
}

use core::cmp::min;

use crate::buf::Buf;

/// TODO: docs
#[derive(Debug, Clone, Copy)]
pub struct Take<T> {
	inner: T,
	left: usize,
}

impl<T> Take<T> {
	pub(super) fn new(inner: T, left: usize) -> Self {
		Self { inner, left }
	}

	/// TODO: docs
	#[inline]
	pub fn get(&self) -> &T {
		&self.inner
	}

	/// TODO: docs
	#[inline]
	pub fn get_mut(&mut self) -> &mut T {
		&mut self.inner
	}

	/// TODO: docs
	#[inline]
	pub fn left(&self) -> usize {
		self.left
	}

	/// TODO: docs
	#[inline]
	pub fn set_left(&mut self, left: usize) {
		self.left = left;
	}

	/// TODO: docs
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

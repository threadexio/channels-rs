use core::cmp::min;

use crate::buf::BufMut;

/// TODO: docs
#[derive(Debug, Clone, Copy)]
pub struct Limit<T> {
	inner: T,
	left: usize,
}

impl<T> Limit<T> {
	pub(super) fn new(inner: T, left: usize) -> Self {
		Self { inner, left }
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

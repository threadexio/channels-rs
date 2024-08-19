use crate::buf::{Buf, BufMut};

/// TODO: docs
#[derive(Debug, Clone, Copy)]
pub struct Chain<A, B> {
	first: A,
	second: B,
}

impl<A, B> Chain<A, B> {
	pub(super) fn new(first: A, second: B) -> Self {
		Self { first, second }
	}

	/// TODO: docs
	#[inline]
	pub fn first(&self) -> &A {
		&self.first
	}

	/// TODO: docs
	#[inline]
	pub fn first_mut(&mut self) -> &mut A {
		&mut self.first
	}

	/// TODO: docs
	#[inline]
	pub fn second(&self) -> &B {
		&self.second
	}

	/// TODO: docs
	#[inline]
	pub fn second_mut(&mut self) -> &mut B {
		&mut self.second
	}

	/// TODO: docs
	#[inline]
	pub fn into_first(self) -> A {
		self.first
	}

	/// TODO: docs
	#[inline]
	pub fn into_second(self) -> B {
		self.second
	}

	/// TODO: docs
	#[inline]
	pub fn into_inner(self) -> (A, B) {
		(self.first, self.second)
	}
}

impl<A, B> Buf for Chain<A, B>
where
	A: Buf,
	B: Buf,
{
	fn remaining(&self) -> usize {
		self.first.remaining() + self.second.remaining()
	}

	fn chunk(&self) -> &[u8] {
		if self.first.has_remaining() {
			self.first.chunk()
		} else {
			self.second.chunk()
		}
	}

	fn advance(&mut self, n: usize) {
		let a_rem = self.first.remaining();

		if n > a_rem {
			self.first.advance(a_rem);
			self.second.advance(n - a_rem);
		} else {
			self.first.advance(n);
		}
	}
}

impl<A, B> BufMut for Chain<A, B>
where
	A: BufMut,
	B: BufMut,
{
	fn remaining_mut(&self) -> usize {
		self.first.remaining_mut() + self.second.remaining_mut()
	}

	fn chunk_mut(&mut self) -> &mut [u8] {
		if self.first.has_remaining_mut() {
			self.first.chunk_mut()
		} else {
			self.second.chunk_mut()
		}
	}

	fn advance_mut(&mut self, n: usize) {
		let a_rem = self.first.remaining_mut();

		if n > a_rem {
			self.first.advance_mut(a_rem);
			self.second.advance_mut(n - a_rem);
		} else {
			self.first.advance_mut(n);
		}
	}
}
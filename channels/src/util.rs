use alloc::vec::Vec;

/// Grow `vec` by `n` bytes and return the newly allocated bytes as a mutable
/// slice.
#[inline]
pub fn grow_vec_by_n(vec: &mut Vec<u8>, n: usize) -> &mut [u8] {
	let old_len = vec.len();
	let new_len = usize::saturating_add(old_len, n);

	vec.resize(new_len, 0);
	&mut vec[old_len..new_len]
}

/// Wrapper for an immutable slice that allows consuming the slice in parts.
pub struct ConsumeSlice<'a> {
	slice: &'a [u8],
}

impl<'a> ConsumeSlice<'a> {
	pub fn new(slice: &'a [u8]) -> Self {
		Self { slice }
	}

	/// Get the number of bytes remaining in the slice.
	pub fn remaining(&self) -> usize {
		self.slice.len()
	}

	/// Consume `n` bytes from the front of the slice.
	///
	/// # Panics
	///
	/// If `n` does not point inside the slice.s
	pub fn consume(&mut self, n: usize) -> &'a [u8] {
		let (a, b) = self.slice.split_at(n);
		self.slice = b;
		a
	}
}

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
	/// Return the slice of the entire buffer.
	pub fn entire(&self) -> &[u8] {
		self.data.as_bytes()
	}

	/// Return the slice before the cursor.
	pub fn filled(&self) -> &[u8] {
		&self.entire()[..self.pos]
	}

	/// Return the slice after the cursor.
	pub fn unfilled(&self) -> &[u8] {
		&self.entire()[self.pos..]
	}
}

impl<T: BytesMut> IoSliceGeneric<T> {
	/// Return the slice of the entire buffer.
	pub fn entire_mut(&mut self) -> &mut [u8] {
		self.data.as_mut_bytes()
	}

	/// Return the slice before the cursor.
	pub fn filled_mut(&mut self) -> &mut [u8] {
		let i = ..self.pos;
		&mut self.entire_mut()[i]
	}

	/// Return the slice after the cursor.
	pub fn unfilled_mut(&mut self) -> &mut [u8] {
		let i = self.pos..;
		&mut self.entire_mut()[i]
	}
}

impl<T: Bytes> IoSliceGeneric<T> {
	/// Set the length of the filled slice.
	///
	/// # Safety
	///
	/// `n` must not be greater than the length of the entire slice.
	///
	/// # Panics
	///
	/// Panics if `n` is greater than the length of the entire slice.
	#[track_caller]
	pub fn set_filled(&mut self, n: usize) {
		assert!(n <= self.entire().len());
		self.pos = n;
	}

	/// Advance of the filled slice by `n` bytes.
	///
	/// # Safety
	///
	/// `n` must not be greater than the length of the unfilled slice.
	///
	/// # Panics
	///
	/// Panics if `n` is greater than the length of the unfilled slice.
	#[track_caller]
	pub fn advance(&mut self, n: usize) {
		let n = usize::saturating_add(self.filled().len(), n);
		self.set_filled(n);
	}
}

impl<T: Bytes> IoSliceGeneric<T> {
	/// Compute the delta of the filled length across the execution of _f_.
	///
	/// Returns the delta of the length and the output of _f_ in a tuple.
	///
	/// # Example
	/// ```
	/// let mut buf = channels_io::IoSlice::new([0u8; 4]);
	///
	/// let (delta, output) = buf.delta_len(|buf| {
	///     buf[..4].copy_from_slice(&[1, 2, 3, 4]);
	///     buf.advance(4);
	/// });
	///
	/// assert_eq!(delta, 4);
	/// assert_eq!(buf.filled(), &[1, 2, 3, 4]);
	/// ```
	pub fn delta_len<F, O>(&mut self, f: F) -> (usize, O)
	where
		F: FnOnce(&mut Self) -> O,
	{
		let l0 = self.filled().len();
		let output = f(self);
		let l1 = self.filled().len();

		let delta = usize::saturating_sub(l1, l0);

		(delta, output)
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

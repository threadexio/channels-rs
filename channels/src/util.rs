use core::task::Poll;

/// Types which can be used as a byte slice.
pub trait Bytes: AsRef<[u8]> {
	/// Convert this type to a byte slice.
	fn as_bytes(&self) -> &[u8] {
		self.as_ref()
	}
}

/// Types which can be used as a mutable byte slice.
pub trait BytesMut: Bytes + AsMut<[u8]> {
	/// Convert this type to a mutable byte slice.
	fn as_mut_bytes(&mut self) -> &mut [u8] {
		self.as_mut()
	}
}

impl<T: AsRef<[u8]>> Bytes for T {}
impl<T: Bytes + AsMut<[u8]>> BytesMut for T {}

/// Extension trait for [`Poll`].
pub trait PollExt<T>: Sized {
	/// Returns the contained [`Poll::Ready`] value consuming the `self` value.
	///
	/// # Panics
	///
	/// Panics if the value is a [`Poll::Pending`] with a panic message provided
	/// by `msg`.
	#[track_caller]
	fn expect(self, msg: &str) -> T;

	/// Returns the contained [`Poll::Ready`] value consuming the `self` value.
	///
	/// # Panics
	///
	/// Panics if the value is a [`Poll::Pending`].
	#[track_caller]
	fn unwrap(self) -> T {
		self.expect("unwrap called on a `Poll::Pending`")
	}

	/// Returns the contained [`Poll::Ready`] value or `other` if the value was
	/// [`Poll::Pending`].
	#[track_caller]
	fn unwrap_or(self, other: T) -> T {
		self.unwrap_or_else(|| other)
	}

	/// Returns the contained [`Poll::Ready`] value or computes if from _f_.
	#[track_caller]
	fn unwrap_or_else<F>(self, f: F) -> T
	where
		F: FnOnce() -> T;

	/// Returns the contained [`Poll::Ready`] value or the default value of `T`
	/// if the value was [`Poll::Pending`].
	#[track_caller]
	fn unwrap_or_default(self) -> T
	where
		T: Default,
	{
		self.unwrap_or_else(|| T::default())
	}
}

impl<T> PollExt<T> for Poll<T> {
	#[track_caller]
	fn expect(self, msg: &str) -> T {
		#[cold]
		#[inline(never)]
		#[track_caller]
		fn panic_pending(msg: &str) -> ! {
			panic!("{}", msg)
		}

		match self {
			Poll::Pending => panic_pending(msg),
			Poll::Ready(v) => v,
		}
	}

	#[track_caller]
	fn unwrap_or_else<F>(self, f: F) -> T
	where
		F: FnOnce() -> T,
	{
		match self {
			Poll::Pending => f(),
			Poll::Ready(v) => v,
		}
	}
}

/// Copy the largest sub-slice of `src` possible into `dst`.
///
/// Returns the number of individual elements copied.
pub fn copy_slice<T: Copy>(src: &[T], dst: &mut [T]) -> usize {
	match core::cmp::min(src.len(), dst.len()) {
		0 => 0,
		n => {
			dst[..n].copy_from_slice(&src[..n]);
			n
		},
	}
}

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

/// A trait for mutable buffers.
pub trait BufMut: Buf {
	/// Get a slice to the remaining bytes.
	fn unfilled_mut(&mut self) -> &mut [u8];
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

impl<T: BufMut + ?Sized> BufMut for &mut T {
	fn unfilled_mut(&mut self) -> &mut [u8] {
		(**self).unfilled_mut()
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

impl Buf for &mut [u8] {
	fn remaining(&self) -> usize {
		self.len()
	}

	fn unfilled(&self) -> &[u8] {
		self
	}

	fn advance(&mut self, n: usize) {
		let b = core::mem::take(self);
		*self = &mut b[n..];
	}
}

impl BufMut for &mut [u8] {
	fn unfilled_mut(&mut self) -> &mut [u8] {
		self
	}
}

/// An owned byte buffer that tracks how many bytes are filled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IoSlice<T> {
	data: T,
	pos: usize,
}

#[allow(dead_code)]
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
	fn unfilled_mut(&mut self) -> &mut [u8] {
		&mut self.data.as_mut_bytes()[self.pos..]
	}
}

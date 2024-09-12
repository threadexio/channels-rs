use core::cmp::min;
use core::pin::Pin;
use core::task::{Context, Poll};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::buf::{Chain, Take};
use crate::error::{IoError, ReadError};
use crate::util::copy_slice;
use crate::{AsyncRead, Read};

/// An immutable buffer that contains an internal cursor.
pub trait Buf {
	/// Get the number of bytes remaining in the buffer.
	fn remaining(&self) -> usize;

	/// Get the remaining bytes of the buffer as a slice.
	///
	/// This method is allowed to return slices smaller than what [`Buf::remaining()`]
	/// describes. This allows non contiguous representation of the buffer in memory.
	fn chunk(&self) -> &[u8];

	/// Advance the internal cursor of the buffer by `n` bytes.
	///
	/// # Panics
	///
	/// If `n` is a value that would cause the cursor to go out of bounds.
	fn advance(&mut self, n: usize);

	/// Check whether the buffer has any data left in it.
	fn has_remaining(&self) -> bool {
		self.remaining() != 0
	}

	/// Copy data from the buffer into `slice` advancing the buffer accordingly.
	fn copy_to_slice(&mut self, slice: &mut [u8]) -> usize {
		let n = min(self.remaining(), slice.len());
		let mut i = 0;

		while i < n {
			let x = copy_slice(self.chunk(), &mut slice[i..]);
			self.advance(x);
			i += x;
		}

		n
	}

	#[cfg(feature = "alloc")]
	/// Copy the buffer into `vec`.
	fn copy_to_vec(mut self, vec: &mut Vec<u8>)
	where
		Self: Sized,
	{
		vec.reserve(self.remaining());

		while self.has_remaining() {
			let c = self.chunk();
			vec.extend_from_slice(c);
			self.advance(c.len());
		}
	}

	/// Create a "by reference" adapter that takes the current instance of [`Buf`]
	/// by mutable reference.
	fn by_ref(&mut self) -> &mut Self
	where
		Self: Sized,
	{
		self
	}

	/// Create an adapter that implements the [`Read`] and [`AsyncRead`] traits
	/// on the current instance of [`Buf`].
	fn reader(self) -> Reader<Self>
	where
		Self: Sized,
	{
		Reader::new(self)
	}

	/// Create a [`Chain`] adapter that chains this buffer with `other`.
	///
	/// The returned [`Buf`] will behave as a non-contiguous buffer made up of
	/// the contents of `self` and `other`.
	fn chain<T: Buf>(self, other: T) -> Chain<Self, T>
	where
		Self: Sized,
	{
		Chain::new(self, other)
	}

	/// Create a [`Take`] adapter that takes `n` bytes from this buffer.
	///
	/// The returned [`Buf`] will contain only the first `n` bytes of this buffer.
	fn take(self, n: usize) -> Take<Self>
	where
		Self: Sized,
	{
		Take::new(self, n)
	}
}

macro_rules! forward_impl_buf {
	($to:ty) => {
		fn remaining(&self) -> usize {
			<$to>::remaining(self)
		}

		fn chunk(&self) -> &[u8] {
			<$to>::chunk(self)
		}

		fn advance(&mut self, n: usize) {
			<$to>::advance(self, n);
		}

		fn has_remaining(&self) -> bool {
			<$to>::has_remaining(self)
		}

		fn copy_to_slice(&mut self, slice: &mut [u8]) -> usize {
			<$to>::copy_to_slice(self, slice)
		}
	};
}

impl<T: Buf + ?Sized> Buf for &mut T {
	forward_impl_buf!(T);
}

#[cfg(feature = "alloc")]
impl<T: Buf + ?Sized> Buf for alloc::boxed::Box<T> {
	forward_impl_buf!(T);
}

impl Buf for &[u8] {
	fn remaining(&self) -> usize {
		self.len()
	}

	fn chunk(&self) -> &[u8] {
		self
	}

	fn advance(&mut self, n: usize) {
		*self = &self[n..];
	}
}

/// The error returned by IO read calls to a [`Reader`].
#[derive(Debug, Clone, Copy)]
pub enum ReaderError {
	/// The reader has reached EOF.
	Eof,
}

impl IoError for ReaderError {
	fn should_retry(&self) -> bool {
		false
	}
}

impl ReadError for ReaderError {
	fn eof() -> Self {
		Self::Eof
	}
}

/// An adapter for [`Buf`] that implements both [`Read`] and [`AsyncRead`].
#[derive(Debug, Clone, Copy)]
pub struct Reader<B> {
	buf: B,
}

impl<B> Reader<B> {
	fn new(buf: B) -> Self {
		Self { buf }
	}
}

impl<B: Buf> Read for Reader<B> {
	type Error = ReaderError;

	fn read_slice(
		&mut self,
		buf: &mut [u8],
	) -> Result<usize, Self::Error> {
		if buf.is_empty() {
			return Ok(0);
		}

		let n = self.buf.copy_to_slice(buf);
		if n == 0 {
			return Err(ReaderError::Eof);
		}

		Ok(n)
	}
}

impl<B: Buf + Unpin> AsyncRead for Reader<B> {
	type Error = ReaderError;

	fn poll_read_slice(
		mut self: Pin<&mut Self>,
		_: &mut Context,
		buf: &mut [u8],
	) -> Poll<Result<usize, Self::Error>> {
		Poll::Ready(self.read_slice(buf))
	}
}

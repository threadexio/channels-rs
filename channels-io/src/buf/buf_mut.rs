use core::cmp::min;
use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::error::{IoError, WriteError};
use crate::util::copy_slice;
use crate::{AsyncWrite, Write};

/// A mutable and initialized buffer that contains an internal cursor.
pub trait BufMut {
	/// Get the number of bytes this buffer can hold.
	fn remaining_mut(&self) -> usize;

	/// Get the remaining part of the buffer as a mutable slice.
	///
	/// This method is allowed to return slices smaller than what [`BufMut::remaining_mut()`]
	/// describes. This allows non contiguous representation of the buffer in memory.
	fn chunk_mut(&mut self) -> &mut [u8];

	/// Advance the internal cursor of the buffer by `n` bytes.
	///
	/// # Panics
	///
	/// If `n` is a value that would cause the cursor to go out of bounds.
	fn advance_mut(&mut self, n: usize);

	/// Check whether the buffer has any data left in it.
	fn has_remaining_mut(&self) -> bool {
		self.remaining_mut() != 0
	}

	/// Copy data from the buffer into `slice` advancing the buffer accordingly.
	fn copy_from_slice(&mut self, slice: &[u8]) -> usize {
		let n = min(slice.len(), self.remaining_mut());
		let mut i = 0;

		while i < n {
			let x = copy_slice(&slice[i..], self.chunk_mut());
			self.advance_mut(x);
			i += x;
		}

		n
	}

	/// Create a "by reference" adapter that takes the current instance of [`BufMut`]
	/// by mutable reference.
	fn by_ref(&mut self) -> &mut Self
	where
		Self: Sized,
	{
		self
	}

	/// Create an adapter that implements the [`Write`] and [`AsyncWrite`] traits
	/// on the current instance of [`BufMut`].
	fn writer(self) -> Writer<Self>
	where
		Self: Sized,
	{
		Writer::new(self)
	}
}

macro_rules! forward_impl_buf_mut {
	($to:ty) => {
		fn remaining_mut(&self) -> usize {
			<$to>::remaining_mut(self)
		}

		fn chunk_mut(&mut self) -> &mut [u8] {
			<$to>::chunk_mut(self)
		}

		fn advance_mut(&mut self, n: usize) {
			<$to>::advance_mut(self, n)
		}

		fn has_remaining_mut(&self) -> bool {
			<$to>::has_remaining_mut(self)
		}

		fn copy_from_slice(&mut self, slice: &[u8]) -> usize {
			<$to>::copy_from_slice(self, slice)
		}
	};
}

impl<T: BufMut + ?Sized> BufMut for &mut T {
	forward_impl_buf_mut!(T);
}

#[cfg(feature = "alloc")]
impl<T: BufMut + ?Sized> BufMut for alloc::boxed::Box<T> {
	forward_impl_buf_mut!(T);
}

impl BufMut for &mut [u8] {
	fn remaining_mut(&self) -> usize {
		self.len()
	}

	fn chunk_mut(&mut self) -> &mut [u8] {
		self
	}

	fn advance_mut(&mut self, n: usize) {
		let tmp = mem::take(self);
		*self = &mut tmp[n..];
	}
}

/// The error returned by IO write calls to a [`Writer`].
#[derive(Debug, Clone, Copy)]
pub enum WriterError {
	/// The writer has no more space left.
	WriteZero,
}

impl IoError for WriterError {
	fn should_retry(&self) -> bool {
		false
	}
}

impl WriteError for WriterError {
	fn write_zero() -> Self {
		Self::WriteZero
	}
}

/// An adapter for [`BufMut`] that implements both [`Write`] and [`AsyncWrite`].
#[derive(Debug, Clone, Copy)]
pub struct Writer<B> {
	buf: B,
}

impl<B> Writer<B> {
	pub(crate) fn new(buf: B) -> Self {
		Self { buf }
	}
}

impl<B: BufMut> Write for Writer<B> {
	type Error = WriterError;

	fn write_slice(
		&mut self,
		buf: &[u8],
	) -> Result<usize, Self::Error> {
		if buf.is_empty() {
			return Ok(0);
		}

		let n = self.buf.copy_from_slice(buf);
		if n == 0 {
			return Err(WriterError::WriteZero);
		}

		Ok(n)
	}

	fn flush_once(&mut self) -> Result<(), Self::Error> {
		Ok(())
	}
}

impl<B: BufMut + Unpin> AsyncWrite for Writer<B> {
	type Error = WriterError;

	fn poll_write_slice(
		mut self: Pin<&mut Self>,
		_: &mut Context,
		buf: &[u8],
	) -> Poll<Result<usize, Self::Error>> {
		Poll::Ready(self.write_slice(buf))
	}

	fn poll_flush_once(
		self: Pin<&mut Self>,
		_: &mut Context,
	) -> Poll<Result<(), Self::Error>> {
		Poll::Ready(Ok(()))
	}
}

//! Non-atomic adapter types. Only safe for single threaded applications.
//!
//! So the following code will not compile.
//!
//! ```compile_fail
//! use channels::adapter::unsync::*;
//!
//! use std::io::{Read, Write, Cursor};
//! use std::thread;
//!
//! let rw = Cursor::new(vec![0u8; 32]);
//! let (mut r, mut w) = split(rw);
//!
//! thread::scope(|s| {
//!     s.spawn(|| {
//!         let _ = r.read(&mut []).unwrap();
//!     });
//!
//!     s.spawn(move || {
//!         let _ = w.write(&[]).unwrap();
//!     });
//! });
//! ```
#![allow(clippy::mut_from_ref)]

use core::cell::UnsafeCell;
use std::rc::Rc;

use crate::util::PhantomUnsend;

/// The read half of `T`. This half can **NOT** be sent to other
/// threads as it implements neither [`Send`] nor [`Sync`].
#[derive(Clone)]
pub struct ReadHalf<T> {
	inner: Rc<UnsafeCell<T>>,
	_marker: PhantomUnsend,
}

impl<T> ReadHalf<T> {
	fn inner_mut(&self) -> &mut T {
		unsafe { self.inner.get().as_mut().unwrap() }
	}

	/// Check whether `r` is the [`WriteHalf`] for this [`ReadHalf`].
	pub fn is_other(&self, w: &WriteHalf<T>) -> bool {
		are_same(self, w)
	}
}

/// The write half of `T`. This half can **NOT** be sent to other
/// threads as it implements neither [`Send`] nor [`Sync`].
#[derive(Clone)]
pub struct WriteHalf<T> {
	inner: Rc<UnsafeCell<T>>,
	_marker: PhantomUnsend,
}

impl<T> WriteHalf<T> {
	fn inner_mut(&mut self) -> &mut T {
		unsafe { self.inner.get().as_mut().unwrap() }
	}

	/// Check whether `r` is the [`ReadHalf`] for this [`WriteHalf`].
	pub fn is_other(&self, r: &ReadHalf<T>) -> bool {
		are_same(r, self)
	}
}

fn are_same<T>(r: &ReadHalf<T>, w: &WriteHalf<T>) -> bool {
	Rc::ptr_eq(&r.inner, &w.inner)
}

use std::io;

impl<T> io::Read for ReadHalf<T>
where
	T: io::Read,
{
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		self.inner_mut().read(buf)
	}
}

impl<T> io::Write for WriteHalf<T>
where
	T: io::Write,
{
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.inner_mut().write(buf)
	}

	fn flush(&mut self) -> io::Result<()> {
		self.inner_mut().flush()
	}
}

cfg_tokio! {
	use core::marker::Unpin;
	use core::pin::Pin;

	use std::task::{Context, Poll};

	use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

	impl<R> AsyncRead for ReadHalf<R>
	where
		R: AsyncRead + Unpin,
	{
		fn poll_read(
			self: Pin<&mut Self>,
			cx: &mut Context<'_>,
			buf: &mut ReadBuf<'_>,
		) -> Poll<io::Result<()>> {
			Pin::new(self.inner_mut()).poll_read(cx, buf)
		}
	}

	impl<W> AsyncWrite for ReadHalf<W>
	where
		W: AsyncWrite + Unpin,
	{
		fn poll_write(
			self: Pin<&mut Self>,
			cx: &mut Context<'_>,
			buf: &[u8],
		) -> Poll<io::Result<usize>> {
			Pin::new(self.inner_mut()).poll_write(cx, buf)
		}

		fn poll_flush(
			self: Pin<&mut Self>,
			cx: &mut Context<'_>,
		) -> Poll<io::Result<()>> {
			Pin::new(self.inner_mut()).poll_flush(cx)
		}

		fn poll_shutdown(
			self: Pin<&mut Self>,
			cx: &mut Context<'_>,
		) -> Poll<io::Result<()>> {
			Pin::new(self.inner_mut()).poll_shutdown(cx)
		}
	}
}

/// Split a type `T` to its 2 halves.
pub fn split<T>(rw: T) -> (ReadHalf<T>, WriteHalf<T>) {
	let rw = Rc::new(UnsafeCell::new(rw));

	(
		ReadHalf {
			inner: Rc::clone(&rw),
			_marker: PhantomUnsend::default(),
		},
		WriteHalf { inner: rw, _marker: PhantomUnsend::default() },
	)
}

/// Join back the 2 halves and return the original object passed to
/// the [`split`] function.
///
/// Returns ownership of the 2 halves if they could not be joined.
/// The only reason for this function returning `Err` is that `r`
/// and `w` were not made from the same `T`.
#[allow(clippy::missing_panics_doc)]
// This is needed because of crate lint level. This function does not actually panic.
pub fn join<T>(
	r: ReadHalf<T>,
	w: WriteHalf<T>,
) -> Result<T, (ReadHalf<T>, WriteHalf<T>)> {
	if !are_same(&r, &w) {
		return Err((r, w));
	}

	// `r` and `w` point to the same object, so we can safely join
	// them back

	// We must drop one half before attempting to destruct `Rc` so
	// that only one reference exists. See: [`Rc::into_inner`].
	drop(w);

	// This unwrap is safe because `into_inner` returns `None` if and
	// only if there are multiple strong references. We have dropped
	// the only other reference above and no other references could
	// have been made as the `Arc` field is inaccessible from external
	// code.
	let inner = Rc::into_inner(r.inner).unwrap();

	// Destruct the `UnsafeCell`
	let t = inner.into_inner();

	Ok(t)
}

//! Atomic adapter types. Safe for both single and multi-threaded
//! applications.
//!
//! Atomics have significant performance overhead over their [`unsync`]
//! counterparts.
//!
//! So the following code will compile.
//!
//! ```no_run
//! use channels::adapter::sync::*;
//!
//! use std::io::{Read, Write, Cursor};
//! use std::thread;
//!
//! let rw = Cursor::new(vec![0u8; 32]);
//!
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
//!
//! [`unsync`]: super::unsync

use std::sync::Arc;
use std::sync::{Mutex, MutexGuard};

/// The read half of `T`. This half can be sent to other threads as it
/// implements [`Send`] and [`Sync`].
#[derive(Clone)]
pub struct ReadHalf<T>(Arc<Mutex<T>>);

impl<T> ReadHalf<T> {
	fn inner_mut(&mut self) -> MutexGuard<T> {
		match self.0.lock() {
			Ok(v) => v,
			Err(e) => e.into_inner(),
		}
	}

	/// Check whether `r` is the [`WriteHalf`] for this [`ReadHalf`].
	pub fn is_other(&self, w: &WriteHalf<T>) -> bool {
		are_same(self, w)
	}
}

/// The write half of `T`. This half can be sent to other threads as it
/// implements [`Send`] and [`Sync`].
#[derive(Clone)]
pub struct WriteHalf<T>(Arc<Mutex<T>>);

impl<T> WriteHalf<T> {
	fn inner_mut(&mut self) -> MutexGuard<T> {
		match self.0.lock() {
			Ok(v) => v,
			Err(e) => e.into_inner(),
		}
	}

	/// Check whether `r` is the [`ReadHalf`] for this [`WriteHalf`].
	pub fn is_other(&self, r: &ReadHalf<T>) -> bool {
		are_same(r, self)
	}
}

fn are_same<T>(r: &ReadHalf<T>, w: &WriteHalf<T>) -> bool {
	Arc::ptr_eq(&r.0, &w.0)
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
			mut self: Pin<&mut Self>,
			cx: &mut Context<'_>,
			buf: &mut ReadBuf<'_>,
		) -> Poll<io::Result<()>> {
			let mut inner = self.inner_mut();
			Pin::new(&mut *inner).poll_read(cx, buf)
		}
	}

	impl<W> AsyncWrite for ReadHalf<W>
	where
		W: AsyncWrite + Unpin,
	{
		fn poll_write(
			mut self: Pin<&mut Self>,
			cx: &mut Context<'_>,
			buf: &[u8],
		) -> Poll<io::Result<usize>> {
			let mut inner = self.inner_mut();
			Pin::new(&mut *inner).poll_write(cx, buf)
		}

		fn poll_flush(
			mut self: Pin<&mut Self>,
			cx: &mut Context<'_>,
		) -> Poll<io::Result<()>> {
			let mut inner = self.inner_mut();
			Pin::new(&mut *inner).poll_flush(cx)
		}

		fn poll_shutdown(
			mut self: Pin<&mut Self>,
			cx: &mut Context<'_>,
		) -> Poll<io::Result<()>> {
			let mut inner = self.inner_mut();
			Pin::new(&mut *inner).poll_shutdown(cx)
		}
	}
}

/// Split a type `T` to its 2 halves.
pub fn split<T>(rw: T) -> (ReadHalf<T>, WriteHalf<T>) {
	let rw = Arc::new(Mutex::new(rw));

	(ReadHalf(Arc::clone(&rw)), WriteHalf(rw))
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

	// We must drop one half before attempting to destruct `Arc` so
	// that only one strong reference exists. See: [`Arc::into_inner`].
	drop(w);

	// This unwrap is safe because `into_inner` returns `None` if and
	// only if there are multiple strong references. We have dropped
	// the only other reference above and no other references could
	// have been made as the `Arc` field is inaccessible from external
	// code.
	let inner = Arc::into_inner(r.0).unwrap();

	// Destruct the mutex
	match inner.into_inner() {
		Ok(v) => Ok(v),
		Err(e) => Ok(e.into_inner()),
	}
}

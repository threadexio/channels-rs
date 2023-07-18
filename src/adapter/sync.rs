use std::sync::Arc;
use std::sync::{Mutex, MutexGuard};

use std::io::{Read, Result, Write};

/// The read half of `T`. This half can be sent to other threads as it
/// implements [`Send`] and [`Sync`].
#[derive(Clone)]
pub struct ReadHalf<T>(Arc<Mutex<T>>)
where
	T: Read;

impl<T> ReadHalf<T>
where
	T: Read,
{
	fn inner_mut(&mut self) -> MutexGuard<T> {
		match self.0.lock() {
			Ok(v) => v,
			Err(e) => e.into_inner(),
		}
	}
}

impl<T> Read for ReadHalf<T>
where
	T: Read,
{
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		self.inner_mut().read(buf)
	}
}

/// The write half of `T`. This half can be sent to other threads as it
/// implements [`Send`] and [`Sync`].
#[derive(Clone)]
pub struct WriteHalf<T>(Arc<Mutex<T>>)
where
	T: Write;

impl<T> WriteHalf<T>
where
	T: Write,
{
	fn inner_mut(&mut self) -> MutexGuard<T> {
		match self.0.lock() {
			Ok(v) => v,
			Err(e) => e.into_inner(),
		}
	}
}

impl<T> Write for WriteHalf<T>
where
	T: Write,
{
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		self.inner_mut().write(buf)
	}

	fn flush(&mut self) -> Result<()> {
		self.inner_mut().flush()
	}
}

/// Split a [`Read`] and [`Write`] type `t` to its 2 halves.
pub fn split<T>(rw: T) -> (ReadHalf<T>, WriteHalf<T>)
where
	T: Read + Write,
{
	let rw = Arc::new(Mutex::new(rw));

	(ReadHalf(Arc::clone(&rw)), WriteHalf(rw))
}

/// Join back the 2 halves and return the original object passed to
/// the [`split`] function. Returns `None` if `r` and `w` were not
/// obtained from the same [`split`] call.
#[allow(clippy::missing_panics_doc)]
// This is needed because of crate lint level. This function does not actually panic.
pub fn join<T>(r: ReadHalf<T>, w: WriteHalf<T>) -> Option<T>
where
	T: Read + Write,
{
	if !Arc::ptr_eq(&r.0, &w.0) {
		return None;
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
		Ok(v) => Some(v),
		Err(e) => Some(e.into_inner()),
	}
}

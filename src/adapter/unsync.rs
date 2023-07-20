#![allow(clippy::mut_from_ref)]
use core::cell::UnsafeCell;
use std::rc::Rc;

use std::io::{Read, Result, Write};

use crate::util::PhantomUnsend;

/// The read half of `T`. This half can **NOT** be sent to other
/// threads as it implements neither [`Send`] nor [`Sync`].
#[derive(Clone)]
pub struct ReadHalf<T>
where
	T: Read,
{
	inner: Rc<UnsafeCell<T>>,
	_marker: PhantomUnsend,
}

impl<T> ReadHalf<T>
where
	T: Read,
{
	fn inner_mut(&self) -> &mut T {
		unsafe { self.inner.get().as_mut().unwrap() }
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

/// The write half of `T`. This half can **NOT** be sent to other
/// threads as it implements neither [`Send`] nor [`Sync`].
#[derive(Clone)]
pub struct WriteHalf<T>
where
	T: Write,
{
	inner: Rc<UnsafeCell<T>>,
	_marker: PhantomUnsend,
}

impl<T> WriteHalf<T>
where
	T: Write,
{
	fn inner_mut(&mut self) -> &mut T {
		unsafe { self.inner.get().as_mut().unwrap() }
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
/// the [`split`] function. Returns `None` if `r` and `w` were not
/// obtained from the same [`split`] call.
#[allow(clippy::missing_panics_doc)]
// This is needed because of crate lint level. This function does not actually panic.
pub fn join<T>(r: ReadHalf<T>, w: WriteHalf<T>) -> Option<T>
where
	T: Read + Write,
{
	if !Rc::ptr_eq(&r.inner, &w.inner) {
		return None;
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

	Some(t)
}

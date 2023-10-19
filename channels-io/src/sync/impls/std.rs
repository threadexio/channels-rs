use core::task::Poll;

use crate::buf::{IoSliceMut, IoSliceRef};
use crate::util::newtype;
use crate::{IntoReader, IntoWriter, Read, Write};

use std::io;

newtype! { StdWrite for: io::Write }

impl<T: io::Write> IntoWriter<StdWrite<T>> for T {
	fn into_writer(self) -> StdWrite<T> {
		StdWrite(self)
	}
}

impl<T: io::Write> Write for StdWrite<T> {
	type Error = io::Error;

	fn write_all(
		&mut self,
		buf: &mut IoSliceRef,
	) -> Poll<Result<(), Self::Error>> {
		use io::ErrorKind as E;

		while !buf.is_empty() {
			match self.0.write(buf) {
				Ok(0) => {
					return Poll::Ready(Err(E::WriteZero.into()))
				},
				Ok(n) => buf.advance(n),
				Err(e) if e.kind() == E::Interrupted => continue,
				Err(e) if e.kind() == E::WouldBlock => {
					return Poll::Pending
				},
				Err(e) => return Poll::Ready(Err(e)),
			}
		}

		Poll::Ready(Ok(()))
	}

	fn flush(&mut self) -> Poll<Result<(), Self::Error>> {
		use io::ErrorKind as E;

		match self.0.flush() {
			Ok(()) => Poll::Ready(Ok(())),
			Err(e) if e.kind() == E::WouldBlock => Poll::Pending,
			Err(e) => Poll::Ready(Err(e)),
		}
	}
}

newtype! { StdRead for: io::Read }

impl<T: io::Read> IntoReader<StdRead<T>> for T {
	fn into_reader(self) -> StdRead<T> {
		StdRead(self)
	}
}

impl<T: io::Read> Read for StdRead<T> {
	type Error = io::Error;

	fn read_all(
		&mut self,
		buf: &mut IoSliceMut,
	) -> Poll<Result<(), Self::Error>> {
		use io::ErrorKind as E;

		while !buf.is_empty() {
			match self.0.read(buf) {
				Ok(0) => {
					return Poll::Ready(Err(E::UnexpectedEof.into()))
				},
				Ok(n) => buf.advance(n),
				Err(e) if e.kind() == E::Interrupted => continue,
				Err(e) if e.kind() == E::WouldBlock => {
					return Poll::Pending
				},
				Err(e) => return Poll::Ready(Err(e)),
			}
		}

		Poll::Ready(Ok(()))
	}
}

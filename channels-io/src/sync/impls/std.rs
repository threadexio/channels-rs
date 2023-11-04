use core::task::Poll;

use crate::{Buf, BufMut, IntoReader, IntoWriter, Read, Write};

use std::io;

crate::util::newtype! {
	/// IO wrapper for the [`mod@std`] traits.
	StdIo for:
}

impl<T: io::Write> IntoWriter<StdIo<T>> for T {
	fn into_writer(self) -> StdIo<T> {
		StdIo(self)
	}
}

impl<T: io::Write> Write for StdIo<T> {
	type Error = io::Error;

	fn write_all(
		&mut self,
		mut buf: impl Buf,
	) -> Poll<Result<(), Self::Error>> {
		use io::ErrorKind as E;

		while buf.has_remaining() {
			match self.0.write(buf.unfilled()) {
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

impl<T: io::Read> IntoReader<StdIo<T>> for T {
	fn into_reader(self) -> StdIo<T> {
		StdIo(self)
	}
}

impl<T: io::Read> Read for StdIo<T> {
	type Error = io::Error;

	fn read_all(
		&mut self,
		mut buf: impl BufMut,
	) -> Poll<Result<(), Self::Error>> {
		use io::ErrorKind as E;

		while buf.has_remaining_mut() {
			match self.0.read(buf.unfilled_mut()) {
				Ok(0) => {
					return Poll::Ready(Err(E::UnexpectedEof.into()))
				},
				Ok(n) => buf.advance_mut(n),
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

use core::task::Poll;

use crate::util::newtype;
use crate::{Buf, BufMut};
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
		mut buf: impl Buf,
	) -> Poll<Result<(), Self::Error>> {
		use io::ErrorKind as E;

		while !buf.has_remaining() {
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
		mut buf: impl BufMut,
	) -> Poll<Result<(), Self::Error>> {
		use io::ErrorKind as E;

		while !buf.has_remaining_mut() {
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

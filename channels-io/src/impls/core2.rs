use super::prelude::*;

use ::core2::io::ErrorKind as E;

impl ReadError for ::core2::io::Error {
	fn eof() -> Self {
		Self::from(E::UnexpectedEof)
	}

	fn should_retry(&self) -> bool {
		self.kind() == E::UnexpectedEof
	}
}

newtype! {
	/// Wrapper IO type for [`core2::io::Read`] and [`core2::io::Write`].
	Core2
}

impl_newtype_read! { Core2: ::core2::io::Read }

impl<T> Read for Core2<T>
where
	T: ::core2::io::Read,
{
	type Error = ::core2::io::Error;

	fn read_slice(
		&mut self,
		buf: &mut [u8],
	) -> Result<usize, Self::Error> {
		self.0.read(buf)
	}
}

impl_newtype_write! { Core2: ::core2::io::Write }

impl<T> Write for Core2<T>
where
	T: ::core2::io::Write,
{
	type Error = ::core2::io::Error;

	fn write(&mut self, mut buf: &[u8]) -> Result<(), Self::Error> {
		while !buf.is_empty() {
			match self.0.write(buf) {
				Ok(i) => buf = &buf[i..],
				Err(e) if e.kind() == E::Interrupted => continue,
				Err(e) => return Err(e),
			}
		}

		Ok(())
	}

	fn flush(&mut self) -> Result<(), Self::Error> {
		loop {
			match self.0.flush() {
				Ok(()) => break Ok(()),
				Err(e) if e.kind() == E::Interrupted => continue,
				Err(e) => break Err(e),
			}
		}
	}
}

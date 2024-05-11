use super::prelude::*;

use ::std::io::ErrorKind as E;

impl ReadError for ::std::io::Error {
	fn eof() -> Self {
		Self::from(E::UnexpectedEof)
	}

	fn should_retry(&self) -> bool {
		self.kind() == E::Interrupted
	}
}

newtype! {
	/// Wrapper IO type for [`std::io::Read`] and [`std::io::Write`].
	Std
}

impl_newtype_read! { Std: ::std::io::Read }

impl<T> Read for Std<T>
where
	T: ::std::io::Read,
{
	type Error = ::std::io::Error;

	fn read_slice(
		&mut self,
		buf: &mut [u8],
	) -> Result<usize, Self::Error> {
		self.0.read(buf)
	}
}

impl_newtype_write! { Std: ::std::io::Write }

impl<T> Write for Std<T>
where
	T: ::std::io::Write,
{
	type Error = ::std::io::Error;

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

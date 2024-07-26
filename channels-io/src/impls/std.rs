use super::prelude::*;

use ::std::io::ErrorKind as E;

impl IoError for ::std::io::Error {
	fn should_retry(&self) -> bool {
		self.kind() == E::Interrupted
	}
}

impl ReadError for ::std::io::Error {
	fn eof() -> Self {
		Self::from(E::UnexpectedEof)
	}
}

impl WriteError for ::std::io::Error {
	fn write_zero() -> Self {
		Self::from(E::WriteZero)
	}
}

/// Wrapper IO type for [`std::io::Read`] and [`std::io::Write`].
#[derive(Debug)]
pub struct Std<T>(pub T);

impl_newtype! { Std }

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

	fn write_slice(
		&mut self,
		buf: &[u8],
	) -> Result<usize, Self::Error> {
		self.0.write(buf)
	}

	fn flush_once(&mut self) -> Result<(), Self::Error> {
		self.0.flush()
	}
}

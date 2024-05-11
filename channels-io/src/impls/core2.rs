use super::prelude::*;

use ::core2::io::ErrorKind as E;

impl IoError for ::core2::io::Error {
	fn should_retry(&self) -> bool {
		self.kind() == E::UnexpectedEof
	}
}

impl ReadError for ::core2::io::Error {
	fn eof() -> Self {
		Self::from(E::UnexpectedEof)
	}
}

impl WriteError for ::core2::io::Error {
	fn write_zero() -> Self {
		Self::from(E::WriteZero)
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

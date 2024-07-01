use super::prelude::*;

use ::embedded_io::{Error as _, ErrorKind};

impl IoError for ErrorKind {
	fn should_retry(&self) -> bool {
		self.kind() == ErrorKind::Interrupted
	}
}

impl ReadError for ErrorKind {
	fn eof() -> Self {
		ErrorKind::BrokenPipe
	}
}

impl WriteError for ErrorKind {
	fn write_zero() -> Self {
		ErrorKind::WriteZero
	}
}

newtype! {
	/// Wrapper IO type for [`embedded_io::Read`] and [`embedded_io::Write`].
	///
	/// [`embedded_io::Read`]: ::embedded_io::Read
	/// [`embedded_io::Write`]: ::embedded_io::Write
	EmbeddedIo
}

impl_newtype_read! { EmbeddedIo: ::embedded_io::Read }

impl<T> Read for EmbeddedIo<T>
where
	T: ::embedded_io::Read,
{
	type Error = ErrorKind;

	fn read_slice(
		&mut self,
		buf: &mut [u8],
	) -> Result<usize, Self::Error> {
		self.0.read(buf).map_err(|x| x.kind())
	}
}

impl_newtype_write! { EmbeddedIo: ::embedded_io::Write }

impl<T> Write for EmbeddedIo<T>
where
	T: ::embedded_io::Write,
{
	type Error = ErrorKind;

	fn write_slice(
		&mut self,
		buf: &[u8],
	) -> Result<usize, Self::Error> {
		self.0.write(buf).map_err(|x| x.kind())
	}

	fn flush_once(&mut self) -> Result<(), Self::Error> {
		self.0.flush().map_err(|x| x.kind())
	}
}

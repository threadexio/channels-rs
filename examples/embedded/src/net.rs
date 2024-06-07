use channels::io::error::{IoError, ReadError, WriteError};
use channels::io::{Read, Write};

/*
 * Implementation of networking varies between environments.
 * For this reason, and also because it is out of scope, the
 * actual implementation is left out.
 */

#[derive(Debug)]
pub struct SocketError {}

impl IoError for SocketError {
	fn should_retry(&self) -> bool {
		unimplemented!()
	}
}

impl ReadError for SocketError {
	fn eof() -> Self {
		unimplemented!()
	}
}

impl WriteError for SocketError {
	fn write_zero() -> Self {
		unimplemented!()
	}
}

#[derive(Debug)]
pub struct Socket {}

impl Socket {
	/// Create a new socket on `addr`.
	pub fn new(_addr: &str) -> Self {
		Self {}
	}

	/// Duplicate the handle of the socket allowing IO to occur from either one.
	pub fn dup(&self) -> Socket {
		unimplemented!()
	}

	/// Join back the handle obtained by [`Socket::dup()`].
	pub fn join(&self, _other: Socket) {
		unimplemented!()
	}
}

impl Read for Socket {
	type Error = SocketError;

	fn read_slice(
		&mut self,
		_: &mut [u8],
	) -> Result<usize, Self::Error> {
		unimplemented!()
	}
}

impl Write for Socket {
	type Error = SocketError;

	fn write_slice(
		&mut self,
		_: &[u8],
	) -> Result<usize, Self::Error> {
		unimplemented!()
	}

	fn flush_once(&mut self) -> Result<(), Self::Error> {
		unimplemented!()
	}
}

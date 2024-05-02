use channels::io::{Read, Write};

/*
 * Implementation of networking varies between environments.
 * For this reason, and also because it is out of scope, the
 * actual implementation is left out.
 */

#[derive(Debug)]
pub struct SocketError {}

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

	fn read(&mut self, _buf: &mut [u8]) -> Result<(), Self::Error> {
		unimplemented!()
	}
}

impl Write for Socket {
	type Error = SocketError;

	fn write(&mut self, _buf: &[u8]) -> Result<(), Self::Error> {
		unimplemented!()
	}

	fn flush(&mut self) -> Result<(), Self::Error> {
		unimplemented!()
	}
}

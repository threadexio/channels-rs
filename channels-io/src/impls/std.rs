use super::prelude::*;

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

	fn read(
		&mut self,
		mut buf: &mut [u8],
	) -> Result<(), Self::Error> {
		while !buf.is_empty() {
			use ::std::io::ErrorKind as E;
			match self.0.read(buf) {
				Ok(i) => buf = &mut buf[i..],
				Err(e) if e.kind() == E::Interrupted => continue,
				Err(e) => return Err(e),
			}
		}

		Ok(())
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
			use ::std::io::ErrorKind as E;
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
			use ::std::io::ErrorKind as E;
			match self.0.flush() {
				Ok(()) => break Ok(()),
				Err(e) if e.kind() == E::Interrupted => continue,
				Err(e) => break Err(e),
			}
		}
	}
}

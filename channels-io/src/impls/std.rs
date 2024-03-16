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

	fn read<B>(&mut self, mut buf: B) -> Result<(), Self::Error>
	where
		B: ContiguousMut,
	{
		while buf.has_remaining_mut() {
			use ::std::io::ErrorKind as E;
			match self.0.read(buf.chunk_mut()) {
				Ok(i) => buf.advance_mut(i),
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

	fn write<B>(&mut self, mut buf: B) -> Result<(), Self::Error>
	where
		B: Contiguous,
	{
		while buf.has_remaining() {
			use ::std::io::ErrorKind as E;
			match self.0.write(buf.chunk()) {
				Ok(i) => buf.advance(i),
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

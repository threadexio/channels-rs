use super::prelude::*;

newtype! {
	/// Wrapper IO type for [`core2::`].
	Core2
}

impl_newtype_read! { Core2: ::core2::io::Read }

impl<T> Read for Core2<T>
where
	T: ::core2::io::Read,
{
	type Error = ::core2::io::Error;

	fn read<B: ContiguousMut>(
		&mut self,
		mut buf: B,
	) -> Result<(), Self::Error> {
		while buf.has_remaining_mut() {
			use ::core2::io::ErrorKind as E;
			match self.0.read(buf.chunk_mut()) {
				Ok(i) => buf.advance_mut(i),
				Err(e) if e.kind() == E::Interrupted => continue,
				Err(e) => return Err(e),
			}
		}

		Ok(())
	}
}

impl_newtype_write! { Core2: ::core2::io::Write }

impl<T> Write for Core2<T>
where
	T: ::core2::io::Write,
{
	type Error = ::core2::io::Error;

	fn write<B: Contiguous>(
		&mut self,
		mut buf: B,
	) -> Result<(), Self::Error> {
		while buf.has_remaining() {
			use ::core2::io::ErrorKind as E;
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
			use ::core2::io::ErrorKind as E;
			match self.0.flush() {
				Ok(()) => break Ok(()),
				Err(e) if e.kind() == E::Interrupted => continue,
				Err(e) => break Err(e),
			}
		}
	}
}

use super::prelude::*;

newtype! {
	/// Wrapper IO type for [`acid_io::Read`] and [`acid_io::Write`].
	AcidIo
}

impl_newtype_read! { AcidIo: ::acid_io::Read }

impl<T> Read for AcidIo<T>
where
	T: ::acid_io::Read,
{
	type Error = ::acid_io::Error;

	fn read<B: ContiguousMut>(
		&mut self,
		mut buf: B,
	) -> Result<(), Self::Error> {
		while buf.has_remaining_mut() {
			use ::acid_io::ErrorKind as E;
			match self.0.read(buf.chunk_mut()) {
				Ok(i) => buf.advance_mut(i),
				Err(e) if e.kind() == E::Interrupted => continue,
				Err(e) => return Err(e),
			}
		}

		Ok(())
	}
}

impl_newtype_write! { AcidIo: ::acid_io::Write }

impl<T> Write for AcidIo<T>
where
	T: ::acid_io::Write,
{
	type Error = ::acid_io::Error;

	fn write<B: Contiguous>(
		&mut self,
		mut buf: B,
	) -> Result<(), Self::Error> {
		while buf.has_remaining() {
			use ::acid_io::ErrorKind as E;
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
			use ::acid_io::ErrorKind as E;
			match self.0.flush() {
				Ok(()) => break Ok(()),
				Err(e) if e.kind() == E::Interrupted => continue,
				Err(e) => return Err(e),
			}
		}
	}
}

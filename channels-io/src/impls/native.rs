use super::prelude::*;

newtype! {
	/// Wrapper IO type for [`Read`] and [`Write`].
	Native
}

impl_newtype_read! { Native: Read }

impl<T> Read for Native<T>
where
	T: Read,
{
	type Error = T::Error;

	fn read<B>(&mut self, buf: B) -> Result<(), Self::Error>
	where
		B: ContiguousMut,
	{
		self.0.read(buf)
	}
}

impl_newtype_write! { Native: Write }

impl<T> Write for Native<T>
where
	T: Write,
{
	type Error = T::Error;

	fn write<B>(&mut self, buf: B) -> Result<(), Self::Error>
	where
		B: Contiguous,
	{
		self.0.write(buf)
	}

	fn flush(&mut self) -> Result<(), Self::Error> {
		self.0.flush()
	}
}

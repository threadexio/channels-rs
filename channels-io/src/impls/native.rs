use super::prelude::*;

use crate::util::Future;

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

	fn read<B: ContiguousMut>(
		&mut self,
		buf: B,
	) -> Result<(), Self::Error> {
		self.0.read(buf)
	}
}

impl_newtype_write! { Native: Write }

impl<T> Write for Native<T>
where
	T: Write,
{
	type Error = T::Error;

	fn write<B: Contiguous>(
		&mut self,
		buf: B,
	) -> Result<(), Self::Error> {
		self.0.write(buf)
	}

	fn flush(&mut self) -> Result<(), Self::Error> {
		self.0.flush()
	}
}

newtype! {
	/// Wrapper IO type for [`AsyncRead`] and [`AsyncWrite`].
	NativeAsync
}

impl_newtype_read! { NativeAsync: AsyncRead }

impl<T> AsyncRead for NativeAsync<T>
where
	T: AsyncRead,
{
	type Error = T::Error;

	fn read<B: ContiguousMut>(
		&mut self,
		buf: B,
	) -> impl Future<Output = Result<(), Self::Error>> {
		self.0.read(buf)
	}
}

impl_newtype_write! { NativeAsync: AsyncWrite }

impl<T> AsyncWrite for NativeAsync<T>
where
	T: AsyncWrite,
{
	type Error = T::Error;

	fn write<B: Contiguous>(
		&mut self,
		buf: B,
	) -> impl Future<Output = Result<(), Self::Error>> {
		self.0.write(buf)
	}

	fn flush(
		&mut self,
	) -> impl Future<Output = Result<(), Self::Error>> {
		self.0.flush()
	}
}

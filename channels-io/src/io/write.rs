use crate::buf::Contiguous;

use super::future;

pub trait Write {
	type Error;

	fn write<B>(&mut self, buf: B) -> Result<(), Self::Error>
	where
		B: Contiguous;

	fn flush(&mut self) -> Result<(), Self::Error>;
}

pub trait AsyncWrite {
	type Error;

	fn write<B>(
		&mut self,
		buf: B,
	) -> future! { Result<(), Self::Error> }
	where
		B: Contiguous;

	fn flush(&mut self) -> future! { Result<(), Self::Error> };
}

pub trait Writer {
	type Inner;

	fn get(&self) -> &Self::Inner;
	fn get_mut(&mut self) -> &mut Self::Inner;
	fn into_inner(self) -> Self::Inner;
}

pub trait IntoWriter<T> {
	fn into_writer(self) -> T;
}

macro_rules! forward_impl_write {
	($typ:ty) => {
		type Error = <$typ>::Error;

		fn write<B>(&mut self, buf: B) -> Result<(), Self::Error>
		where
			B: Contiguous,
		{
			(**self).write(buf)
		}

		fn flush(&mut self) -> Result<(), Self::Error> {
			(**self).flush()
		}
	};
}

macro_rules! forward_impl_async_write {
	($typ:ty) => {
		type Error = <$typ>::Error;

		async fn write<B>(
			&mut self,
			buf: B,
		) -> Result<(), Self::Error>
		where
			B: Contiguous,
		{
			(**self).write(buf).await
		}

		async fn flush(&mut self) -> Result<(), Self::Error> {
			(**self).flush().await
		}
	};
}

macro_rules! forward_impl_all_write {
	($typ:ty) => {
		impl<T: Write> Write for $typ {
			forward_impl_write!(T);
		}

		impl<T: AsyncWrite> AsyncWrite for $typ {
			forward_impl_async_write!(T);
		}
	};
}

forward_impl_all_write! { &mut T }

#[cfg(feature = "alloc")]
mod alloc_impls {
	use super::*;

	forward_impl_all_write! { alloc::boxed::Box<T> }
}

use crate::buf::ContiguousMut;

use super::future;

pub trait Read {
	type Error;

	fn read<B>(&mut self, buf: B) -> Result<(), Self::Error>
	where
		B: ContiguousMut;
}

pub trait AsyncRead: Send {
	type Error;

	fn read<B>(
		&mut self,
		buf: B,
	) -> future! { Result<(), Self::Error> }
	where
		B: ContiguousMut;
}

pub trait Reader {
	type Inner;

	fn get(&self) -> &Self::Inner;
	fn get_mut(&mut self) -> &mut Self::Inner;
	fn into_inner(self) -> Self::Inner;
}

pub trait IntoReader<T> {
	fn into_reader(self) -> T;
}

macro_rules! forward_impl_read {
	($typ:ty) => {
		type Error = <$typ>::Error;

		fn read<B: ContiguousMut>(
			&mut self,
			buf: B,
		) -> Result<(), Self::Error> {
			(**self).read(buf)
		}
	};
}

macro_rules! forward_impl_async_read {
	($typ:ty) => {
		type Error = <$typ>::Error;

		async fn read<B: ContiguousMut>(
			&mut self,
			buf: B,
		) -> Result<(), Self::Error> {
			(**self).read(buf).await
		}
	};
}

macro_rules! forward_impl_all_read {
	($typ:ty) => {
		impl<T: Read> Read for $typ {
			forward_impl_read!(T);
		}

		impl<T: AsyncRead> AsyncRead for $typ {
			forward_impl_async_read!(T);
		}
	};
}

forward_impl_all_read! { &mut T }

#[cfg(feature = "alloc")]
mod alloc_impls {
	use super::*;

	forward_impl_all_read! { alloc::boxed::Box<T> }
}

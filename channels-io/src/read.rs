use crate::util::Future;
use crate::ContiguousMut;

/// This trait allows reading bytes from a source.
///
/// Types implementing this trait are called "readers".
///
/// Readers have only one method, [`Read::read()`] which will attempt to read some
/// bytes into the provided buffer.
pub trait Read {
	/// Error type for [`Read::read()`].
	type Error;

	/// Read some bytes into `buf`.
	///
	/// Upon return, this function must guarantee that either: a) `buf` has no
	/// more space to fill, `buf.has_remaining_mut()` should return `false`. b)
	/// an error has occurred that _cannot_ be handled immediately.
	///
	/// If `buf` has been filled with data, then this function must return with
	/// [`Ok((())`](Ok). In any other case it must return an [`Err`].
	fn read<B>(&mut self, buf: B) -> Result<(), Self::Error>
	where
		B: ContiguousMut;
}

/// This trait is the asynchronous version of [`Read`].
pub trait AsyncRead: Send {
	/// Error type for [`AsyncRead::read()`].
	type Error;

	/// Asynchronously read some bytes into `buf`.
	///
	/// This function behaves in the same way as [`Read::read()`] except that it
	/// returns a [`Future`] that must be `.await`ed.
	///
	/// [`Future`]: core::future::Future
	fn read<B>(
		&mut self,
		buf: B,
	) -> impl Future<Output = Result<(), Self::Error>>
	where
		B: ContiguousMut;
}

/// Convert a type to a reader.
///
/// This trait is how functions can accept different readers under one unified
/// interface. It is very flexible, allowing code to be agnostic over synchronous
/// or asynchronous readers and/or different interfaces.
///
/// The trait consists of only one method [`IntoReader::into_reader()`]. The
/// purpose of this method is to wrap any type `T` with its appropriate wrapper
/// type so that it can implement [`Read`] and/or [`AsyncRead`]. This is necessary
/// because we can't implement a trait directly for every type `T` multiple times
/// with different trait bounds. Which basically means we cannot do this:
///
/// ```rust,compile_fail
/// trait AsyncRead {
///     // -- snip --
/// }
///
/// impl<T> AsyncRead for T
/// where
///     T: tokio::io::AsyncRead
/// {
///     // -- snip --
/// }
///
/// impl<T> AsyncRead for T
/// where
///     T: futures::AsyncRead
/// {
///     // -- snip --
/// }
/// ```
///
/// We _can_ solve this problem though. The Rust book [recommends using the
/// "newtype" pattern to solve this](https://doc.rust-lang.org/book/ch19-03-advanced-traits.html#using-the-newtype-pattern-to-implement-external-traits-on-external-types).
/// To do this we must wrap the type `T` in a new type for which then we implement
/// the desired trait. This trait is what wraps a `T` with the new type.
///
/// # Examples
///
/// - Accepting any reader.
///
/// ```rust,no_run
/// use channels_io::{IntoReader, AsyncRead, Read};
///
/// struct MyStruct<R> {
///     reader: R
/// }
///
/// impl<R> MyStruct<R> {
///     pub fn new(reader: impl IntoReader<R>) -> Self {
///         Self {
///             reader: reader.into_reader()
///         }
///     }
/// }
///
/// impl<R: Read> MyStruct<R> {
///     // implement things for when the reader is synchronous
/// }
///
/// impl<R: AsyncRead> MyStruct<R> {
///     // implement things for when the reader is asynchronous
/// }
///
/// // With a synchronous reader.
/// let _ = MyStruct::new(std::io::empty());
///
/// // With an asynchronous reader.
/// let _ = MyStruct::new(tokio::io::empty());
/// ```
///
/// - Accepting synchronous/asynchronous readers only.
///
/// ```rust,no_run
/// use channels_io::{IntoReader, AsyncRead, Read};
///
/// fn sync_only<R: Read>(reader: impl IntoReader<R>) { /* ... */ }
/// fn async_only<R: AsyncRead>(reader: impl IntoReader<R>) { /* ... */ }
///
/// let _ = sync_only(std::io::empty());
/// let _ = async_only(tokio::io::empty());
/// ```
///
/// So the following cannot work:
///
/// ```rust,compile_fail
/// use channels_io::{IntoReader, Read};
///
/// fn sync_only<R: Read>(reader: impl IntoReader<R>) { /* ... */ }
///
/// let _ = sync_only(tokio::io::empty());
/// ```
///
/// ```rust,compile_fail
/// use channels_io::{IntoReader, AsyncRead};
///
/// fn async_only<R: AsyncRead>(reader: impl IntoReader<R>) { /* ... */ }
///
/// let _ = async_only(std::io::empty());
/// ```
pub trait IntoReader<T> {
	/// Convert `self` to a reader `T`.
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
		impl<T: $crate::Read> $crate::Read for $typ {
			forward_impl_read!(T);
		}

		impl<T: $crate::AsyncRead> $crate::AsyncRead for $typ {
			forward_impl_async_read!(T);
		}
	};
}

forward_impl_all_read! { &mut T }

#[cfg(feature = "alloc")]
forward_impl_all_read! { alloc::boxed::Box<T> }

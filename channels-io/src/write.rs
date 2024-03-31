use crate::buf::Contiguous;
use crate::util::Future;

/// This trait allows writing bytes to a writer.
///
/// Types implementing this trait are called "writers".
///
/// Writers have only one method, [`Write::write()`] which will attempt to write
/// some bytes to the sink.
pub trait Write {
	/// Error type for [`Write::write()`].
	type Error;

	/// Write `buf` to the writer.
	///
	/// Upon return, this function must guarantee that either: a) `buf` has been
	/// fully written, `buf.has_remaining()` should return `false`. b)
	/// an error has occurred that _cannot_ be handled immediately.
	///
	/// If `buf` has been written to the writer, then this function must return
	/// with [`Ok(())`](Ok). In any other case it must return an [`Err`].
	fn write<B>(&mut self, buf: B) -> Result<(), Self::Error>
	where
		B: Contiguous;

	/// Flush this writer ensuring all bytes reach their destination.
	///
	/// Upon return, this function must ensure that all bytes written to the
	/// writer by previous calls to [`Write::write()`] have reached their
	/// destination.
	fn flush(&mut self) -> Result<(), Self::Error>;
}

/// This trait is the asynchronous version of [`Write`].
pub trait AsyncWrite {
	/// Error type for [`AsyncWrite::write()`].
	type Error;

	/// Asynchronously write `buf` to the writer.
	///
	/// This function behaves in the same way as [`Write::write()`] except that
	/// it returns a [`Future`] that must be `.await`ed.
	///
	/// [`Future`]: core::future::Future
	fn write<B>(
		&mut self,
		buf: B,
	) -> impl Future<Output = Result<(), Self::Error>>
	where
		B: Contiguous;

	/// Asynchronously flush the writer.
	///
	/// This function behaves in the same way as [`Write::flush()`] except that
	/// it returns a [`Future`] that must be `.await`ed.
	///
	/// [`Future`]: core::future::Future
	fn flush(
		&mut self,
	) -> impl Future<Output = Result<(), Self::Error>>;
}

/// Convert a type to a writer.
///
/// This trait is how functions can accept different writers under one unified
/// interface. It is very flexible, allowing code to be agnostic over synchronous
/// or asynchronous writers and/or different interfaces.
///
/// The trait consists of only one method [`IntoWriter::into_writer()`]. The
/// purpose of this method is to wrap any type `T` with its appropriate wrapper
/// type so that it can implement [`Write`] and/or [`AsyncWrite`]. This is necessary
/// because we can't implement a trait directly for every type `T` multiple times
/// with different trait bounds. Which basically means we cannot do this:
///
/// ```rust,compile_fail
/// trait AsyncWrite {
///     // -- snip --
/// }
///
/// impl<T> AsyncWrite for T
/// where
///     T: tokio::io::AsyncWrite
/// {
///     // -- snip --
/// }
///
/// impl<T> AsyncWrite for T
/// where
///     T: futures::AsyncWrite
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
/// - Accepting any writer.
///
/// ```rust,no_run
/// use channels_io::{IntoWriter, AsyncWrite, Write};
///
/// struct MyStruct<R> {
///     writer: R
/// }
///
/// impl<R> MyStruct<R> {
///     pub fn new(writer: impl IntoWriter<R>) -> Self {
///         Self {
///             writer: writer.into_writer()
///         }
///     }
/// }
///
/// impl<R: Write> MyStruct<R> {
///     // implement things for when the writer is synchronous
/// }
///
/// impl<R: AsyncWrite> MyStruct<R> {
///     // implement things for when the writer is asynchronous
/// }
///
/// // With a synchronous writer.
/// let _ = MyStruct::new(std::io::empty());
///
/// // With an asynchronous writer.
/// let _ = MyStruct::new(tokio::io::empty());
/// ```
///
/// - Accepting synchronous/asynchronous readers only.
///
/// ```rust,no_run
/// use channels_io::{IntoWriter, AsyncWrite, Write};
///
/// fn sync_only<W: Write>(writer: impl IntoWriter<W>) { /* ... */ }
/// fn async_only<W: AsyncWrite>(writer: impl IntoWriter<W>) { /* ... */ }
///
/// let _ = sync_only(std::io::sink());
/// let _ = async_only(tokio::io::sink());
/// ```
///
/// So the following cannot work:
///
/// ```rust,compile_fail
/// use channels_io::{IntoWriter, Write};
///
/// fn sync_only<W: Write>(writer: impl IntoWriter<W>) { /* ... */ }
///
/// let _ = sync_only(tokio::io::empty());
/// ```
///
/// ```rust,compile_fail
/// use channels_io::{IntoWriter, AsyncWrite};
///
/// fn async_only<W: AsyncWrite>(writer: impl IntoWriter<W>) { /* ... */ }
///
/// let _ = async_only(std::io::empty());
/// ```
pub trait IntoWriter<T> {
	/// Convert `self` to a writer `T`.
	fn into_writer(self) -> T;
}

impl<T: Write> IntoWriter<T> for T {
	fn into_writer(self) -> T {
		self
	}
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
		impl<T: $crate::Write> $crate::Write for $typ {
			forward_impl_write!(T);
		}

		impl<T: $crate::AsyncWrite> $crate::AsyncWrite for $typ {
			forward_impl_async_write!(T);
		}
	};
}

forward_impl_all_write! { &mut T }

#[cfg(feature = "alloc")]
forward_impl_all_write! { alloc::boxed::Box<T> }

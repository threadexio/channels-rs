/// Convert a type to a reader.
///
/// This trait is how functions can accept different readers under one unified
/// interface. It is very flexible, allowing code to be agnostic over synchronous
/// or asynchronous readers and/or different interfaces.
///
/// The trait consists of only one method [`IntoRead::into_read()`]. The
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
/// use channels_io::{IntoRead, AsyncRead, Read};
///
/// struct MyStruct<R> {
///     reader: R
/// }
///
/// impl<R> MyStruct<R> {
///     pub fn new(reader: impl IntoRead<R>) -> Self {
///         Self {
///             reader: reader.into_read()
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
/// use channels_io::{IntoRead, AsyncRead, Read};
///
/// fn sync_only<R: Read>(reader: impl IntoRead<R>) { /* ... */ }
/// fn async_only<R: AsyncRead>(reader: impl IntoRead<R>) { /* ... */ }
///
/// let _ = sync_only(std::io::empty());
/// let _ = async_only(tokio::io::empty());
/// ```
///
/// So the following cannot work:
///
/// ```rust,compile_fail
/// use channels_io::{IntoRead, Read};
///
/// fn sync_only<R: Read>(reader: impl IntoRead<R>) { /* ... */ }
///
/// let _ = sync_only(tokio::io::empty());
/// ```
///
/// ```rust,compile_fail
/// use channels_io::{IntoRead, AsyncRead};
///
/// fn async_only<R: AsyncRead>(reader: impl IntoRead<R>) { /* ... */ }
///
/// let _ = async_only(std::io::empty());
/// ```
///
/// [`Read`]: crate::Read
/// [`AsyncRead`]: crate::AsyncRead
pub trait IntoRead<T> {
	/// Convert `self` to a reader `T`.
	fn into_read(self) -> T;
}

/// Convert a type to a writer.
///
/// This trait is how functions can accept different writers under one unified
/// interface. It is very flexible, allowing code to be agnostic over synchronous
/// or asynchronous writers and/or different interfaces.
///
/// The trait consists of only one method [`IntoWrite::into_write()`]. The
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
/// use channels_io::{IntoWrite, AsyncWrite, Write};
///
/// struct MyStruct<R> {
///     writer: R
/// }
///
/// impl<R> MyStruct<R> {
///     pub fn new(writer: impl IntoWrite<R>) -> Self {
///         Self {
///             writer: writer.into_write()
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
/// use channels_io::{IntoWrite, AsyncWrite, Write};
///
/// fn sync_only<W: Write>(writer: impl IntoWrite<W>) { /* ... */ }
/// fn async_only<W: AsyncWrite>(writer: impl IntoWrite<W>) { /* ... */ }
///
/// let _ = sync_only(std::io::sink());
/// let _ = async_only(tokio::io::sink());
/// ```
///
/// So the following cannot work:
///
/// ```rust,compile_fail
/// use channels_io::{IntoWrite, Write};
///
/// fn sync_only<W: Write>(writer: impl IntoWrite<W>) { /* ... */ }
///
/// let _ = sync_only(tokio::io::empty());
/// ```
///
/// ```rust,compile_fail
/// use channels_io::{IntoWrite, AsyncWrite};
///
/// fn async_only<W: AsyncWrite>(writer: impl IntoWrite<W>) { /* ... */ }
///
/// let _ = async_only(std::io::empty());
/// ```
///
/// [`Write`]: crate::Write
/// [`AsyncWrite`]: crate::AsyncWrite
pub trait IntoWrite<T> {
	/// Convert `self` to a writer `T`.
	fn into_write(self) -> T;
}

/// This trait should be implemented for every "newtype".
///
/// It is a generic interface that allows access to a type `T` while still
/// allowing foreign trait implementations on it. Usually this trait is
/// implemented by single-field tuple structs that store a type `T` and
/// implement some traits. This is used to implement [`Read`], [`Write`] and
/// friends on foreign types.
///
/// Consider the following:
///
/// ```rust,no_run
/// use std::io::{empty, Empty};
/// use channels_io::{IntoRead, Std, Read, Container};
///
/// struct MyStruct<R> {
///     reader: R
/// }
///
/// impl<R> MyStruct<R> {
///     pub fn new(reader: impl IntoRead<R>) -> Self {
///         Self { reader: reader.into_read() }
///     }
/// }
///
/// impl<R: Container> MyStruct<R> {
///     pub fn get(&self) -> &R::Inner {
///         self.reader.get_ref()
///     }
///
///     pub fn get_mut(&mut self) -> &mut R::Inner {
///         self.reader.get_mut()
///     }
/// }
///
/// let mut foo = MyStruct::new(empty());
/// // foo.reader is not an `Empty`
///
/// // Get references
/// let reader: &Empty = foo.get();
/// let reader: &mut Empty = foo.get_mut();
///
/// // Destruct into the original type.
/// let reader: Empty = foo.reader.into_inner();
/// ```
///
/// [`Read`]: crate::Read
/// [`Write`]: crate::Write
pub trait Container {
	/// The inner type this container stores.
	type Inner;

	/// Construct a `Self` from the inner type.
	fn from_inner(inner: Self::Inner) -> Self;

	/// Get a reference to the inner type.
	fn get_ref(&self) -> &Self::Inner;

	/// Get a mutable reference to the inner type.
	fn get_mut(&mut self) -> &mut Self::Inner;

	/// Destruct the container into its inner type.
	fn into_inner(self) -> Self::Inner;
}

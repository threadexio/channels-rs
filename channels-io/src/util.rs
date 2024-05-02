pub use core::future::Future;

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

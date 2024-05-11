/// A trait that describes an error returned by [`Read`] and [`AsyncRead`].
///
/// [`Read`]: trait@crate::Read
/// [`AsyncRead`]: trait@crate::AsyncRead
pub trait ReadError {
	/// Create a new End-Of-File error.
	fn eof() -> Self;

	/// Checks whether the given error indicates that the operation should be retried.
	fn should_retry(&self) -> bool;
}

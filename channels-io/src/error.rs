//! Error traits that describe IO errors.

/// Common functionality for [`ReadError`] and [`WriteError`].
///
/// [`ReadError`]: trait@ReadError
/// [`WriteError`]: trait@WriteError
pub trait IoError {
	/// Checks whether the given error indicates that the operation should be retried.
	fn should_retry(&self) -> bool;
}

/// A trait that describes an error returned by [`Read`] and [`AsyncRead`].
///
/// [`Read`]: trait@crate::Read
/// [`AsyncRead`]: trait@crate::AsyncRead
pub trait ReadError: IoError {
	/// Create a new End-Of-File error.
	fn eof() -> Self;
}

/// A trait that describes an error returned by [`Write`] and [`AsyncWrite`].
///
/// [`Write`]: trait@crate::Write
/// [`AsyncWrite`]: trait@crate::AsyncWrite
pub trait WriteError: IoError {
	/// Create a new error that indicates zero written bytes when writing.
	fn write_zero() -> Self;
}

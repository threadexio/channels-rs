use core::fmt;

use std::error::Error as StdError;
use std::io;

/// The error type returned the [`Gzip`](super::Gzip) middleware.
#[derive(Debug)]
pub enum Error<T>
where
	T: StdError,
{
	/// An IO error was encountered while trying to read/write the length field.
	Io(io::Error),
	/// The next serializer/deserializer returned an error.
	Next(T),
}

impl<T> fmt::Display for Error<T>
where
	T: StdError,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Io(io_error) => write!(f, "{io_error}"),
			Self::Next(e) => write!(f, "{e}"),
		}
	}
}

impl<T> StdError for Error<T> where T: StdError {}

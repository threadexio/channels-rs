use std::error::Error as StdError;
use std::fmt;

/// An error type that represents all possible errors that a [`Serializer`]
/// or [`Deserializer`] can throw.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
	/// The given data is not enought to complete a full object.
	NotEnough,
	/// Other error.
	Other(String),
}

unsafe impl Send for Error {}
unsafe impl Sync for Error {}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::NotEnough => write!(f, "not enough data"),
			Self::Other(s) => write!(f, "{s}"),
		}
	}
}

impl StdError for Error {}

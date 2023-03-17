use std::error::Error as StdError;
use std::fmt;
use std::io;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	VersionMismatch,
	ChecksumError,
	SizeLimit,
	OutOfOrder,
	Serde(Box<dyn StdError>),
	Io(io::Error),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::VersionMismatch => write!(
				f,
				"peer does not have the correct crate version"
			),
			Self::ChecksumError => write!(f, "corrupted data"),
			Self::SizeLimit => write!(f, "data too large"),
			Self::OutOfOrder => {
				write!(f, "data was received out of order")
			},
			Self::Serde(e) => write!(f, "{}", e),
			Self::Io(e) => write!(f, "{}", e),
		}
	}
}

impl StdError for Error {}

impl From<Box<dyn StdError>> for Error {
	fn from(value: Box<dyn StdError>) -> Self {
		Self::Serde(value)
	}
}

impl From<io::Error> for Error {
	fn from(value: io::Error) -> Self {
		Self::Io(value)
	}
}

#[cfg(feature = "serde")]
impl From<bincode::Error> for Error {
	fn from(value: bincode::Error) -> Self {
		Self::Serde(value)
	}
}

unsafe impl Send for Error {}
unsafe impl Sync for Error {}

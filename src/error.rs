use std::error::Error as StdError;
use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("peer does not have the correct crate version")]
	VersionMismatch,
	#[error("corrupted data")]
	ChecksumError,
	#[error("data too large")]
	SizeLimit,
	#[error("data was received out of order")]
	OutOfOrder,

	#[error(transparent)]
	Serde(#[from] Box<dyn StdError>),
	#[error(transparent)]
	Io(#[from] io::Error),
}

#[cfg(feature = "serde")]
impl From<bincode::Error> for Error {
	fn from(value: bincode::Error) -> Self {
		Self::Serde(value)
	}
}

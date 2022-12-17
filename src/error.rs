use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("peer does not have the correct crate version")]
	VersionMismatch,
	#[error("corrupted data")]
	ChecksumError,
	#[error("data was received out of order")]
	OutOfOrder,

	#[error(transparent)]
	Serde(#[from] bincode::Error),
	#[error(transparent)]
	Io(#[from] io::Error),
}

use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	VersionMismatch,
	ChecksumError,

	DataTooLarge, // data size

	DataError(bincode::Error),

	Io(io::Error),
}

impl std::fmt::Display for Error {
	fn fmt(
		&self,
		f: &mut std::fmt::Formatter<'_>,
	) -> std::fmt::Result {
		use Error::*;

		match self {
			VersionMismatch => write!(f, "version mismatch"),
			ChecksumError => write!(f, "checksum did not match"),
			DataTooLarge => {
				write!(f, "data bigger than max packet size")
			},
			DataError(e) => write!(f, "data error: {}", e),
			Io(e) => write!(f, "io error: {}", e),
		}
	}
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
	fn from(e: io::Error) -> Self {
		Self::Io(e)
	}
}

impl From<bincode::Error> for Error {
	fn from(e: bincode::Error) -> Self {
		use bincode::ErrorKind;
		match *e {
			ErrorKind::Io(e) => Self::Io(e),
			ErrorKind::SizeLimit => Self::DataTooLarge,
			_ => Self::DataError(e),
		}
	}
}

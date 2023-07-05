use std::error::Error as StdError;
use std::fmt;
use std::io;

use crate::serdes;

/// A result type for [`Error`].
pub type Result<T> = core::result::Result<T, Error>;

/// The error type returned by [`Sender`](crate::Sender)s and [`Receiver`](crate::Receiver)s.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
	/// The 2 peers are not using the same protocol version. This means
	/// that each end is not using the same version of the crate.
	///
	/// # Safety
	///
	/// This error is **NOT** recoverable and the crate version should be
	/// updated.
	VersionMismatch,
	/// The underlying transport is not reliable and the sent data has
	/// suffered modification and/or corruption.
	///
	/// # Safety
	///
	/// This error is usually **NOT** recoverable and the channel should
	/// be closed immediately.
	ChecksumError,
	/// The underlying transport is not reliable and the sent data has
	/// been received in the wrong order.
	///
	/// # Safety
	///
	/// This error is usually **NOT** recoverable and the channel should
	/// be closed immediately.
	OutOfOrder,
	/// The serializer has encountered an error while trying to serialize/deserialize
	/// the data. This error is usually recoverable and the channel might still be
	/// able to be used normally.
	Serde(serdes::Error),
	/// The underlying transport has returned an error while the data was
	/// being sent/received. This error is recoverable and the channel can
	/// continue to be used normally.
	Io(io::Error),
}

unsafe impl Send for Error {}
unsafe impl Sync for Error {}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::VersionMismatch => write!(
				f,
				"peer does not have the correct crate version"
			),
			Self::ChecksumError => write!(f, "corrupted data"),
			Self::OutOfOrder => {
				write!(f, "data was received out of order")
			},
			Self::Serde(e) => write!(f, "{e}"),
			Self::Io(e) => write!(f, "{e}"),
		}
	}
}

impl StdError for Error {}

impl From<serdes::Error> for Error {
	fn from(value: serdes::Error) -> Self {
		Self::Serde(value)
	}
}

impl From<io::Error> for Error {
	fn from(value: io::Error) -> Self {
		Self::Io(value)
	}
}

impl PartialEq for Error {
	fn eq(&self, other: &Self) -> bool {
		use core::mem::discriminant;
		match (self, other) {
			(Self::Serde(l0), Self::Serde(r0)) => l0 == r0,
			_ => discriminant(self) == discriminant(other),
		}
	}
}

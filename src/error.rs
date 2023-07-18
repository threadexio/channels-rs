use core::fmt;

use std::error::Error as StdError;
use std::io;

/// The error type returned by [`Sender`](crate::Sender).
#[derive(Debug)]
#[non_exhaustive]
pub enum SendError {
	/// The serializer has encountered an error while trying to serialize/deserialize
	/// the data. This error is usually recoverable and the channel might still be
	/// able to be used normally.
	Serde(Box<dyn StdError>),
	/// The underlying transport has returned an error while the data was
	/// being sent/received. This error is recoverable and the channel can
	/// continue to be used normally.
	Io(io::Error),
}

impl fmt::Display for SendError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Serde(e) => write!(f, "{e}"),
			Self::Io(e) => write!(f, "{e}"),
		}
	}
}

impl StdError for SendError {}

impl From<io::Error> for SendError {
	fn from(value: io::Error) -> Self {
		Self::Io(value)
	}
}

/// The error type returned by [`Receiver`](crate::Receiver).
#[derive(Debug)]
#[non_exhaustive]
pub enum RecvError {
	/// The serializer has encountered an error while trying to serialize/deserialize
	/// the data. This error is usually recoverable and the channel might still be
	/// able to be used normally.
	Serde(Box<dyn StdError>),
	/// The underlying transport has returned an error while the data was
	/// being sent/received. This error is recoverable and the channel can
	/// continue to be used normally.
	Io(io::Error),

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
}

impl fmt::Display for RecvError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Serde(e) => write!(f, "{e}"),
			Self::Io(e) => write!(f, "{e}"),
			Self::VersionMismatch => write!(f, "version mismatch"),
			Self::ChecksumError => write!(f, "corrupted data"),
			Self::OutOfOrder => {
				write!(f, "data was received out of order")
			},
		}
	}
}

impl StdError for RecvError {}

impl From<io::Error> for RecvError {
	fn from(value: io::Error) -> Self {
		Self::Io(value)
	}
}

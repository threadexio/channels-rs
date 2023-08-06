//! Error module.

use core::fmt;

use std::error::Error as StdError;
use std::io;

/// The error type returned by [`Sender`](crate::Sender).
#[derive(Debug)]
#[non_exhaustive]
pub enum SendError<SE>
where
	SE: StdError,
{
	/// The serializer has encountered an error while trying to serialize/deserialize
	/// the data. This error is usually recoverable and the channel might still be
	/// able to be used normally.
	Serde(SE),
	/// The underlying transport has returned an error while the data was
	/// being sent/received. This error is recoverable and the channel can
	/// continue to be used normally.
	Io(io::Error),
}

impl<SE> fmt::Display for SendError<SE>
where
	SE: StdError,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Serde(e) => write!(f, "{e}"),
			Self::Io(e) => write!(f, "{e}"),
		}
	}
}

impl<SE> StdError for SendError<SE> where SE: StdError {}

impl<SE> From<io::Error> for SendError<SE>
where
	SE: StdError,
{
	fn from(value: io::Error) -> Self {
		Self::Io(value)
	}
}

/// The possible errors when verifying a received packet.
#[derive(Debug)]
#[non_exhaustive]
pub enum VerifyError {
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
	/// The received header contained invalid data. This error is
	/// usually **NOT** recoverable and the channel should be closed
	/// immediately.
	InvalidHeader,
}

impl fmt::Display for VerifyError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::VersionMismatch => write!(f, "version mismatch"),
			Self::ChecksumError => write!(f, "corrupted data"),
			Self::OutOfOrder => {
				write!(f, "data was received out of order")
			},
			Self::InvalidHeader => {
				write!(f, "invalid packet")
			},
		}
	}
}

impl StdError for VerifyError {}

/// The error type returned by [`Receiver`](crate::Receiver).
#[derive(Debug)]
#[non_exhaustive]
pub enum RecvError<DE>
where
	DE: StdError,
{
	/// The serializer has encountered an error while trying to serialize/deserialize
	/// the data. This error is usually recoverable and the channel might still be
	/// able to be used normally.
	Serde(DE),
	/// The underlying transport has returned an error while the data was
	/// being sent/received. This error is recoverable and the channel can
	/// continue to be used normally.
	Io(io::Error),
	/// A received packet could not be verified. This error is usually unrecoverable
	/// and the channel should not be used further.
	Verify(VerifyError),
	/// The error variant returned by [`Receiver::recv_timeout`]. This
	/// error is only returned by the above method, thus it is safe to
	/// ignore in cases were that method is not being used.
	///
	/// **NOTE:** [`Receiver::recv_timeout`]
	///
	/// [`Receiver::recv_timeout`]: crate::Receiver::recv_timeout
	Timeout,
}

impl<DE> fmt::Display for RecvError<DE>
where
	DE: StdError,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Serde(e) => write!(f, "{e}"),
			Self::Io(e) => write!(f, "{e}"),
			Self::Verify(e) => write!(f, "{e}"),
			Self::Timeout => write!(f, "timed out"),
		}
	}
}

impl<DE> StdError for RecvError<DE> where DE: StdError {}

impl<DE> From<io::Error> for RecvError<DE>
where
	DE: StdError,
{
	fn from(value: io::Error) -> Self {
		Self::Io(value)
	}
}

impl<DE> From<VerifyError> for RecvError<DE>
where
	DE: StdError,
{
	fn from(value: VerifyError) -> Self {
		Self::Verify(value)
	}
}

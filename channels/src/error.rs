//! Error types for `channels`.
use core::fmt::{self, Debug, Display};

trait Error: Debug + Display {}
impl<T: Debug + Display + ?Sized> Error for T {}

/// The error type returned by [`Sender`](crate::Sender).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum SendError<Ser, Io> {
	/// The underlying transport has returned an error while the data was being
	/// sent/received. This error is recoverable and the channel can continue to
	/// be used normally.
	Serde(Ser),
	/// The serializer has encountered an error while trying to
	/// serialize/deserialize the data. This error is usually recoverable and
	/// the channel might still be able to be used normally.
	Io(Io),
}

impl<Ser: Error, Io: Error> Display for SendError<Ser, Io> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use SendError as A;
		match self {
			A::Serde(e) => Display::fmt(e, f),
			A::Io(e) => Display::fmt(e, f),
		}
	}
}

#[cfg(feature = "std")]
impl<Ser: Error, Io: Error> std::error::Error for SendError<Ser, Io> {}

/// The possible errors when verifying a received packet.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum VerifyError {
	/// The 2 peers are not using the same protocol version. This means that
	/// each end is not using the same version of the crate.
	///
	/// # Safety
	///
	/// This error is **NOT** recoverable and the crate version should be
	/// updated.
	VersionMismatch,
	/// The underlying transport is not reliable and the sent data has suffered
	/// modification and/or corruption.
	///
	/// # Safety
	///
	/// This error is usually **NOT** recoverable and the channel should be
	/// closed immediately.
	ChecksumError,
	/// The underlying transport is not reliable and the sent data has been
	/// received in the wrong order.
	///
	/// # Safety
	///
	/// This error is usually **NOT** recoverable and the channel should be
	/// closed immediately.
	OutOfOrder,
	/// The received header contained invalid data. This error is usually
	/// **NOT** recoverable and the channel should be closed immediately.
	InvalidHeader,
}

use channels_packet::HeaderReadError;
impl From<HeaderReadError> for VerifyError {
	fn from(value: HeaderReadError) -> Self {
		use HeaderReadError as L;
		use VerifyError as R;

		match value {
			L::VersionMismatch => R::VersionMismatch,
			L::InvalidChecksum => R::ChecksumError,
			L::InvalidLength => R::InvalidHeader,
			L::OutOfOrder => R::OutOfOrder,
		}
	}
}

impl Display for VerifyError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::VersionMismatch => f.write_str("version mismatch"),
			Self::ChecksumError => f.write_str("corrupted data"),
			Self::OutOfOrder => {
				f.write_str("data was received out of order")
			},
			Self::InvalidHeader => f.write_str("invalid packet"),
		}
	}
}

#[cfg(feature = "std")]
impl std::error::Error for VerifyError {}

/// The possible errors when receiving data.
///
/// These errors can be caused by misconfiguration of the [`Receiver`] or a
/// misbehaving sender.
///
/// [`Receiver`]: crate::Receiver
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
	/// The received data exceeded the maximum amount of data the receiver was
	/// configured to receive. This error indicates either that: a) you must
	/// configure the receiver to allow larger payloads with [`max_size()`], or
	/// b) an attack was prevented.
	///
	/// # Safety
	///
	/// This error is **NOT** recoverable and the channel should be closed
	/// immediately.
	///
	/// [`max_size()`]: crate::receiver::Config::max_size()
	ExceededMaximumSize,
}

impl Display for ProtocolError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::ExceededMaximumSize => {
				f.write_str("exceeded maximum payload size")
			},
		}
	}
}

#[cfg(feature = "std")]
impl std::error::Error for ProtocolError {}

/// The error type returned by [`Receiver`](crate::Receiver).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum RecvError<Ser, Io> {
	/// The underlying transport has returned an error while the data was being
	/// sent/received. This error is recoverable and the channel can continue to
	/// be used normally.
	Serde(Ser),
	/// The serializer has encountered an error while trying to
	/// serialize/deserialize the data. This error is usually recoverable and
	/// the channel might still be able to be used normally.
	Io(Io),
	/// A received packet could not be verified. This error is usually
	/// unrecoverable and the channel should not be used further.
	Verify(VerifyError),
	/// The receiver encountered an error while processing the data. See
	/// [`ProtocolError`] for more.
	Protocol(ProtocolError),
}

impl<Ser: Error, Io: Error> Display for RecvError<Ser, Io> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use RecvError as A;
		match self {
			A::Serde(e) => Display::fmt(e, f),
			A::Io(e) => Display::fmt(e, f),
			A::Verify(e) => Display::fmt(e, f),
			A::Protocol(e) => Display::fmt(e, f),
		}
	}
}

#[cfg(feature = "std")]
impl<Ser: Error, Io: Error> std::error::Error for RecvError<Ser, Io> {}

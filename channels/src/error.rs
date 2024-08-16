//! Error types for `channels`.

use core::fmt::{self, Debug, Display};

use channels_packet::codec::{DecodeError, EncodeError};

use crate::io::framed::{FramedReadError, FramedWriteError};

/// The error type returned by [`Sender`].
///
/// [`Sender`]: crate::Sender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SendError<Ser, Io> {
	/// The serializer has encountered an error while trying to
	/// serialize/deserialize the data. This error is usually recoverable and
	/// the channel might still be able to be used normally.
	Io(Io),

	/// The underlying transport has returned an error while the data was being
	/// sent/received. This error is recoverable and the channel can continue to
	/// be used normally.
	Serde(Ser),

	/// TODO: docs
	TooLarge,
}

impl<Ser, Io> Display for SendError<Ser, Io>
where
	Ser: Display,
	Io: Display,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Io(e) => Display::fmt(e, f),
			Self::Serde(e) => Display::fmt(e, f),
			Self::TooLarge => f.write_str("data too large"),
		}
	}
}

#[cfg(feature = "std")]
impl<Ser, Io> std::error::Error for SendError<Ser, Io> where
	Self: Debug + Display
{
}

impl<Ser, Io> From<FramedWriteError<EncodeError, Io>>
	for SendError<Ser, Io>
{
	fn from(err: FramedWriteError<EncodeError, Io>) -> Self {
		use FramedWriteError as A;
		use SendError as B;

		match err {
			A::Io(e) => B::Io(e),
			A::Encode(EncodeError::TooLarge) => B::TooLarge,
		}
	}
}

/// The error type returned by [`Receiver`].
///
/// [`Receiver`]: crate::Receiver
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RecvError<Des, Io> {
	/// The underlying transport has returned an error while the data was being
	/// sent/received.
	Io(Io),

	/// The serializer has encountered an error while trying to
	/// serialize/deserialize the data.
	Serde(Des),

	/// The underlying transport is not reliable and the sent data has suffered
	/// modification and/or corruption.
	InvalidChecksum,

	/// The received data exceeded the maximum amount of data the receiver was
	/// configured to receive. This error indicates either that: a) you must
	/// configure the receiver to allow larger payloads with [`max_size()`], or
	/// b) an attack was prevented.
	///
	/// [`max_size()`]: crate::receiver::Config::max_size()
	TooLarge,

	/// The underlying transport is not reliable and the sent data has been
	/// received in the wrong order.
	OutOfOrder,

	/// The 2 peers are not using the same protocol version. This means that
	/// each end is not using the same version of the crate.
	VersionMismatch,
}

impl<Des: Display, Io: Display> Display for RecvError<Des, Io> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Io(e) => Display::fmt(e, f),
			Self::Serde(e) => Display::fmt(e, f),
			Self::InvalidChecksum => f.write_str("corrupted data"),
			Self::OutOfOrder => f.write_str("data out of order"),
			Self::TooLarge => f.write_str("payload too large"),
			Self::VersionMismatch => f.write_str("version mismatch"),
		}
	}
}

#[cfg(feature = "std")]
impl<Des, Io> std::error::Error for RecvError<Des, Io> where
	Self: Debug + Display
{
}

impl<Des, Io> From<FramedReadError<DecodeError, Io>>
	for RecvError<Des, Io>
{
	fn from(err: FramedReadError<DecodeError, Io>) -> Self {
		use FramedReadError as A;
		use RecvError as B;

		match err {
			A::Io(e) => B::Io(e),
			A::Decode(DecodeError::InvalidChecksum) => {
				B::InvalidChecksum
			},
			A::Decode(DecodeError::OutOfOrder) => B::OutOfOrder,
			A::Decode(DecodeError::TooLarge) => B::TooLarge,
			A::Decode(DecodeError::VersionMismatch) => {
				B::VersionMismatch
			},
		}
	}
}

//! Error types for `channels`.

use core::fmt::{self, Debug, Display};

use channels_io::framed::{FramedReadError, FramedWriteError};

/// Errors during encoding of a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EncodeError {
	/// Frame is too large.
	TooLarge,
}

impl fmt::Display for EncodeError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::TooLarge => f.write_str("data too large"),
		}
	}
}

#[cfg(feature = "std")]
impl std::error::Error for EncodeError {}

/// The error type returned by [`Sender`].
///
/// [`Sender`]: crate::Sender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SendError<Ser, Io> {
	/// The underlying protocol could not encode the provided data.
	Encode(EncodeError),

	/// The serializer has encountered an error while trying to
	/// serialize/deserialize the data. This error is usually recoverable and
	/// the channel might still be able to be used normally.
	Io(Io),

	/// The underlying transport has returned an error while the data was being
	/// sent/received. This error is recoverable and the channel can continue to
	/// be used normally.
	Serde(Ser),
}

impl<Ser, Io> Display for SendError<Ser, Io>
where
	Ser: Display,
	Io: Display,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Encode(e) => Display::fmt(e, f),
			Self::Io(e) => Display::fmt(e, f),
			Self::Serde(e) => Display::fmt(e, f),
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
	fn from(value: FramedWriteError<EncodeError, Io>) -> Self {
		match value {
			FramedWriteError::Encode(x) => Self::Encode(x),
			FramedWriteError::Io(x) => Self::Io(x),
		}
	}
}

/// Errors during decoding of a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DecodeError {
	/// The underlying transport is not reliable and the sent data has suffered
	/// modification and/or corruption.
	InvalidChecksum,

	/// The underlying transport is not reliable and the sent data has been
	/// received in the wrong order.
	OutOfOrder,

	/// The received data exceeded the maximum amount of data the receiver was
	/// configured to receive. This error indicates either that: a) you must
	/// configure the receiver to allow larger payloads with [`max_size()`], or
	/// b) an attack was prevented.
	///
	/// [`max_size()`]: crate::receiver::Config::max_size()
	TooLarge,

	/// The 2 peers are not using the same protocol version. This means that
	/// each end is not using the same version of the crate.
	VersionMismatch,
}

impl Display for DecodeError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::InvalidChecksum => f.write_str("invalid checksum"),
			Self::OutOfOrder => f.write_str("data out of order"),
			Self::TooLarge => f.write_str("data too large"),
			Self::VersionMismatch => f.write_str("version mismatch"),
		}
	}
}

#[cfg(feature = "std")]
impl std::error::Error for DecodeError {}

/// The error type returned by [`Receiver`].
///
/// [`Receiver`]: crate::Receiver
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RecvError<Des, Io> {
	/// The underlying protocol could not decode the received data.
	Decode(DecodeError),

	/// The underlying transport has returned an error while the data was being
	/// sent/received.
	Io(Io),

	/// The serializer has encountered an error while trying to
	/// serialize/deserialize the data.
	Serde(Des),
}

impl<Des: Display, Io: Display> Display for RecvError<Des, Io> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Decode(e) => Display::fmt(e, f),
			Self::Io(e) => Display::fmt(e, f),
			Self::Serde(e) => Display::fmt(e, f),
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
	fn from(value: FramedReadError<DecodeError, Io>) -> Self {
		match value {
			FramedReadError::Decode(x) => Self::Decode(x),
			FramedReadError::Io(x) => Self::Io(x),
		}
	}
}

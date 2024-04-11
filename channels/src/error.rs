//! Error types for `channels`.

use core::fmt::{self, Debug, Display};

use channels_packet::header::VerifyError;

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
}

impl<Ser, Io> Display for SendError<Ser, Io>
where
	Ser: Display,
	Io: Display,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use SendError as A;
		match self {
			A::Io(e) => Display::fmt(e, f),
			A::Serde(e) => Display::fmt(e, f),
		}
	}
}

#[cfg(feature = "std")]
impl<Ser, Io> std::error::Error for SendError<Ser, Io>
where
	Ser: Debug + Display,
	Io: Debug + Display,
{
}

/// The error type returned by [`Receiver`].
///
/// [`Receiver`]: crate::Receiver
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RecvError<Des, Io> {
	/// The underlying transport is not reliable and the sent data has suffered
	/// modification and/or corruption.
	ChecksumError,

	/// The received data exceeded the maximum amount of data the receiver was
	/// configured to receive. This error indicates either that: a) you must
	/// configure the receiver to allow larger payloads with [`max_size()`], or
	/// b) an attack was prevented.
	///
	/// [`max_size()`]: crate::receiver::Config::max_size()
	ExceededMaximumSize,

	/// The received header contained invalid data.
	InvalidHeader,

	/// The underlying transport has returned an error while the data was being
	/// sent/received.
	Io(Io),

	/// The underlying transport is not reliable and the sent data has been
	/// received in the wrong order.
	OutOfOrder,

	/// The serializer has encountered an error while trying to
	/// serialize/deserialize the data.
	Serde(Des),

	/// The 2 peers are not using the same protocol version. This means that
	/// each end is not using the same version of the crate.
	VersionMismatch,

	/// A fragment was received but it had no data.
	ZeroSizeFragment,
}

impl<Ser: Display, Io: Display> Display for RecvError<Ser, Io> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use RecvError as A;
		match self {
			A::ChecksumError => f.write_str("corrupted data"),
			A::ExceededMaximumSize => {
				f.write_str("exceeded maximum payload size")
			},
			A::InvalidHeader => f.write_str("invalid packet"),
			A::Io(e) => Display::fmt(e, f),
			A::OutOfOrder => {
				f.write_str("data was received out of order")
			},
			A::Serde(e) => Display::fmt(e, f),
			A::VersionMismatch => f.write_str("version mismatch"),
			A::ZeroSizeFragment => f.write_str("zero size fragment"),
		}
	}
}

#[cfg(feature = "std")]
impl<Ser, Io> std::error::Error for RecvError<Ser, Io>
where
	Ser: Debug + Display,
	Io: Debug + Display,
{
}

impl<Des, Io> From<VerifyError> for RecvError<Des, Io> {
	fn from(value: VerifyError) -> Self {
		use RecvError as B;
		use VerifyError as A;

		match value {
			A::InvalidChecksum => B::ChecksumError,
			A::InvalidLength => B::InvalidHeader,
			A::OutOfOrder => B::OutOfOrder,
			A::VersionMismatch => B::VersionMismatch,
		}
	}
}

use alloc::vec::Vec;

use crate::error::RecvError;
use crate::io::{AsyncRead, Read};
use crate::util::StatIO;

use super::deframer::{DeframeError, DeframeStatus, Deframer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecvPayloadError<Io> {
	ChecksumError,
	ExceededMaximumSize,
	InvalidHeader,
	Io(Io),
	OutOfOrder,
	VersionMismatch,
	ZeroSizeFragment,
}

impl<Io> From<DeframeError> for RecvPayloadError<Io> {
	fn from(value: DeframeError) -> Self {
		use DeframeError as A;
		use RecvPayloadError as B;

		match value {
			A::ChecksumError => B::ChecksumError,
			A::ExceededMaximumSize => B::ExceededMaximumSize,
			A::InvalidHeader => B::InvalidHeader,
			A::OutOfOrder => B::OutOfOrder,
			A::VersionMismatch => B::VersionMismatch,
			A::ZeroSizeFragment => B::ZeroSizeFragment,
		}
	}
}

impl<Des, Io> From<RecvPayloadError<Io>> for RecvError<Des, Io> {
	fn from(value: RecvPayloadError<Io>) -> Self {
		use RecvError as B;
		use RecvPayloadError as A;

		match value {
			A::ChecksumError => B::ChecksumError,
			A::ExceededMaximumSize => B::ExceededMaximumSize,
			A::InvalidHeader => B::InvalidHeader,
			A::Io(x) => B::Io(x),
			A::OutOfOrder => B::OutOfOrder,
			A::VersionMismatch => B::VersionMismatch,
			A::ZeroSizeFragment => B::ZeroSizeFragment,
		}
	}
}

channels_macros::replace! {
	replace: {
		// Synchronous version
		[
			(async =>)
			(await =>)
			(recv  => recv_sync)
			(Read  => Read)
			(run   => run_sync)
		]
		// Asynchronous version
		[
			(async => async)
			(await => .await)
			(recv => recv_async)
			(Read => AsyncRead)
			(run => run_async)
		]
	}
	code: {

pub async fn recv<R>(
	reader: &mut StatIO<R>,
	deframer: &mut Deframer,
) -> Result<Vec<u8>, RecvPayloadError<R::Error>>
where
	R: Read,
{
	use DeframeStatus::{NotReady, Ready};

	#[cfg(not(feature = "statistics"))]
	reader.statistics.inc_ops();

	loop {
		match deframer.deframe({
			#[cfg(feature = "statistics")]
			let statistics = &mut reader.statistics;
			#[cfg(not(feature = "statistics"))]
			let statistics = &mut Statistics::new();
			statistics
		}) {
			Ready(Ok(payload)) => break Ok(payload),
			Ready(Err(e)) =>  break Err(e.into()),
			NotReady(r) => {
				reader.read(r.buf) await
					.map_err(RecvPayloadError::Io)?;
				continue;
			}
		}
	}
}

	}
}

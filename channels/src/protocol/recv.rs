use alloc::vec::Vec;

use crate::error::RecvError;
use crate::io::{AsyncRead, Read};
use crate::statistics::StatIO;

use super::deframer::{DeframeError, DeframeStatus, Deframer};

pub(crate) struct ReceiverCore<R> {
	pub(crate) reader: StatIO<R>,
	pub(crate) deframer: Deframer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreRecvError<Io> {
	ChecksumError,
	ExceededMaximumSize,
	InvalidHeader,
	Io(Io),
	OutOfOrder,
	VersionMismatch,
	ZeroSizeFragment,
}

impl<Io> From<DeframeError> for CoreRecvError<Io> {
	fn from(value: DeframeError) -> Self {
		use CoreRecvError as B;
		use DeframeError as A;

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

impl<Des, Io> From<CoreRecvError<Io>> for RecvError<Des, Io> {
	fn from(value: CoreRecvError<Io>) -> Self {
		use CoreRecvError as A;
		use RecvError as B;

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
		]
		// Asynchronous version
		[
			(async => async)
			(await => .await)
			(recv => recv_async)
			(Read => AsyncRead)
		]
	}
	code: {

impl<R> ReceiverCore<R>
where
	R: Read,
{
	pub async fn recv(
		&mut self,
	) -> Result<Vec<u8>, CoreRecvError<R::Error>> {
		use DeframeStatus::{NotReady, Ready};

		self.reader.statistics.inc_ops();

		loop {
			match self.deframer.deframe(&mut self.reader.statistics) {
				Ready(Ok(payload)) => break Ok(payload),
				Ready(Err(e)) =>  break Err(e.into()),
				NotReady(r) => {
					self.reader.read(r.buf) await
						.map_err(CoreRecvError::Io)?;
					continue;
				}
			}
		}
	}
}

	}
}

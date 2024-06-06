use alloc::vec::Vec;

use crate::{AsyncWrite, Write};

#[derive(Debug)]
pub struct Buffered<'a> {
	buf: &'a mut Vec<u8>,
}

pub fn new(buf: &mut Vec<u8>) -> Buffered {
	Buffered { buf }
}

#[allow(clippy::unnecessary_wraps, clippy::unused_async)]
impl<'a> Buffered<'a> {
	channels_macros::replace! {
		replace: {
			[
				(async =>)
				(await =>)
				(Write => Write)
				(add => add_sync)
				(finish => finish_sync)
			]
			[
				(async => async)
				(await => .await)
				(Write => AsyncWrite)
				(add => add_async)
				(finish => finish_async)
			]
		}
		code: {

	pub async fn add<W: Write>(
		&mut self,
		_writer: &mut W,
		buf: &[u8],
	) -> Result<(), W::Error> {
		self.buf.extend_from_slice(buf);
		Ok(())
	}

	pub async fn finish<W: Write>(
		self,
		writer: &mut W,
	) -> Result<(), W::Error> {
		writer.write(self.buf) await
	}

		}
	}
}

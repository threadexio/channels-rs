use crate::{AsyncWrite, Write};

#[derive(Debug)]
pub struct Unbuffered {}

pub fn new() -> Unbuffered {
	Unbuffered {}
}

#[allow(
	clippy::unnecessary_wraps,
	clippy::unused_async,
	clippy::unused_self
)]
impl Unbuffered {
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
		writer: &mut W,
		buf: &[u8],
	) -> Result<(), W::Error> {
		writer.write(buf) await
	}

	pub async fn finish<W: Write>(
		self,
		_writer: &mut W,
	) -> Result<(), W::Error> {
		Ok(())
	}

		}
	}
}

use std::convert::Infallible;
use std::hint::black_box;

use channels::io::{Buf, Cursor, Native};
use channels::receiver::{Config, Receiver};
use channels::serdes::Deserializer;

static PACKET: &[u8] = include_bytes!("./packet.bin");

struct NoopDeserializer;

impl Deserializer<()> for NoopDeserializer {
	type Error = Infallible;

	fn deserialize(
		&mut self,
		_buf: &mut [u8],
	) -> Result<(), Self::Error> {
		Ok(())
	}
}

struct Repeat<T: AsRef<[u8]>> {
	inner: Cursor<T>,
}

impl<T: AsRef<[u8]>> Buf for Repeat<T> {
	fn remaining(&self) -> usize {
		self.inner.remaining()
	}

	fn chunk(&self) -> &[u8] {
		self.inner.chunk()
	}

	fn advance(&mut self, n: usize) {
		let new_pos =
			(self.inner.pos() + n) % self.inner.get().as_ref().len();
		self.inner.set_pos(new_pos);
	}
}

fn main() {
	let mut rx: Receiver<(), Native<_>, NoopDeserializer> =
		Receiver::<(), _, _>::builder()
			.reader(Repeat { inner: Cursor::new(PACKET) }.reader())
			.deserializer(NoopDeserializer)
			.config(Config::default().with_verify_order(false))
			.build();

	perf::run_for_default_duration(|| {
		#[allow(clippy::unit_arg)]
		black_box(rx.recv_blocking().unwrap());
	});
}

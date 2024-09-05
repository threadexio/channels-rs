use std::convert::Infallible;
use std::hint::black_box;

use channels::io::{Buf, Cursor, Native};
use channels::receiver::{Config, Receiver};
use channels::serdes::Deserializer;

// Taken from: `cargo run --package stdio -- sender - 48`
const PACKET: [u8; 64] = [
	0x42, 0x00, 0x85, 0xff, 0x38, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x30, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05,
	0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
	0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
	0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26,
	0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f,
];

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

	let r = perf::run_for_default_duration(|| {
		#[allow(clippy::unit_arg)]
		black_box(rx.recv_blocking().unwrap());
	});

	eprintln!("{r}");
}

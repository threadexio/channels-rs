use std::convert::Infallible;
use std::hint::black_box;
use std::io::empty;

use channels::sender::{Config, Sender};
use channels::serdes::Serializer;

struct NoopSerializer;

impl Serializer<()> for NoopSerializer {
	type Error = Infallible;

	fn serialize(&mut self, _: &()) -> Result<Vec<u8>, Self::Error> {
		Ok(Vec::new())
	}
}

fn main() {
	let mut tx = Sender::<(), _, _>::builder()
		.writer(empty())
		.serializer(NoopSerializer)
		.config(Config::default())
		.build();

	perf::run_for_default_duration(|| {
		#[allow(clippy::unit_arg)]
		black_box(tx.send_blocking(black_box(())).unwrap());
	});
}

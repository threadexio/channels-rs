use std::convert::Infallible;

use criterion::{criterion_group, criterion_main, Criterion};

use channels::io::IntoWrite;
use channels::sender::{Config, Sender};
use channels::serdes::Serializer;

struct FastSerializer;

impl Serializer<()> for FastSerializer {
	type Error = Infallible;

	fn serialize(&mut self, _: &()) -> Result<Vec<u8>, Self::Error> {
		Ok(Vec::new())
	}
}

fn make_sender<W>(
	writer: impl IntoWrite<W>,
	config: Config,
) -> Sender<(), W, FastSerializer> {
	Sender::builder()
		.serializer(FastSerializer)
		.writer(writer)
		.config(config)
		.build()
}

fn tokio_rt() -> tokio::runtime::Runtime {
	tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
		.unwrap()
}

fn bench_send_sync(c: &mut Criterion, id: &str, config: Config) {
	let mut sender = make_sender(std::io::empty(), config);

	c.bench_function(id, |b| {
		b.iter(|| {
			sender.send_blocking(()).unwrap();
		})
	});
}

fn bench_send_async(c: &mut Criterion, id: &str, config: Config) {
	let mut sender = make_sender(tokio::io::empty(), config);

	let rt = tokio_rt();

	c.bench_function(id, |b| {
		b.iter(|| {
			rt.block_on(async {
				sender.send(()).await.unwrap();
			})
		})
	});
}

struct SendBenchmark {
	variant: &'static str,
	config: Config,
}

fn send_benchmarks() -> Vec<SendBenchmark> {
	[
		SendBenchmark {
			variant: "default",
			config: Config::default(),
		},
		SendBenchmark {
			variant: "no_flush",
			config: Config::default().with_flush_on_send(false),
		},
	]
	.into()
}

fn send_benches_all(c: &mut Criterion) {
	send_benchmarks().iter().for_each(|bench| {
		let SendBenchmark { variant, config } = bench;

		bench_send_sync(
			c,
			&format!("sync_send ({variant})"),
			config.clone(),
		);
		bench_send_async(
			c,
			&format!("async_send ({variant})"),
			config.clone(),
		);
	});
}

criterion_group!(send_bench, send_benches_all);
criterion_main!(send_bench);

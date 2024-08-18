use std::convert::Infallible;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use criterion::{
	black_box, criterion_group, criterion_main, Criterion,
};

use channels::io::{AsyncRead, IntoRead, Native, NativeAsync, Read};
use channels::receiver::{Config, Receiver};
use channels::serdes::Deserializer;

struct FastDeserializer;

impl Deserializer<()> for FastDeserializer {
	type Error = Infallible;

	fn deserialize(
		&mut self,
		_: &mut [u8],
	) -> Result<(), Self::Error> {
		Ok(())
	}
}

// Taken from: `cargo run --package stdio -- sender - 64`
static PACKET: [u8; 80] = [
	0xfd, 0x3f, 0x00, 0x50, 0x02, 0x70, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05,
	0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
	0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
	0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26,
	0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31,
	0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b, 0x3c,
	0x3d, 0x3e, 0x3f,
];

struct FixedPacketReader {
	pos: usize,
}

impl FixedPacketReader {
	fn new() -> Self {
		Self { pos: 0 }
	}

	fn advance(&mut self, n: usize) {
		self.pos = (self.pos + n) % PACKET.len();
	}

	fn copy_to_buf(&mut self, buf: &mut [u8]) -> usize {
		let mut i = 0;

		while !buf[i..].is_empty() {
			let x = copy_min_len(&PACKET[self.pos..], &mut buf[i..]);
			i += x;
			self.advance(x);
		}

		i
	}
}

impl Read for FixedPacketReader {
	type Error = io::Error;

	fn read_slice(
		&mut self,
		buf: &mut [u8],
	) -> Result<usize, Self::Error> {
		Ok(self.copy_to_buf(buf))
	}
}

impl AsyncRead for FixedPacketReader {
	type Error = io::Error;

	fn poll_read_slice(
		mut self: Pin<&mut Self>,
		_: &mut Context,
		buf: &mut [u8],
	) -> Poll<Result<usize, Self::Error>> {
		Poll::Ready(Ok(self.copy_to_buf(buf)))
	}
}

fn copy_min_len(src: &[u8], dst: &mut [u8]) -> usize {
	let i = src.len().min(dst.len());
	dst[..i].copy_from_slice(&src[..i]);
	i
}

fn make_receiver<R>(
	reader: impl IntoRead<R>,
	config: Config,
) -> Receiver<(), R, FastDeserializer> {
	Receiver::builder()
		.deserializer(FastDeserializer)
		.reader(reader)
		.config(config)
		.build()
}

fn tokio_rt() -> tokio::runtime::Runtime {
	tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
		.unwrap()
}

fn bench_recv_sync(c: &mut Criterion, id: &str, config: Config) {
	let mut receiver: Receiver<(), Native<_>, _> =
		make_receiver(FixedPacketReader::new(), config);

	c.bench_function(id, |b| {
		b.iter(|| black_box(receiver.recv_blocking()).unwrap())
	});
}

fn bench_recv_async(c: &mut Criterion, id: &str, config: Config) {
	let mut receiver: Receiver<(), NativeAsync<_>, _> =
		make_receiver(FixedPacketReader::new(), config);

	let rt = tokio_rt();

	c.bench_function(id, |b| {
		b.iter(|| {
			rt.block_on(async {
				black_box(receiver.recv().await).unwrap();
			})
		})
	});
}

struct RecvBenchmark {
	variant: &'static str,
	config: Config,
}

fn recv_benchmarks() -> Vec<RecvBenchmark> {
	[
		RecvBenchmark {
			variant: "default",
			config: Config::default(),
		},
	]
	.into()
}

fn recv_benches_all(c: &mut Criterion) {
	recv_benchmarks().iter().for_each(|bench| {
		let RecvBenchmark { variant, config } = bench;

		bench_recv_sync(
			c,
			&format!("sync_recv ({variant})"),
			config.clone(),
		);
		bench_recv_async(
			c,
			&format!("async_recv ({variant})"),
			config.clone(),
		);
	});
}

criterion_group!(recv_bench, recv_benches_all);
criterion_main!(recv_bench);

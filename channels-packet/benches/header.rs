#![allow(missing_docs, clippy::unwrap_used)]

use criterion::{
	black_box, criterion_group, criterion_main, Criterion,
};

use channels_packet::{header::Header, Flags, FrameNum};

#[allow(clippy::unusual_byte_groupings)]
const HEADER: [u8; 8] =
	[0x42, 0b010111_00, 0xbd, 0xa3, 0xff, 0xff, 0, 0];

fn parse_header_bench_all(c: &mut Criterion) {
	let input = black_box(&HEADER);

	c.bench_function("Header::try_parse", |b| {
		b.iter(|| {
			Header::try_parse(input).unwrap().unwrap();
		});
	});
}

fn serialize_header_bench_all(c: &mut Criterion) {
	let input = black_box(Header {
		flags: Flags::empty(),
		frame_num: FrameNum::new(5),
		data_len: 42,
	});

	c.bench_function("Header::to_bytes", |b| {
		b.iter(|| {
			let _ = Header::to_bytes(input);
		});
	});
}

criterion_group!(
	header_bench,
	parse_header_bench_all,
	serialize_header_bench_all
);
criterion_main!(header_bench);

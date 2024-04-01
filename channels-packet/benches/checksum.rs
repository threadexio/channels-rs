#![allow(missing_docs)]

use criterion::{
	black_box, criterion_group, criterion_main, Criterion,
};

use channels_packet::{checksum, header::Header};

fn calculate_checksum(c: &mut Criterion) {
	const TEST_HEADER: [u8; Header::SIZE] =
		[0xfd, 0x3f, 0x00, 0x2a, 0x00, 0x00, 0x80, 0x01];

	c.bench_function("calculate checksum", |b| {
		b.iter(|| {
			let _ = black_box(checksum::checksum(black_box(
				&TEST_HEADER,
			)));
		});
	});
}

fn verify_checksum(c: &mut Criterion) {
	const TEST_HEADER: [u8; Header::SIZE] =
		[0xfd, 0x3f, 0x00, 0x2a, 0x82, 0x94, 0x80, 0x01];

	c.bench_function("verify checksum", |b| {
		b.iter(|| {
			let _ = black_box(checksum::checksum(black_box(
				&TEST_HEADER,
			)));
		});
	});
}

criterion_group!(benches, calculate_checksum, verify_checksum);
criterion_main!(benches);

#![allow(missing_docs)]

use criterion::{
	black_box, criterion_group, criterion_main, Criterion,
};

use channels_packet::{
	header::{Header, VerifyId, WithChecksum},
	id::IdSequence,
};

const TEST_HEADER: [u8; Header::SIZE] =
	[0xfd, 0x3f, 0x00, 0x2a, 0x82, 0x94, 0x80, 0x00];

struct ParseVariant {
	id: &'static str,
	with_checksum: bool,
	verify_id: bool,
}

fn parse(c: &mut Criterion) {
	let mut seq = IdSequence::new();

	let variants = [
		ParseVariant {
			id: "parse (no id & no checksum)",
			with_checksum: false,
			verify_id: false,
		},
		ParseVariant {
			id: "parse (no checksum)",
			with_checksum: false,
			verify_id: true,
		},
		ParseVariant {
			id: "parse (no id)",
			with_checksum: true,
			verify_id: false,
		},
		ParseVariant {
			id: "parse",
			with_checksum: true,
			verify_id: true,
		},
	];

	for variant in variants {
		c.bench_function(variant.id, |b| {
			b.iter(|| {
				let with_checksum = if variant.with_checksum {
					WithChecksum::Yes
				} else {
					WithChecksum::No
				};

				let verify_id = if variant.verify_id {
					VerifyId::Yes(&mut seq)
				} else {
					VerifyId::No
				};

				let _ = black_box(Header::try_from_bytes(
					black_box(TEST_HEADER),
					with_checksum,
					verify_id,
				));
			});
		});
	}
}

criterion_group!(benches, parse);
criterion_main!(benches);

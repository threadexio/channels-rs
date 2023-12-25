use criterion::{
	black_box, criterion_group, criterion_main, Criterion,
};

use benches::{complex, simple, Complex, Simple};
use std::io::empty;

fn send_simple(c: &mut Criterion) {
	let mut sender = channels::Sender::<Simple, _, _>::new(empty());

	let data = simple();

	c.bench_function("send simple", |b| {
		b.iter(|| sender.send_blocking(black_box(&data)).unwrap())
	});
}

fn send_complex(c: &mut Criterion) {
	let mut sender = channels::Sender::<Complex, _, _>::new(empty());

	let data = complex();

	c.bench_function("send complex", |b| {
		b.iter(|| sender.send_blocking(black_box(&data)).unwrap())
	});
}

criterion_group!(benches, send_simple, send_complex);
criterion_main!(benches);

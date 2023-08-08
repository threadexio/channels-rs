use std::collections::VecDeque;

use criterion::{
	black_box, criterion_group, criterion_main, Bencher, Criterion,
};

use benches::Player;

fn in_memory(b: &mut Bencher) {
	let rw: VecDeque<u8> = VecDeque::with_capacity(2048);
	let (r, w) = channels::adapter::unsync::split(rw);

	let mut server =
		channels::channel::<Player, _, _>(r.clone(), w.clone());
	let mut client = channels::channel::<Player, _, _>(r, w);

	let player = Player::random();

	b.iter(|| {
		black_box(server.0.send_blocking(&player)).unwrap();
		black_box(client.1.recv_blocking()).unwrap();
	});
}

fn bench(c: &mut Criterion) {
	c.bench_function("in_memory", in_memory);
}

criterion_group!(benches, bench);
criterion_main!(benches);

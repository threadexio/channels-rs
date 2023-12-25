use ringbuf::StaticRb;
use stress_tests::{time, Data, TestResults};

const ITER: usize = 1_000_000;

#[test]
fn test_in_memory() {
	let mut rb = StaticRb::<u8, 1024>::default();

	let (w, r) = rb.split_ref();
	let (mut tx, mut rx) = channels::channel::<Data, _, _>(r, w);

	let (duration, _) = time(|| {
		for i in 0..ITER {
			let data =
				Data { a: 42, b: i, c: format!("test str #{i}") };

			tx.send_blocking(&data).unwrap();

			let r = rx.recv_blocking().unwrap();
			assert_eq!(r, data);
		}
	});

	let stats = TestResults { duration, stats: tx.statistics() };
	eprintln!("{stats}\n");
}

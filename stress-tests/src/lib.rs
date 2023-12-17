use std::thread;
use std::time::{Duration, Instant};

/// Spawn a server and a client thread and wait for them to complete.
pub fn spawn_server_client<S, C, So, Co>(
	server: S,
	client: C,
) -> (So, Co)
where
	So: Send,
	Co: Send,
	S: FnOnce() -> So + Send,
	C: FnOnce() -> Co + Send,
{
	thread::scope(|scope| {
		let t1 = thread::Builder::new()
			.name("server".into())
			.spawn_scoped(scope, server)
			.unwrap();

		thread::sleep(Duration::from_secs(1));

		let t2 = thread::Builder::new()
			.name("client".into())
			.spawn_scoped(scope, client)
			.unwrap();

		(t1.join().unwrap(), t2.join().unwrap())
	})
}

/// Block until `f` returns `true`.
pub fn block_until<F>(mut f: F)
where
	F: FnMut() -> bool,
{
	loop {
		if f() {
			break;
		}

		thread::sleep(Duration::from_millis(500));
	}
}

/// Measure the execution time of _f_.
pub fn time<F, Output>(f: F) -> (Duration, Output)
where
	F: FnOnce() -> Output,
{
	let start = Instant::now();
	let output = f();
	let elapsed = start.elapsed();

	(elapsed, output)
}

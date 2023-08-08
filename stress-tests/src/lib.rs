use std::thread;
use std::time::Duration;

/// Spawn a server and a client thread and wait for them to complete.
pub fn spawn_server_client<S, C>(server: S, client: C)
where
	S: FnOnce() + Send + 'static,
	C: FnOnce() + Send + 'static,
{
	let t1 = thread::Builder::new()
		.name("server".into())
		.spawn(server)
		.unwrap();

	thread::sleep(Duration::from_secs(1));

	let t2 = thread::Builder::new()
		.name("client".into())
		.spawn(client)
		.unwrap();

	t2.join().unwrap();
	t1.join().unwrap();
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

use std::net::TcpListener;
use std::thread;

fn main() {
	let listener = TcpListener::bind("0.0.0.0:8081").unwrap();

	for connection in listener.incoming().filter_map(|x| x.ok()) {
		let (mut tx, mut rx) = channels::channel::<i32>(
			connection.try_clone().unwrap(),
			connection,
		);

		// receiving thread
		thread::spawn(move || loop {
			let v = rx.recv().unwrap();
			println!("Received: {}", v);
		});

		// sending thread
		thread::spawn(move || loop {
			thread::sleep(std::time::Duration::from_secs_f32(0.5));
			tx.send(1337).unwrap();
		});
	}
}

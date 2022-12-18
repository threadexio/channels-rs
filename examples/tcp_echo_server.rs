use std::net::TcpListener;

fn main() {
	let listener = TcpListener::bind("0.0.0.0:8080").unwrap();

	for connection in listener.incoming().filter_map(|x| x.ok()) {
		let (mut tx, mut rx) = channels::channel::<i32>(
			connection.try_clone().unwrap(),
			connection,
		);

		loop {
			let received = match rx.recv() {
				Ok(v) => v,
				Err(_) => break,
			};

			match received {
				1337 => break,
				n => println!("Received i32: {}", n),
			};

			let _ = tx.send(received);
		}
	}
}

use std::net::TcpListener;

use channels;

fn main() {
	let listener = TcpListener::bind("0.0.0.0:8080").unwrap();

	for connection in listener.incoming().filter_map(|x| x.ok()) {
		let (mut tx, mut rx) = channels::channel::<i32, _>(connection);

		loop {
			let received = rx.recv().unwrap();

			match received {
				1337 => break,
				n => println!("Received i32: {}", n),
			};

			tx.send(received).unwrap();
		}
	}
}

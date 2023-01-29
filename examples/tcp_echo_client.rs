use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use rand::Rng;

fn main() {
	let connection = TcpStream::connect("127.0.0.1:9999").unwrap();

	let (mut tx, mut rx) = channels::channel::<i32>(
		connection.try_clone().unwrap(),
		connection,
	);

	let mut rng = rand::thread_rng();

	tx.send(rng.gen::<i32>()).unwrap();
	let received = rx.recv().unwrap();

	println!("Received: {received}");

	thread::sleep(Duration::from_secs(rng.gen_range(1..3)));
}

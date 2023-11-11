use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use rand::Rng;

fn main() {
	let connection = TcpStream::connect("127.0.0.1:10000").unwrap();

	let sd = channels::serdes::Json::new();

	let mut tx = channels::Sender::builder()
		.writer(connection.try_clone().unwrap())
		.serializer(sd.clone())
		.build();

	let mut rx = channels::Receiver::builder()
		.reader(connection)
		.deserializer(sd)
		.build();

	let mut rng = rand::thread_rng();
	loop {
		tx.send_blocking(rng.gen::<i32>()).unwrap();
		let received: i32 = rx.recv_blocking().unwrap();

		println!("Received: {received}");

		thread::sleep(Duration::from_secs(rng.gen_range(1..3)));
	}
}

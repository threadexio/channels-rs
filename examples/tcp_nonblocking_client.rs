use std::net::TcpStream;
use std::thread;

use channels;

fn main() {
	let connection = TcpStream::connect("127.0.0.1:8081").unwrap();

	let (mut tx, mut rx) = channels::channel::<i32, _>(connection);

	tx.inner().set_nonblocking(true).unwrap();

	let mut i = 0;
	loop {
		use std::io::ErrorKind;
		match rx.recv() {
			Err(e) => match e.kind() {
				ErrorKind::WouldBlock => {}
				_ => panic!("{}", e),
			},
			Ok(v) => println!("Received: {}", v),
		}

		i += 1;

		tx.send(i).unwrap();

		thread::sleep(std::time::Duration::from_secs_f32(0.25));
	}
}

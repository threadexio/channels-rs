use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use rand::Rng;

fn main() {
	let connection = TcpStream::connect("127.0.0.1:10000").unwrap();
	connection.set_nonblocking(true).unwrap();
	let (mut tx, mut rx) = channels::channel(
		connection.try_clone().unwrap(),
		connection,
	);

	let mut rng = rand::thread_rng();

	let mut i = 0;
	loop {
		use channels::error::RecvError;
		use std::io::ErrorKind;
		match rx.recv() {
			Ok(v) => println!("Received: {v}"),
			Err(e) => match e {
				RecvError::VersionMismatch => {
					eprintln!("client uses wrong version");
					break;
				},
				RecvError::ChecksumError => {
					eprintln!("packet checksum does not match. discarding...");
					continue;
				},
				RecvError::Io(e)
					if e.kind() == ErrorKind::WouldBlock =>
				{
					continue
				},
				e => {
					eprintln!("{e}");
					continue;
				},
			},
		}

		i += 1;

		tx.send(i).unwrap();

		// some expensive computation
		thread::sleep(Duration::from_secs(rng.gen_range(1..5)));
	}
}

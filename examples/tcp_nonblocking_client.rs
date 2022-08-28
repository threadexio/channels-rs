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
			Err(e) => match e {
				channels::Error::VersionMismatch => {
					eprintln!("client uses wrong version");
					break;
				}
				channels::Error::ChecksumError => {
					eprintln!("packet checksum does not match. discarding...");
					continue;
				}
				channels::Error::Io(e) => match e.kind() {
					ErrorKind::WouldBlock => continue,
					_ => {
						eprintln!("{}", e);
						break;
					}
				},
				e => {
					eprintln!("{}", e);
					continue;
				}
			},
			Ok(v) => println!("Received: {}", v),
		}

		i += 1;

		tx.send(i).unwrap();

		// some expensive computation
		thread::sleep(std::time::Duration::from_secs_f32(0.5));
	}
}

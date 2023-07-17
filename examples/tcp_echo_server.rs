use std::net::{TcpListener, TcpStream};
use std::thread;

fn connection_handler(connection: TcpStream) {
	let (mut tx, mut rx) = channels::channel::<i32, _, _>(
		connection.try_clone().unwrap(),
		connection,
	);

	for received in rx.incoming() {
		println!(
			"{}: Received: {received}",
			thread::current().name().unwrap()
		);

		tx.send(received).unwrap();
	}
}

fn main() {
	let listener = TcpListener::bind("127.0.0.1:10000").unwrap();

	for (i, connection) in listener.incoming().enumerate() {
		match connection {
			Ok(conn) => {
				thread::Builder::new()
					.name(format!("client #{i}"))
					.spawn(move || connection_handler(conn))
					.unwrap();
			},
			Err(e) => {
				eprintln!("Client failed to connect: {e}");
				continue;
			},
		}
	}
}

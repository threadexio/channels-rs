use std::net::{TcpListener, TcpStream};
use std::thread;

fn connection_handler(connection: TcpStream) {
	let sd = channels::serdes::Json::new();

	let mut tx = channels::Sender::<i32, _, _>::builder()
		.writer(connection.try_clone().unwrap())
		.serializer(sd.clone())
		.build();

	let mut rx = channels::Receiver::<i32, _, _>::builder()
		.reader(connection)
		.deserializer(sd)
		.build();

	for received in rx.incoming().map(|x| x.unwrap()) {
		println!(
			"{}: Received: {received}",
			thread::current().name().unwrap()
		);

		tx.send_blocking(received).unwrap();
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

use std::net::{TcpListener, TcpStream};
use std::thread;

type Sd<T> =
	channels::serdes::gzip::Gzip<T, channels::serdes::Bincode>;

type Tx<T> = channels::Sender<T, TcpStream, Sd<T>>;
type Rx<T> = channels::Receiver<T, TcpStream, Sd<T>>;

fn connection_handler(connection: TcpStream) {
	let mut tx = Tx::<i32>::with_serializer(
		connection.try_clone().unwrap(),
		Sd::builder().build(channels::serdes::Bincode::default()),
	);

	let mut rx = Rx::<i32>::with_deserializer(
		connection,
		Sd::builder().build(channels::serdes::Bincode::default()),
	);

	for received in rx.incoming() {
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

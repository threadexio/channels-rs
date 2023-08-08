use std::net::{TcpListener, TcpStream};

#[derive(
	Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
)]
struct Data {
	a: i32,
	b: usize,
	c: String,
}

const ADDR: &str = "127.0.0.1:10000";
const ITER: usize = 1024;

fn server() {
	let listener = TcpListener::bind(ADDR).unwrap();
	let (s, _) = listener.accept().unwrap();
	let (mut tx, mut rx) =
		channels::channel(s.try_clone().unwrap(), s);

	for i in 0..ITER {
		let data: Data = rx.recv_blocking().unwrap();

		assert_eq!(
			data,
			Data { a: 42, b: i, c: format!("test str #{i}") }
		);

		tx.send_blocking(data).unwrap();
	}
}

fn client() {
	let s = TcpStream::connect(ADDR).unwrap();
	let (mut tx, mut rx) =
		channels::channel(s.try_clone().unwrap(), s);

	for i in 0..ITER {
		let data = Data { a: 42, b: i, c: format!("test str #{i}") };

		tx.send_blocking(data.clone()).unwrap();

		assert_eq!(rx.recv_blocking().unwrap(), data);
	}
}

#[test]
fn transport() {
	stress_tests::spawn_server_client(server, client)
}

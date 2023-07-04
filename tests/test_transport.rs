use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

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
		let data: Data = rx.try_recv().unwrap();

		assert_eq!(
			data,
			Data { a: 42, b: i, c: format!("test str #{i}") }
		);

		tx.try_send(data).unwrap();
	}
}

fn client() {
	let s = TcpStream::connect(ADDR).unwrap();
	let (mut tx, mut rx) =
		channels::channel(s.try_clone().unwrap(), s);

	for i in 0..ITER {
		let data = Data { a: 42, b: i, c: format!("test str #{i}") };

		tx.try_send(data.clone()).unwrap();

		assert_eq!(rx.try_recv().unwrap(), data);
	}
}

#[test]
fn test_transport() {
	let s = thread::Builder::new()
		.name("server".into())
		.spawn(server)
		.unwrap();

	thread::sleep(Duration::from_secs(1));

	let c = thread::Builder::new()
		.name("client".into())
		.spawn(client)
		.unwrap();

	s.join().unwrap();
	c.join().unwrap();
}

use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

#[derive(
	Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
)]
struct Data {
	buffer: Vec<u8>,
}

const ADDR: &str = "127.0.0.1:10001";
const ITER: usize = 64;

fn server() {
	let listener = TcpListener::bind(ADDR).unwrap();
	let (s, _) = listener.accept().unwrap();
	let (mut tx, mut rx) =
		channels::channel(s.try_clone().unwrap(), s);

	for _ in 0..ITER {
		let data: Data = rx.recv_blocking().unwrap();

		assert!(
			data.buffer
				.iter()
				.enumerate()
				.all(|(i, x)| *x == i as u8),
			"the buffer has corrupted data"
		);

		tx.send_blocking(data).unwrap();
	}
}

fn client() {
	let s = TcpStream::connect(ADDR).unwrap();
	let (mut tx, mut rx) =
		channels::channel::<Data, _, _>(s.try_clone().unwrap(), s);

	for i in 0..ITER {
		let data = Data {
			buffer: (0..usize::from(u16::MAX) + 16000 + i)
				.map(|x| x as u8)
				.collect(),
		};

		tx.send_blocking(&data).unwrap();

		assert_eq!(rx.recv_blocking().unwrap(), data);
	}
}

#[test]
fn test_big_data() {
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

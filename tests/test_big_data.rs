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
		let data: Data = rx.recv().unwrap();

		assert!(
			data.buffer
				.iter()
				.enumerate()
				.all(|(i, x)| *x == i as u8),
			"the buffer has corrupted data"
		);

		tx.send(data).unwrap();
	}
}

fn client() {
	let s = TcpStream::connect(ADDR).unwrap();
	let (mut tx, mut rx) =
		channels::channel(s.try_clone().unwrap(), s);

	for i in 0..ITER {
		let mut data = Data {
			buffer: (0..usize::from(u16::MAX) + i)
				.map(|x| x as u8)
				.collect(),
		};
		assert!(matches!(
			tx.send(data.clone()),
			Err(channels::Error::SizeLimit)
		));

		data.buffer.resize(
			u16::MAX as usize - i - 128, /* ensure we allow enough space for the header */
			0x0,
		);
		assert!(tx.send(data.clone()).is_ok());

		assert_eq!(rx.recv().unwrap(), data);
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

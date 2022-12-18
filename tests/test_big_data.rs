use std::net::{TcpListener, TcpStream};
use std::thread::{sleep, spawn};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Data {
	buffer: Vec<u8>,
}

#[test]
fn test_big_data() {
	let a = spawn(|| {
		let listener = TcpListener::bind("127.0.0.42:9999").unwrap();

		let (s, _) = listener.accept().unwrap();

		let (mut tx, mut rx) =
			channels::channel(s.try_clone().unwrap(), s);

		let d: Data = rx.recv().unwrap();

		assert!(d.buffer.len() < u16::MAX as usize);

		tx.send(d).unwrap();
	});

	sleep(std::time::Duration::from_millis(500));

	let s = TcpStream::connect("127.0.0.42:9999").unwrap();

	let (mut tx, mut rx) =
		channels::channel(s.try_clone().unwrap(), s);

	let mut d = Data { buffer: vec![0u8; u16::MAX as usize + 1024] };

	assert!(matches!(
		tx.send(d.clone()),
		Err(channels::Error::SizeLimit)
	));

	d.buffer = vec![0u8; u16::MAX as usize];

	assert!(matches!(
		tx.send(d.clone()),
		Err(channels::Error::SizeLimit)
	));

	d.buffer = vec![0u8; u16::MAX as usize - 1024];

	assert!(tx.send(d.clone()).is_ok());

	assert_eq!(rx.recv().unwrap(), d);

	a.join().unwrap();
}

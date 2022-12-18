use std::net::{TcpListener, TcpStream};
use std::thread::{sleep, spawn};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Data {
	a: i32,
	b: usize,
	c: String,
}

#[test]
fn test_interference() {
	let a = spawn(|| {
		let listener = TcpListener::bind("127.0.0.42:10000").unwrap();

		let (s, _) = listener.accept().unwrap();

		let (mut tx, mut rx) =
			channels::channel(s.try_clone().unwrap(), s);

		let d: Data = rx.recv().unwrap();

		assert_eq!(d.a, 42);
		assert_eq!(d.b, 9999);
		assert_eq!(d.c, String::from("test str"));

		let _ = tx.get_mut().write(&[0, 0, 0, 0]).unwrap();

		tx.send(d).unwrap();
	});

	sleep(std::time::Duration::from_millis(500));

	let s = TcpStream::connect("127.0.0.42:10000").unwrap();

	let (mut tx, mut rx) =
		channels::channel(s.try_clone().unwrap(), s);

	let d = Data { a: 42, b: 9999, c: String::from("test str") };

	tx.send(d).unwrap();

	assert!(rx.recv().is_err());

	a.join().unwrap();
}

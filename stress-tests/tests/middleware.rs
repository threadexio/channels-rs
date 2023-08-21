use serial_test::serial;

use std::net::{TcpListener, TcpStream};

use channels::serdes::Bincode;
use serde::{Deserialize, Serialize};

const ADDR: &str = "127.0.0.1:10002";
const ITER: usize = 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Data {
	a: i32,
	b: usize,
	c: String,
	d: Vec<u64>,
}

impl Data {
	pub fn new(i: usize) -> Self {
		Self {
			a: 42,
			b: i,
			c: format!("string #{i}"),
			d: vec![42, 69, 420],
		}
	}
}

fn server_accept() -> TcpStream {
	let listener = TcpListener::bind(ADDR).unwrap();
	listener.accept().unwrap().0
}

fn client_connect() -> TcpStream {
	TcpStream::connect(ADDR).unwrap()
}

mod crc {
	use super::*;

	use channels::serdes::crc::Crc;

	type Sd = Crc<Bincode>;
	type Pair = channels::Pair<Data, TcpStream, TcpStream, Sd, Sd>;

	fn create_pair(s: TcpStream) -> Pair {
		let sd = Crc::builder().build(Bincode::default());

		let tx = channels::Sender::<Data, _, _>::with_serializer(
			s.try_clone().unwrap(),
			sd.clone(),
		);
		let rx = channels::Receiver::<Data, _, _>::with_deserializer(
			s, sd,
		);

		(tx, rx)
	}

	fn server() {
		let s = server_accept();
		let (mut tx, mut rx) = create_pair(s);

		for i in 0..ITER {
			let expected = Data::new(i);

			let recv = rx.recv_blocking().unwrap();
			assert_eq!(expected, recv);

			tx.send_blocking(recv).unwrap();
		}
	}

	fn client() {
		let s = client_connect();
		let (mut tx, mut rx) = create_pair(s);

		for i in 0..ITER {
			let data = Data::new(i);

			tx.send_blocking(&data).unwrap();
			let recv = rx.recv_blocking().unwrap();

			assert_eq!(data, recv);
		}
	}

	#[test]
	#[serial]
	fn test_crc() {
		stress_tests::spawn_server_client(server, client);
	}
}

mod gzip {
	use super::*;

	use channels::serdes::gzip::Gzip;

	type Sd = Gzip<Bincode>;
	type Pair = channels::Pair<Data, TcpStream, TcpStream, Sd, Sd>;

	fn create_pair(s: TcpStream) -> Pair {
		let sd = Gzip::builder().build(Bincode::default());

		let tx = channels::Sender::<Data, _, _>::with_serializer(
			s.try_clone().unwrap(),
			sd.clone(),
		);
		let rx = channels::Receiver::<Data, _, _>::with_deserializer(
			s, sd,
		);

		(tx, rx)
	}

	fn server() {
		let s = server_accept();
		let (mut tx, mut rx) = create_pair(s);

		for i in 0..ITER {
			let expected = Data::new(i);

			let recv = rx.recv_blocking().unwrap();
			assert_eq!(expected, recv);

			tx.send_blocking(recv).unwrap();
		}
	}

	fn client() {
		let s = client_connect();
		let (mut tx, mut rx) = create_pair(s);

		for i in 0..ITER {
			let data = Data::new(i);

			tx.send_blocking(&data).unwrap();
			let recv = rx.recv_blocking().unwrap();

			assert_eq!(data, recv);
		}
	}

	#[test]
	#[serial]
	fn test_gzip() {
		stress_tests::spawn_server_client(server, client);
	}
}

mod deflate {
	use super::*;

	use channels::serdes::deflate::Deflate;

	type Sd = Deflate<Bincode>;
	type Pair = channels::Pair<Data, TcpStream, TcpStream, Sd, Sd>;

	fn create_pair(s: TcpStream) -> Pair {
		let sd = Deflate::builder().build(Bincode::default());

		let tx = channels::Sender::<Data, _, _>::with_serializer(
			s.try_clone().unwrap(),
			sd.clone(),
		);
		let rx = channels::Receiver::<Data, _, _>::with_deserializer(
			s, sd,
		);

		(tx, rx)
	}

	fn server() {
		let s = server_accept();
		let (mut tx, mut rx) = create_pair(s);

		for i in 0..ITER {
			let expected = Data::new(i);

			let recv = rx.recv_blocking().unwrap();
			assert_eq!(expected, recv);

			tx.send_blocking(recv).unwrap();
		}
	}

	fn client() {
		let s = client_connect();
		let (mut tx, mut rx) = create_pair(s);

		for i in 0..ITER {
			let data = Data::new(i);

			tx.send_blocking(&data).unwrap();
			let recv = rx.recv_blocking().unwrap();

			assert_eq!(data, recv);
		}
	}

	#[test]
	#[serial]
	fn test_deflate() {
		stress_tests::spawn_server_client(server, client);
	}
}

mod hmac {
	use super::*;

	use channels::serdes::hmac::Hmac;

	const SECRET_KEY: &[u8] = &[0u8; 64];

	type Sd = Hmac<Bincode, &'static [u8]>;
	type Pair = channels::Pair<Data, TcpStream, TcpStream, Sd, Sd>;

	fn create_pair(s: TcpStream) -> Pair {
		let sd = Hmac::builder(SECRET_KEY).build(Bincode::default());

		let tx = channels::Sender::<Data, _, _>::with_serializer(
			s.try_clone().unwrap(),
			sd.clone(),
		);
		let rx = channels::Receiver::<Data, _, _>::with_deserializer(
			s, sd,
		);

		(tx, rx)
	}

	fn server() {
		let s = server_accept();
		let (mut tx, mut rx) = create_pair(s);

		for i in 0..ITER {
			let expected = Data::new(i);

			let recv = rx.recv_blocking().unwrap();
			assert_eq!(expected, recv);

			tx.send_blocking(recv).unwrap();
		}
	}

	fn client() {
		let s = client_connect();
		let (mut tx, mut rx) = create_pair(s);

		for i in 0..ITER {
			let data = Data::new(i);

			tx.send_blocking(&data).unwrap();
			let recv = rx.recv_blocking().unwrap();

			assert_eq!(data, recv);
		}
	}

	#[test]
	#[serial]
	fn test_hmac() {
		stress_tests::spawn_server_client(server, client);
	}
}

mod chained {
	use super::*;

	use channels::serdes::crc::Crc;
	use channels::serdes::deflate::Deflate;
	use channels::serdes::gzip::Gzip;
	use channels::serdes::hmac::Hmac;

	const SECRET_KEY: &[u8] = &[0u8; 64];

	type Sd = Hmac<Deflate<Crc<Gzip<Bincode>>>, &'static [u8]>;
	type Pair = channels::Pair<Data, TcpStream, TcpStream, Sd, Sd>;

	fn create_pair(s: TcpStream) -> Pair {
		let sd = Bincode::default();
		let sd = Gzip::builder().build(sd);
		let sd = Crc::builder().build(sd);
		let sd = Deflate::builder().build(sd);
		let sd = Hmac::builder(SECRET_KEY).build(sd);

		let tx = channels::Sender::<Data, _, _>::with_serializer(
			s.try_clone().unwrap(),
			sd.clone(),
		);
		let rx = channels::Receiver::<Data, _, _>::with_deserializer(
			s, sd,
		);

		(tx, rx)
	}

	fn server() {
		let s = server_accept();
		let (mut tx, mut rx) = create_pair(s);

		for i in 0..ITER {
			let expected = Data::new(i);

			let recv = rx.recv_blocking().unwrap();
			assert_eq!(expected, recv);

			tx.send_blocking(recv).unwrap();
		}
	}

	fn client() {
		let s = client_connect();
		let (mut tx, mut rx) = create_pair(s);

		for i in 0..ITER {
			let data = Data::new(i);

			tx.send_blocking(&data).unwrap();
			let recv = rx.recv_blocking().unwrap();

			assert_eq!(data, recv);
		}
	}

	#[test]
	#[serial]
	fn test_chained() {
		stress_tests::spawn_server_client(server, client);
	}
}

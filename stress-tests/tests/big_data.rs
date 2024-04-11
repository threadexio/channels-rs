use serial_test::serial;

use channels::io::{IntoReader, IntoWriter};

#[derive(
	Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
)]
struct Data {
	buffer: Vec<u8>,
}

const ADDR: &str = "127.0.0.1:10001";
const ITER: usize = 64;

type Pair<R, W> =
	channels::Pair<Data, R, W, channels::serdes::Bincode>;

fn make_pair<R, W>(
	reader: impl IntoReader<R>,
	writer: impl IntoWriter<W>,
) -> Pair<R, W> {
	use channels::receiver;

	const PAYLOAD_SIZE: usize = 100 * 1024; // A very rough estimate

	let tx = channels::Sender::builder()
		.writer(writer)
		.serializer(Default::default())
		.build();

	let config = receiver::Config::default()
		.size_estimate(PAYLOAD_SIZE)
		.max_size(PAYLOAD_SIZE);

	let rx = channels::Receiver::builder()
		.reader(reader)
		.deserializer(Default::default())
		.config(config)
		.build();

	(tx, rx)
}

mod sync_tests {
	use super::*;

	use std::{
		net::{TcpListener, TcpStream},
		time::Duration,
	};

	use stress_tests::{spawn_server_client, time, TestResults};

	type Pair = super::Pair<
		channels::io::Std<TcpStream>,
		channels::io::Std<TcpStream>,
	>;

	fn make_pair(stream: TcpStream) -> Pair {
		super::make_pair(stream.try_clone().unwrap(), stream)
	}

	fn server() -> (Duration, Pair) {
		let listener = TcpListener::bind(ADDR).unwrap();
		let (s, _) = listener.accept().unwrap();
		let (mut tx, mut rx) = make_pair(s);

		time(move || {
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

			(tx, rx)
		})
	}

	fn client() -> (Duration, Pair) {
		let s = TcpStream::connect(ADDR).unwrap();
		let (mut tx, mut rx) = make_pair(s);

		time(move || {
			for i in 0..ITER {
				let data = Data {
					buffer: (0..usize::from(u16::MAX) + 16000 + i)
						.map(|x| x as u8)
						.collect(),
				};

				tx.send_blocking(&data).unwrap();

				assert_eq!(rx.recv_blocking().unwrap(), data);
			}

			(tx, rx)
		})
	}

	#[serial]
	#[test]
	fn big_data() {
		let (server, _) = spawn_server_client(server, client);

		eprintln!(
			"{}",
			TestResults {
				duration: server.0,
				stats: server.1 .0.statistics(),
			}
		);
	}
}

mod async_tests {
	use super::*;

	use tokio::net::{
		tcp::{OwnedReadHalf, OwnedWriteHalf},
		TcpListener, TcpStream,
	};

	type Pair = super::Pair<
		channels::io::Tokio<OwnedReadHalf>,
		channels::io::Tokio<OwnedWriteHalf>,
	>;

	fn make_pair(stream: TcpStream) -> Pair {
		let (r, w) = stream.into_split();
		super::make_pair(r, w)
	}

	#[serial]
	#[tokio::test]
	async fn big_data() {
		let listener = TcpListener::bind(ADDR).await.unwrap();
		let accept = listener.accept();

		let mut client =
			make_pair(TcpStream::connect(ADDR).await.unwrap());

		let mut server = make_pair(accept.await.unwrap().0);

		for i in 0..ITER {
			let data = Data {
				buffer: (0..usize::from(u16::MAX) + 16000 + i)
					.map(|x| x as u8)
					.collect(),
			};

			client.0.send(&data).await.unwrap();

			let received = server.1.recv().await.unwrap();
			assert_eq!(received, data);
			server.0.send(received).await.unwrap();

			let received = client.1.recv().await.unwrap();
			assert_eq!(received, data);
		}
	}
}

use serial_test::serial;

#[derive(
	Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
)]
struct Data {
	buffer: Vec<u8>,
}

const ADDR: &str = "127.0.0.1:10001";
const ITER: usize = 64;

mod sync_tests {
	use super::*;

	use std::{
		net::{TcpListener, TcpStream},
		time::Duration,
	};

	use stress_tests::{spawn_server_client, time, Stats};

	type Pair = channels::Pair<
		Data,
		TcpStream,
		TcpStream,
		channels::serdes::Bincode,
		channels::serdes::Bincode,
	>;

	fn server() -> (Duration, Pair) {
		let listener = TcpListener::bind(ADDR).unwrap();
		let (s, _) = listener.accept().unwrap();
		let (mut tx, mut rx) =
			channels::channel(s.try_clone().unwrap(), s);

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
		let (mut tx, mut rx) = channels::channel::<Data, _, _>(
			s.try_clone().unwrap(),
			s,
		);

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
		let (server, client) = spawn_server_client(server, client);

		let server_stats = Stats {
			duration: server.0,
			tx: server.1 .0.statistics(),
			rx: server.1 .1.statistics(),
		};

		let client_stats = Stats {
			duration: client.0,
			tx: client.1 .0.statistics(),
			rx: client.1 .1.statistics(),
		};

		eprintln!("server:\n===============\n{server_stats}\n");
		eprintln!("client:\n===============\n{client_stats}\n");
	}
}

mod async_tests {
	use super::*;

	use tokio::net::{TcpListener, TcpStream};

	#[serial]
	#[tokio::test]
	async fn big_data() {
		let listener = TcpListener::bind(ADDR).await.unwrap();
		let accept = listener.accept();

		let mut client = {
			let (r, w) =
				TcpStream::connect(ADDR).await.unwrap().into_split();
			channels::channel::<Data, _, _>(r, w)
		};

		let mut server = {
			let (r, w) = accept.await.unwrap().0.into_split();
			channels::channel::<Data, _, _>(r, w)
		};

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

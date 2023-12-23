use serial_test::serial;

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
	>;

	fn server() -> (Duration, Pair) {
		let listener = TcpListener::bind(ADDR).unwrap();
		let (s, _) = listener.accept().unwrap();
		let (mut tx, mut rx) =
			channels::channel(s.try_clone().unwrap(), s);

		time(move || {
			for i in 0..ITER {
				let data: Data = rx.recv_blocking().unwrap();

				assert_eq!(
					data,
					Data { a: 42, b: i, c: format!("test str #{i}") }
				);

				tx.send_blocking(data).unwrap();
			}

			(tx, rx)
		})
	}

	fn client() -> (Duration, Pair) {
		let s = TcpStream::connect(ADDR).unwrap();
		let (mut tx, mut rx) =
			channels::channel(s.try_clone().unwrap(), s);

		time(move || {
			for i in 0..ITER {
				let data =
					Data { a: 42, b: i, c: format!("test str #{i}") };

				tx.send_blocking(data.clone()).unwrap();

				assert_eq!(rx.recv_blocking().unwrap(), data);
			}

			(tx, rx)
		})
	}

	#[serial]
	#[test]
	fn transport() {
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
	async fn transport() {
		let listener = TcpListener::bind(ADDR).await.unwrap();
		let accept = listener.accept();

		let client_stream = TcpStream::connect(ADDR).await.unwrap();
		let (server_stream, _) = accept.await.unwrap();

		let mut server = {
			let (r, w) = server_stream.into_split();
			channels::channel::<Data, _, _>(r, w)
		};

		let mut client = {
			let (r, w) = client_stream.into_split();
			channels::channel::<Data, _, _>(r, w)
		};

		for i in 0..ITER {
			let data =
				Data { a: 42, b: i, c: format!("test str #{i}") };
			client.0.send(&data).await.unwrap();

			let received = server.1.recv().await.unwrap();
			assert_eq!(received, data);
			server.0.send(&received).await.unwrap();

			let received = client.1.recv().await.unwrap();
			assert_eq!(received, data);
		}
	}
}

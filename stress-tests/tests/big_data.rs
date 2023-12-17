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

	use std::net::{TcpListener, TcpStream};

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
		let (mut tx, mut rx) = channels::channel::<Data, _, _>(
			s.try_clone().unwrap(),
			s,
		);

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

	#[serial]
	#[test]
	fn big_data() {
		stress_tests::spawn_server_client(server, client);
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

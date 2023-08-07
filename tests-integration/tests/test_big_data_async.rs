use tokio::net::{TcpListener, TcpStream};

#[derive(
	Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
)]
struct Data {
	buffer: Vec<u8>,
}

const ADDR: &str = "127.0.0.1:10101";
const ITER: usize = 64;

#[tokio::test]
async fn test_big_data() {
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

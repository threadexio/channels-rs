use tokio::net::{TcpListener, TcpStream};

#[derive(
	Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
)]
struct Data {
	a: i32,
	b: usize,
	c: String,
}

const ADDR: &str = "127.0.0.1:10100";
const ITER: usize = 1024;

#[tokio::test]
async fn transport_async() {
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
		let data = Data { a: 42, b: i, c: format!("test str #{i}") };
		client.0.send(&data).await.unwrap();

		let received = server.1.recv().await.unwrap();
		assert_eq!(received, data);
		server.0.send(&received).await.unwrap();

		let received = client.1.recv().await.unwrap();
		assert_eq!(received, data);
	}
}

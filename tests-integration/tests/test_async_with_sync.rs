use std::thread;
use std::time::Duration;

use tokio::runtime::Runtime;

#[derive(
	Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
)]
struct Data {
	a: i32,
	b: usize,
	c: String,
}

const ADDR: &str = "127.0.0.1:10110";
const ITER: usize = 1024;

fn server() {
	use std::net::TcpListener;

	let listener = TcpListener::bind(ADDR).unwrap();
	let (s, _) = listener.accept().unwrap();
	let (mut tx, mut rx) =
		channels::channel(s.try_clone().unwrap(), s);

	for i in 0..ITER {
		let data: Data = rx.recv_blocking().unwrap();

		assert_eq!(
			data,
			Data { a: 42, b: i, c: format!("test str #{i}") }
		);

		tx.send_blocking(data).unwrap();
	}
}

async fn client() {
	use tokio::net::TcpStream;

	let s = TcpStream::connect(ADDR).await.unwrap();
	let (r, w) = s.into_split();
	let (mut tx, mut rx) = channels::channel::<Data, _, _>(r, w);

	for i in 0..ITER {
		let data = Data { a: 42, b: i, c: format!("test str #{i}") };

		tx.send(data.clone()).await.unwrap();

		assert_eq!(rx.recv().await.unwrap(), data);
	}
}

#[test]
fn test_transport() {
	let s = thread::Builder::new()
		.name("server".into())
		.spawn(server)
		.unwrap();

	thread::sleep(Duration::from_secs(1));

	let c = thread::Builder::new()
		.name("client".into())
		.spawn(|| {
			Runtime::new().unwrap().block_on(async { client().await })
		})
		.unwrap();

	s.join().unwrap();
	c.join().unwrap();
}

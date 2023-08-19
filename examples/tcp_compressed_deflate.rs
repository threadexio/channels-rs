use std::env;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{sleep, Duration};

const ADDR: &str = "127.0.0.1:10001";

type Sd =
	channels::serdes::deflate::Deflate<channels::serdes::Bincode>;

type Tx<T, W> = channels::Sender<T, W, Sd>;
type Rx<T, W> = channels::Receiver<T, W, Sd>;
type Pair<T> = channels::Pair<
	T,
	tokio::net::tcp::OwnedReadHalf,
	tokio::net::tcp::OwnedWriteHalf,
	Sd,
	Sd,
>;

fn channel(stream: TcpStream) -> Pair<i32> {
	let (r, w) = stream.into_split();

	use channels::serdes::deflate::{Compression, Deflate};
	use channels::serdes::Bincode;

	let sd = Deflate::builder()
		.level(Compression::best())
		.build(Bincode::default());

	let tx = Tx::with_serializer(w, sd.clone());
	let rx = Rx::with_deserializer(r, sd);

	(tx, rx)
}

async fn server() {
	let listener = TcpListener::bind(ADDR).await.unwrap();
	let (stream, _) = listener.accept().await.unwrap();
	let (mut tx, mut rx) = channel(stream);

	loop {
		let received = rx.recv().await.unwrap();
		println!("received: {received}");
		tx.send(received).await.unwrap();
	}
}

async fn client() {
	let stream = TcpStream::connect(ADDR).await.unwrap();
	let (mut tx, mut rx) = channel(stream);

	loop {
		let data: i32 = rand::random();
		tx.send(data).await.unwrap();
		println!("sent: {data}");

		let received = rx.recv().await.unwrap();
		println!("received: {received}");

		sleep(Duration::from_secs(1)).await;
	}
}

#[tokio::main]
async fn main() {
	match env::args().nth(1) {
		Some(v) if v == "server" => server().await,
		Some(v) if v == "client" => client().await,
		_ => eprintln!("expected one of: client, server"),
	}
}

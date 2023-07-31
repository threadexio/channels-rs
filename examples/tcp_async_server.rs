use tokio::net::TcpListener;
use tokio::time::{sleep, Duration};

async fn accept_client(
	listener: &TcpListener,
) -> channels::Pair<
	i32,
	tokio::net::tcp::OwnedReadHalf,
	tokio::net::tcp::OwnedWriteHalf,
	channels::serdes::Bincode,
	channels::serdes::Bincode,
> {
	let (stream, _) = listener.accept().await.unwrap();
	let (r, w) = stream.into_split();
	channels::channel::<i32, _, _>(r, w)
}

#[cfg(feature = "tokio")]
#[tokio::main]
async fn main() {
	let listener =
		TcpListener::bind("127.0.0.1:10000").await.unwrap();

	let mut client1 = accept_client(&listener).await;
	let mut client2 = accept_client(&listener).await;

	loop {
		tokio::select! {
			r = client1.1.recv() =>{
				let data = r.unwrap();
				println!("client #1: {data}");
				client1.0.send(data).await.unwrap();
			}
			r = client2.1.recv() => {
				let data = r.unwrap();
				println!("client #2: {data}");
				client2.0.send(data).await.unwrap();
			}
			_ = sleep(Duration::from_secs(1)) => {
				println!("working...");
			}
		}
	}
}

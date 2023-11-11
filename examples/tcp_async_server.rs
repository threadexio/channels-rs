use tokio::net::TcpListener;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
	let listener =
		TcpListener::bind("127.0.0.1:10000").await.unwrap();

	let mut client1 = {
		let (stream, _) = listener.accept().await.unwrap();
		let (r, w) = stream.into_split();
		channels::channel_async::<i32, _, _>(r, w)
	};

	let mut client2 = {
		let (stream, _) = listener.accept().await.unwrap();
		let (r, w) = stream.into_split();
		channels::channel_async::<i32, _, _>(r, w)
	};

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

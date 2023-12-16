use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};

use rand::Rng;

#[tokio::main]
async fn main() {
	let stream = TcpStream::connect("127.0.0.1:10000").await.unwrap();

	let (r, w) = stream.into_split();
	let (mut tx, mut rx) = channels::channel::<i32, _, _>(r, w);

	let mut rng = rand::thread_rng();

	loop {
		tokio::select! {
			r = rx.recv() => {
				let data = r.unwrap();
				println!("received: {data}");
			}
			_ = sleep(Duration::from_secs(2)) => {
				let data = rng.gen::<i32>();
				tx.send(data).await.unwrap();
				println!("sent: {data}");
			}
			_ = sleep(Duration::from_secs(3)) => {
				println!("working...");
			}
		}
	}
}

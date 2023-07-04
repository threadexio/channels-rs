use std::net::TcpStream;
use std::time::Duration;

use rand::Rng;

#[tokio::main]
async fn main() {
	let connection = TcpStream::connect("127.0.0.1:10000").unwrap();
	connection.set_nonblocking(true).unwrap();
	let (mut tx, mut rx) = channels::channel(
		connection.try_clone().unwrap(),
		connection,
	);

	let mut rng = rand::thread_rng();

	let mut i = 0;
	tx.send(i).await.unwrap();

	loop {
		tokio::select! {
			r = rx.recv() => {
				match r {
					Ok(v) => println!("Received: {v}"),
					Err(e) => eprintln!("error: {e}"),
				}

				i += 1;

				tx.send(i).await.unwrap();
			}
			_ = tokio::time::sleep(Duration::from_secs(rng.gen_range(1..5))) => {
				println!("Working...");
			}
		};
	}
}

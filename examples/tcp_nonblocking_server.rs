use std::collections::LinkedList;
use std::io;
use std::net::TcpListener;

fn main() {
	let listener = TcpListener::bind("0.0.0.0:8081").unwrap();

	let mut clients = LinkedList::new();

	println!("Waiting for 3 clients...");
	for _ in 0..3 {
		let (connection, _) = listener.accept().unwrap();
		connection.set_nonblocking(true).unwrap();

		clients.push_back(channels::channel::<i32>(
			connection.try_clone().unwrap(),
			connection,
		));
	}

	loop {
		println!("Checking if clients have sent anything");
		for (ref mut tx, ref mut rx) in clients.iter_mut() {
			let received = match rx.recv() {
				Ok(v) => v,
				Err(e) => match e {
					channels::Error::Io(io_err)
						if io_err.kind()
							== io::ErrorKind::WouldBlock =>
					{
						continue
					},
					_ => panic!("{}", e),
				},
			};

			tx.send(-received).unwrap();
		}

		// do something else
		std::thread::sleep(std::time::Duration::from_secs(1));
	}
}

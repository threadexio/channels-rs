use std::io;
use std::net::{TcpListener, TcpStream};

type Sender =
	channels::Sender<i32, TcpStream, channels::serdes::Bincode>;
type Receiver =
	channels::Receiver<i32, TcpStream, channels::serdes::Bincode>;

fn main() {
	let listener = TcpListener::bind("127.0.0.1:10000").unwrap();

	let mut clients = Vec::with_capacity(3);

	while clients.len() < 3 {
		println!(
			"Waiting for {} more clients to connect...",
			3 - clients.len()
		);

		let (connection, _) = listener.accept().unwrap();
		connection.set_nonblocking(true).unwrap();

		clients.push(channels::channel(
			connection.try_clone().unwrap(),
			connection,
		));
	}

	println!("Entering main event loop...");
	loop {
		// loop over all clients, if there is an error in
		// with any client, that client is immediately dropped
		clients.retain_mut(|(tx, rx)| handle_client(tx, rx).is_ok());

		// do something else
		println!("Doing work!");
		std::thread::sleep(std::time::Duration::from_secs_f32(0.25));
	}
}

fn handle_client(
	tx: &mut Sender,
	rx: &mut Receiver,
) -> channels::Result<()> {
	let received = match rx.recv() {
		Ok(v) => {
			println!("Received {v}",);
			v
		},
		Err(e) => match e {
			channels::Error::Io(io_err)
				if io_err.kind() == io::ErrorKind::WouldBlock =>
			{
				return Ok(())
			},
			_ => return Err(e),
		},
	};

	tx.send(-received)?;

	Ok(())
}

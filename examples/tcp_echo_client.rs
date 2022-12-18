use std::net::TcpStream;

fn main() {
	let connection = TcpStream::connect("127.0.0.1:8080").unwrap();

	let (mut tx, mut rx) = channels::channel::<i32>(
		connection.try_clone().unwrap(),
		connection,
	);

	tx.send(69420).unwrap();

	match rx.recv().unwrap() {
		69420 => println!("echo server works!"),
		n => panic!("expected 69420, got {:?}", n),
	}
}

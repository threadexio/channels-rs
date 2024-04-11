#![no_std]
extern crate alloc;

mod serdes;

pub mod net;
pub mod record;

use channels::io::Native;

use self::net::Socket;
use self::record::Record;
use self::serdes::Serdes;

type Sender = channels::Sender<Record, Native<Socket>, Serdes>;
type Receiver = channels::Receiver<Record, Native<Socket>, Serdes>;

/// A channel that sends and received [`Records`].
pub struct Channel {
	tx: Sender,
	rx: Receiver,
}

impl Channel {
	/// Connect to `addr` and make a channel on it.
	pub fn connect(addr: &str) -> Self {
		let socket = Socket::new(addr);

		let tx =
			Sender::with_serializer(socket.dup(), Serdes::default());
		let rx =
			Receiver::with_deserializer(socket, Serdes::default());

		Self { tx, rx }
	}

	/// Send a record through the channel.
	pub fn send(&mut self, record: Record) {
		self.tx.send_blocking(record).unwrap();
	}

	/// Receive a record through the channel.
	pub fn recv(&mut self) -> Record {
		self.rx.recv_blocking().unwrap()
	}

	/// Destruct the channel into its underlying socket.
	pub fn into_socket(self) -> Socket {
		let tx_sock = self.tx.into_writer();
		let rx_sock = self.rx.into_reader();

		tx_sock.join(rx_sock);
		tx_sock
	}
}

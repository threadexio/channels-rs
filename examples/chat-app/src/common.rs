use std::net::SocketAddr;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientNetMessage {
	Authenticate { name: String },
	SendMessage { content: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerNetMessage {
	MessagePosted { owner: String, content: String },
	UserConnected { name: String },
}

type Serdes = channels::serdes::Bincode;

#[derive(Debug)]
pub struct ClientSide {
	tx: channels::Sender<
		ClientNetMessage,
		channels::io::Tokio<OwnedWriteHalf>,
		Serdes,
	>,
	rx: channels::Receiver<ServerNetMessage, OwnedReadHalf, Serdes>,
}

impl ClientSide {
	pub fn from_tcp_stream(stream: TcpStream) -> Self {
		let (r, w) = stream.into_split();

		Self {
			tx: channels::Sender::builder()
				.writer(w)
				.serializer(Default::default())
				.build(),
			rx: channels::Receiver::builder()
				.reader(r)
				.deserializer(Default::default())
				.build(),
		}
	}

	pub async fn send(
		&mut self,
		message: ClientNetMessage,
	) -> anyhow::Result<()> {
		self.tx
			.send(message)
			.await
			.context("failed to send message")
	}

	pub async fn recv(&mut self) -> anyhow::Result<ServerNetMessage> {
		self.rx.recv().await.context("failed to recv message")
	}
}

pub struct ServerSide {
	tx: channels::Sender<
		ServerNetMessage,
		channels::io::Tokio<OwnedWriteHalf>,
		Serdes,
	>,
	rx: channels::Receiver<ClientNetMessage, OwnedReadHalf, Serdes>,
}

impl ServerSide {
	pub fn from_tcp_stream(stream: TcpStream) -> Self {
		let (r, w) = stream.into_split();

		Self {
			tx: channels::Sender::builder()
				.writer(w)
				.serializer(Default::default())
				.build(),
			rx: channels::Receiver::builder()
				.reader(r)
				.deserializer(Default::default())
				.build(),
		}
	}

	pub async fn send(
		&mut self,
		message: ServerNetMessage,
	) -> anyhow::Result<()> {
		self.tx
			.send(message)
			.await
			.context("failed to send message")
	}

	pub async fn recv(&mut self) -> anyhow::Result<ClientNetMessage> {
		self.rx.recv().await.context("failed to recv message")
	}
}

#[derive(Debug, Clone)]
pub enum Notification {
	UserConnected { name: String, addr: SocketAddr },
	ChatMessage { owner: String, content: String },
}

#[derive(Debug)]
pub struct ServerBus {
	tx: broadcast::Sender<Notification>,
	rx: broadcast::Receiver<Notification>,
}

impl ServerBus {
	pub fn new(size: usize) -> Self {
		let (tx, rx) = broadcast::channel(size);
		Self { tx, rx }
	}

	pub fn splice(&self) -> ServerBus {
		Self { tx: self.tx.clone(), rx: self.rx.resubscribe() }
	}

	pub fn send(&mut self, notification: Notification) {
		let _ = self.tx.send(notification);
	}

	pub async fn recv(&mut self) -> Notification {
		self.rx
			.recv()
			.await
			.expect("server communication channel closed")
	}
}

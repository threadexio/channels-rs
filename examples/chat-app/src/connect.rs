use std::net::SocketAddr;

use anyhow::{Context, Result};
use log::info;
use tokio::io::AsyncBufReadExt;
use tokio::net::TcpStream;

use super::Args;
use crate::common::*;
use crate::misc::*;

#[derive(Debug)]
struct State {
	net: ClientSide,
	name: String,
}

#[derive(Debug, clap::Args)]
pub struct ConnectArgs {
	#[arg(help = "Username")]
	name: String,

	#[arg(
		help = "Server to connect to",
		default_value = "127.0.0.1:13942"
	)]
	addr: SocketAddr,
}

pub async fn connect(
	_global_args: &Args,
	args: &ConnectArgs,
) -> Result<()> {
	let stream = TcpStream::connect(args.addr)
		.await
		.context("failed to connect to server")?;

	let mut state = State {
		name: args.name.clone(),
		net: ClientSide::from_tcp_stream(stream),
	};

	do_handshake(&mut state)
		.await
		.context("failed to handshake with server")?;
	info!("connected to {}", Address(args.addr));

	let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());
	let mut line = String::new();
	loop {
		tokio::select! {
			r = async {
				line.clear();
				stdin.read_line(&mut line).await.context("failed to read line from stdin")
			} => {
				let _ = r?;
				let line = line.trim();

				if !line.is_empty() {
					state
						.net
						.send(ClientNetMessage::SendMessage { content: line.to_owned() })
						.await.context("failed to send message")?;
				}
			}
			r = state.net.recv() => {
				let message = r.context("failed to receive message")?;

				match message {
					ServerNetMessage::MessagePosted { owner, content } => info!("{}: {}", Username(owner), Message(content)),
					ServerNetMessage::UserConnected { name } => info!("user '{}' connected", Username(name)),
				}
			}
		};
	}
}

async fn do_handshake(client: &mut State) -> Result<()> {
	client
		.net
		.send(ClientNetMessage::Authenticate {
			name: client.name.clone(),
		})
		.await
		.context("failed to send auth message")?;

	Ok(())
}

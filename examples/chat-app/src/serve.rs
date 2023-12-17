use std::net::SocketAddr;

use anyhow::{bail, Context, Result};
use log::{error, info};
use tokio::net::TcpListener;

use super::Args;
use crate::common::*;
use crate::misc::*;

#[derive(Debug, clap::Args)]
pub struct ServeArgs {
	#[arg(
		help = "Address to listen on",
		default_value = "127.0.0.1:13942"
	)]
	addr: SocketAddr,
}

struct Unauthenticated;

struct Authenticated {
	name: String,
}

struct State<State> {
	net: ServerSide,
	srv: ServerBus,
	addr: SocketAddr,
	state: State,
}

async fn do_handshake(
	mut client: State<Unauthenticated>,
) -> Result<State<Authenticated>> {
	let name = match client.net.recv().await? {
		ClientNetMessage::Authenticate { name } => name,
		_ => bail!("expected authenticate message"),
	};

	Ok(State {
		addr: client.addr,
		net: client.net,
		srv: client.srv,
		state: Authenticated { name },
	})
}

async fn handle_client(state: State<Unauthenticated>) -> Result<()> {
	let mut client = {
		let addr = state.addr;
		do_handshake(state).await.with_context(|| {
			format!("failed to handshake with client {}", addr)
		})?
	};

	client.srv.send(Notification::UserConnected {
		name: client.state.name.clone(),
		addr: client.addr,
	});

	loop {
		tokio::select! {
			r = client.net.recv() => {
				match r? {
					ClientNetMessage::SendMessage { content } => {
						let content = content.trim();
						if !content.is_empty() {
							client.srv.send(Notification::ChatMessage {
								owner: client.state.name.clone(),
								content: content.to_owned(),
							});
						}
					}
					_ => bail!("unexpected message received")
				}
			}
			message = client.srv.recv() => {
				match message {
					Notification::UserConnected { name, .. } => {
						client.net.send(ServerNetMessage::UserConnected { name }).await?;
					},
					Notification::ChatMessage { owner, content } => {
						client.net.send(ServerNetMessage::MessagePosted { owner, content }).await?;
					},
				}
			}
		}
	}
}

async fn server_task(
	_: &Args,
	_: &ServeArgs,
	mut bus: ServerBus,
) -> Result<()> {
	loop {
		match bus.recv().await {
			Notification::ChatMessage { owner, content } => {
				info!("{}: {}", Username(owner), Message(content))
			},
			Notification::UserConnected { name, addr } => {
				info!(
					"user '{}' connected from {}",
					Username(name),
					Address(addr)
				)
			},
		}
	}
}

async fn listen_task(
	_: &Args,
	args: &ServeArgs,
	srv_bus: ServerBus,
) -> Result<()> {
	let listener =
		TcpListener::bind(&args.addr).await.with_context(|| {
			format!(
				"failed to bind listener to {}",
				Address(args.addr)
			)
		})?;
	info!("listening on {}...", Address(args.addr));

	loop {
		let (stream, addr) = match listener.accept().await {
			Ok(v) => v,
			Err(e) => {
				error!("failed to accept client: {e}");
				continue;
			},
		};

		let client = State {
			net: ServerSide::from_tcp_stream(stream),
			srv: srv_bus.splice(),
			addr,
			state: Unauthenticated,
		};

		tokio::spawn(async { handle_client(client).await });
	}
}

pub async fn serve(
	global_args: &Args,
	args: &ServeArgs,
) -> Result<()> {
	let bus = ServerBus::new(16);

	let _ = tokio::join!(
		{
			let bus = bus.splice();
			async { server_task(global_args, args, bus).await }
		},
		{
			let bus = bus.splice();
			async move { listen_task(global_args, args, bus).await }
		},
	);

	Ok(())
}

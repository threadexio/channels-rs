use std::io::IsTerminal;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use log::{debug, error};

mod common;
mod connect;
mod misc;
mod serve;

use self::connect::{connect, ConnectArgs};
use self::serve::{serve, ServeArgs};

#[tokio::main]
async fn main() {
	if let Err(e) = try_main().await {
		error!("{e:}");
	}
}

#[derive(Debug, Parser)]
pub struct Args {
	#[arg(
		long = "level",
		help = "Console log level",
		default_value = "info"
	)]
	log_level: log::LevelFilter,

	#[arg(
		long = "color",
		help = "Specify whether to show colors",
		default_value = "auto"
	)]
	color: Color,

	#[command(subcommand)]
	command: Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Color {
	Always,
	Never,
	Auto,
}

#[derive(Debug, Subcommand)]
pub enum Command {
	Serve(ServeArgs),
	Connect(ConnectArgs),
}

async fn try_main() -> Result<()> {
	let global_args = Args::parse();

	setup_console(&global_args).unwrap();
	debug!("args: {global_args:#?}");

	match global_args.command {
		Command::Serve(ref args) => serve(&global_args, args).await,
		Command::Connect(ref args) => {
			connect(&global_args, args).await
		},
	}
}

fn setup_console(args: &Args) -> Result<()> {
	use log::Level;

	#[rustfmt::skip]
	fn level_to_str(level: Level) -> colored::ColoredString {
		match level {
			Level::Info =>  " INF ".black().on_bright_green(),
			Level::Warn =>  " WRN ".black().on_bright_yellow(),
			Level::Error => " ERR ".black().on_bright_red(),
			Level::Debug => " DBG ".black().on_bright_white(),
			Level::Trace => " TRC ".black().on_white(),
		}
	}

	let output = std::io::stderr();

	let should_use_colors = args.color == Color::Always
		|| (output.is_terminal() && args.color == Color::Auto);

	colored::control::set_override(should_use_colors);
	fern::Dispatch::new()
		.level(args.log_level)
		.format(|out, message, record| {
			out.finish(format_args!(
				"\r[{:<8} {}] {}",
				chrono::Local::now()
					.format("%H:%M:%S")
					.to_string()
					.cyan()
					.bold(),
				level_to_str(record.level()),
				message
			))
		})
		.chain(output)
		.apply()
		.context("failed to install logger")
}

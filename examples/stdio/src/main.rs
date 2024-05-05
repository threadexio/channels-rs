use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

const DEFAULT_PAYLOAD_SIZE: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Unspecified;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Mode {
	Sender,
	Receiver,
}

impl FromStr for Mode {
	type Err = Unspecified;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"sender" => Ok(Mode::Sender),
			"receiver" => Ok(Mode::Receiver),
			_ => Err(Unspecified),
		}
	}
}

#[derive(Debug)]
struct Args {
	mode: Mode,
	path: PathBuf,
	payload_size: usize,
}

impl Args {
	fn parse() -> Self {
		let mut args = env::args();

		let _ = args.next().unwrap(); // arg0

		let mode =
			args.next().unwrap().parse().expect("unknown mode");

		let path = PathBuf::from(args.next().unwrap_or("-".into()));

		let payload_size = args
			.next()
			.map(|x| x.parse::<usize>().unwrap())
			.unwrap_or(DEFAULT_PAYLOAD_SIZE);

		Self { mode, path, payload_size }
	}
}

fn main() {
	let args = Args::parse();

	match args.mode {
		Mode::Sender => Sender::main(args),
		Mode::Receiver => Receiver::main(args),
	}
}

enum Writer {
	Stdout(io::StdoutLock<'static>),
	File(fs::File),
}

impl Write for Writer {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		match self {
			Self::File(x) => x.write(buf),
			Self::Stdout(x) => x.write(buf),
		}
	}

	fn flush(&mut self) -> io::Result<()> {
		match self {
			Self::File(x) => x.flush(),
			Self::Stdout(x) => x.flush(),
		}
	}
}

struct Sender {
	writer: Writer,
	payload: Vec<u8>,
}

impl Sender {
	fn main(args: Args) {
		let mut this = Self::from(args);
		this._main();
	}

	fn _main(&mut self) {
		let mut tx =
			channels::Sender::<Vec<u8>, _, _>::new(&mut self.writer);

		loop {
			tx.send_blocking(&self.payload).unwrap();
			sleep(Duration::from_secs(1))
		}
	}
}

impl From<Args> for Sender {
	fn from(args: Args) -> Self {
		let writer = if &args.path.to_string_lossy() == "-" {
			Writer::Stdout(io::stdout().lock())
		} else {
			let file = fs::File::options()
				.write(true)
				.create(true)
				.truncate(true)
				.open(&args.path)
				.unwrap();
			Writer::File(file)
		};

		let payload =
			(0..args.payload_size).map(|x| x as u8).collect();

		Self { writer, payload }
	}
}

enum Reader {
	Stdin(io::StdinLock<'static>),
	File(fs::File),
}

impl Read for Reader {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		match self {
			Self::File(x) => x.read(buf),
			Self::Stdin(x) => x.read(buf),
		}
	}
}

struct Receiver {
	reader: Reader,
}

impl Receiver {
	fn main(args: Args) {
		let mut this = Self::from(args);
		this._main();
	}

	fn _main(&mut self) {
		let mut rx = channels::Receiver::<Vec<u8>, _, _>::new(
			&mut self.reader,
		);

		loop {
			let _ = rx.recv_blocking().unwrap();
		}
	}
}

impl From<Args> for Receiver {
	fn from(args: Args) -> Self {
		let reader = if &args.path.to_string_lossy() == "-" {
			Reader::Stdin(io::stdin().lock())
		} else {
			let file = fs::File::options()
				.read(true)
				.open(&args.path)
				.unwrap();
			Reader::File(file)
		};

		Self { reader }
	}
}

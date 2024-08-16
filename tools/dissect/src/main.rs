#![allow(missing_docs, rustdoc::missing_crate_level_docs)]

use core::fmt;

use std::io;

use anyhow::{Context, Result};

#[macro_use]
extern crate log;

mod display;
use self::display::{DisplayError, DisplayOk, Tree};

fn main() {
	display::setup_log(io::stderr());
	if let Err(e) = try_main() {
		error!("{e:#}");
	}
}

fn try_main() -> Result<()> {
	// TODO: redo
	Ok(())
}

/*fn try_main() -> Result<()> {
	let mut seq = IdSequence::new();

	let mut input = io::stdin().lock();
	let mut output = io::stdout().lock();

	loop {
		let packet = Packet::read(&mut input)
			.context("failed to read packet from stdin")?;

		let report = Report::new(&seq, packet.header);
		eprintln!("{report}");
		seq.advance();

		packet
			.write(&mut output)
			.context("failed to write packet to stdout")?;
	}
}

struct Packet {
	header: RawHeader,
	payload: Vec<u8>,
}

impl Packet {
	pub fn read(r: &mut dyn io::Read) -> Result<Self> {
		let mut header = RawHeader { bytes: [0u8; 8] };
		unsafe { r.read_exact(&mut header.bytes)? };
		let raw_header = unsafe { header.header };

		let payload_length =
			PacketLength::new(raw_header.length.into())
				.context("failed to decode packet length")?
				.to_payload_length()
				.as_usize();

		let mut payload = vec![0u8; payload_length];
		r.read_exact(&mut payload)?;

		Ok(Self { header, payload })
	}

	pub fn write(&self, w: &mut dyn io::Write) -> Result<()> {
		let header_bytes = unsafe { self.header.bytes };
		w.write_all(&header_bytes)
			.context("failed to write header")?;
		w.write_all(&self.payload)
			.context("failed to write payload")?;
		w.flush().context("failed to flush packet")?;

		Ok(())
	}
}

struct Report<'a> {
	seq: &'a IdSequence,
	header: RawHeader,
}

impl<'a> Report<'a> {
	pub fn new(seq: &'a IdSequence, header: RawHeader) -> Self {
		Self { seq, header }
	}
}

impl fmt::Display for Report<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let header = unsafe { self.header.header };

		let flags = Flags::from_bits_retain(header.flags);

		let mut report = Tree::new(f, "Packet");

		report
			.field("version", |f| {
				const EXPECTED: u16 =
					channels_packet::PROTOCOL_VERSION;

				let version: u16 = header.version.into();

				if version == EXPECTED {
					write!(f, "{}", DisplayOk(hex16(version)))
				} else {
					write!(
						f,
						"{} - expected: {}",
						DisplayError(hex16(version)),
						DisplayOk(hex16(EXPECTED))
					)
				}
			})
			.field("length", |f| {
				let length: u16 = header.length.into();

				match PacketLength::new(length) {
					Some(len) => {
						write!(f, "{}", DisplayOk(len.as_usize()))
					},
					None => write!(
						f,
						"{} - invalid value",
						DisplayError(length)
					),
				}
			})
			.field("checksum", |f| {
				let mut raw = self.header;

				if checksum::verify(unsafe { &raw.bytes }) {
					write!(
						f,
						"{} - {}",
						DisplayOk(hex16(unsafe {
							raw.header.checksum.into()
						})),
						DisplayOk("ok")
					)
				} else {
					let unverified =
						unsafe { raw.header.checksum.into() };

					let calculated = unsafe {
						raw.header.checksum = 0.into();
						checksum::checksum(&raw.bytes)
					};

					write!(
						f,
						"{} - expected: {}",
						DisplayError(hex16(unverified)),
						DisplayOk(hex16(calculated))
					)
				}
			});

		report
			.subtree("flags")
			.field(
				"MORE_DATA",
				format_flag_field(flags, Flags::MORE_DATA),
			)
			.field(
				"Reserved",
				format_flag_field(
					flags,
					Flags::from_bits_retain(255) ^ Flags::all(),
				),
			)
			.finish()?;

		report
			.field("id", |f| {
				let id = header.id;
				let expected = self.seq.peek().as_u8();

				if expected == id {
					write!(f, "{}", DisplayOk(id))
				} else {
					write!(
						f,
						"{} - expected: {}",
						DisplayError(id),
						DisplayOk(expected)
					)
				}
			})
			.finish()?;

		Ok(())
	}
}

fn hex16(x: u16) -> String {
	format!("0x{x:>04x}")
}

fn format_flag_binary(flags: Flags, specific: Flags) -> String {
	let mut out = String::with_capacity(9);

	let flags = flags.bits();
	let specific = specific.bits();

	for i in (0..8).rev() {
		if i == 3 {
			out.push(' ');
		}

		let mask = 1 << i;
		let flag_bit = flags & mask;
		let specific_bit = specific & mask;

		if specific_bit == 0 {
			out.push('.');
		} else if flag_bit == 0 {
			out.push('0');
		} else {
			out.push('1');
		}
	}

	out
}

fn format_flag_field(
	flags: Flags,
	specific: Flags,
) -> impl FnOnce(&mut fmt::Formatter) -> fmt::Result {
	move |f| {
		write!(f, "{}: ", format_flag_binary(flags, specific))?;

		if flags.contains(specific) {
			write!(f, "Set")
		} else {
			write!(f, "Not set")
		}
	}
}
*/

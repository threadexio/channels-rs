//! Utilities to parse channels packets.
#![no_std]
#![allow(
	unknown_lints,
	clippy::new_without_default,
	clippy::needless_doctest_main
)]
#![warn(
	clippy::all,
	clippy::style,
	clippy::perf,
	clippy::correctness,
	clippy::complexity,
	clippy::deprecated,
	clippy::missing_doc_code_examples,
	clippy::missing_panics_doc,
	clippy::missing_safety_doc,
	clippy::missing_doc_code_examples,
	clippy::cast_lossless,
	clippy::cast_possible_wrap,
	clippy::useless_conversion,
	clippy::wrong_self_convention,
	rustdoc::all,
	rustdoc::broken_intra_doc_links
)]
#![deny(missing_docs)]

mod header;
mod util;

pub use self::header::{
	Checksum, Flags, Header, HeaderReadError, Id, IdGenerator,
	PacketLength, PayloadLength,
};
pub use self::util::{slice_to_array, slice_to_array_mut};

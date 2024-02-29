//! Utilities to parse channels packets.
#![no_std]
#![deny(missing_docs)]
#![warn(
	clippy::all,
	clippy::style,
	clippy::cargo,
	clippy::perf,
	clippy::correctness,
	clippy::complexity,
	clippy::pedantic,
	clippy::suspicious,
	arithmetic_overflow,
	missing_debug_implementations,
	clippy::cast_lossless,
	clippy::cast_possible_wrap,
	clippy::useless_conversion,
	clippy::wrong_self_convention,
	clippy::missing_assert_message,
	clippy::unwrap_used,
	// Docs
	rustdoc::all,
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	clippy::missing_panics_doc,
	clippy::missing_safety_doc,
)]
#![allow(
	clippy::new_without_default,
	clippy::module_name_repetitions,
	clippy::missing_errors_doc
)]
#![cfg_attr(channels_nightly, feature(doc_auto_cfg))]

mod header;
mod util;

pub use self::header::{
	Checksum, Flags, Header, HeaderReadError, Id, IdGenerator,
	PacketLength, PayloadLength,
};
pub use self::util::{slice_to_array, slice_to_array_mut};

//! Utilities to parse channels packets.
#![cfg_attr(channels_nightly, feature(doc_auto_cfg))]
#![no_std]

mod header;
mod util;

pub use self::header::{
	Checksum, Flags, Header, HeaderReadError, Id, IdGenerator,
	PacketLength, PayloadLength,
};
pub use self::util::{slice_to_array, slice_to_array_mut};

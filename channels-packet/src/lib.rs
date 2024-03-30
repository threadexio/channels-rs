//! Utilities to parse channels packets.
#![cfg_attr(channels_nightly, feature(doc_auto_cfg))]
#![no_std]

mod consts;
mod flags;
mod num;
mod util;

pub mod checksum;
pub mod header;
pub mod id;
pub mod raw;

pub use self::flags::Flags;
pub use self::num::{PacketLength, PayloadLength};
pub use self::util::{slice_to_array, slice_to_array_mut};

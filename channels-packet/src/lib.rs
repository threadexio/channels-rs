//! TODO: docs
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod frame;
pub mod header;
pub mod num;

#[cfg(feature = "framed")]
pub mod codec;

pub use self::frame::Frame;
pub use self::header::{FrameNumSequence, Header};

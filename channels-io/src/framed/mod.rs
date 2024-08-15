//! TODO: docs

mod decoder;
mod framed_read;

pub use self::decoder::Decoder;
pub use self::framed_read::{FramedRead, FramedReadError};

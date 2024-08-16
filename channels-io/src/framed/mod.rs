//! TODO: docs

mod decoder;
mod encoder;
mod framed_read;
mod framed_write;

pub use self::decoder::Decoder;
pub use self::encoder::Encoder;
pub use self::framed_read::{FramedRead, FramedReadError};
pub use self::framed_write::{FramedWrite, FramedWriteError};

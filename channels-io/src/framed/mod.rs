//! Abstractions for turning unstructure I/O streams like [`Read`] and [`Write`] to structured
//! types streams like [`Source`] and [`Sink`].
//!
//! [`Read`]: crate::Read
//! [`Write`]: crate::Write
//! [`Source`]: crate::source::Source
//! [`Sink`]: crate::sink::Sink

mod decoder;
mod encoder;
mod framed_read;
mod framed_write;

pub use self::decoder::Decoder;
pub use self::encoder::Encoder;
pub use self::framed_read::{FramedRead, FramedReadError};
pub use self::framed_write::{FramedWrite, FramedWriteError};

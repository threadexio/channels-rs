//! [`Buf`], [`BufMut`] and other utilities to work with buffers.

#[allow(clippy::module_inception)]
mod buf;
mod buf_mut;
mod cursor;

pub use self::buf::{Buf, Reader, ReaderError};
pub use self::buf_mut::{BufMut, Writer, WriterError};

pub use self::cursor::Cursor;

//! [`Buf`], [`BufMut`] and other utilities to work with buffers.

#[allow(clippy::module_inception)]
mod buf;
mod buf_mut;
mod chain;
mod cursor;
mod limit;
mod take;

pub use self::buf::{Buf, Reader, ReaderError};
pub use self::buf_mut::{BufMut, Writer, WriterError};
pub use self::chain::Chain;
pub use self::cursor::Cursor;
pub use self::limit::Limit;
pub use self::take::Take;

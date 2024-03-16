mod read;
mod write;

mod impls;

macro_rules! future {
	($($typ:tt)+) => {
		impl ::core::future::Future<Output = $($typ)+>
	}
}
use future;

pub use self::read::{AsyncRead, IntoReader, Read, Reader};
pub use self::write::{AsyncWrite, IntoWriter, Write, Writer};

pub use self::impls::*;

mod async_read;
mod async_write;

mod impls;

pub use self::async_read::{AsyncRead, IntoAsyncReader};
pub use self::async_write::{AsyncWrite, IntoAsyncWriter};

macro_rules! decouple {
	($e:expr; as mut) => {
		unsafe { ::core::ptr::addr_of_mut!($e).as_mut().unwrap() }
	};
	($e:expr; as const) => {
		unsafe { ::core::ptr::addr_of!($e).as_ref().unwrap() }
	};
}
use decouple; // needed to set visibility as pub(self)

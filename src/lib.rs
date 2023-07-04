#![doc = include_str!("../README.md")]
#![allow(unknown_lints, dead_code)]
#![warn(
	clippy::all,
	clippy::style,
	clippy::cargo,
	clippy::perf,
	clippy::correctness,
	clippy::complexity,
	clippy::deprecated,
	clippy::missing_doc_code_examples,
	clippy::missing_panics_doc,
	clippy::missing_safety_doc,
	clippy::missing_doc_code_examples,
	clippy::cast_lossless,
	clippy::cast_possible_wrap,
	clippy::as_conversions,
	clippy::useless_conversion,
	clippy::wrong_self_convention,
	rustdoc::all
)]
#![deny(missing_docs)]

mod io;
mod mem;
mod packet;

#[cfg(feature = "serde")]
mod serde;

#[cfg(feature = "statistics")]
/// Structures that hold statistic information about channels.
///
/// See: [`statistics`](crate#features) feature.
pub mod stats;

/// Error module.
pub mod error;
pub use error::{Error, Result};

mod sender;
pub use sender::Sender;

mod receiver;
pub use receiver::Receiver;

/// A tuple containing a [`Sender`] and a [`Receiver`].
pub type Pair<T, Reader, Writer> = Pair2<T, T, Reader, Writer>;

/// A tuple containing a [`Sender`] and a [`Receiver`] that use different types.
pub type Pair2<S, R, Reader, Writer> =
	(Sender<S, Writer>, Receiver<R, Reader>);

/// Creates a new channel.
///
/// > **NOTE:** If you need a [`Sender`] and a [`Receiver`] that use different types use [`channel2`].
///
/// # Usage
/// ```no_run
/// use std::net::TcpStream;
///
/// let conn = TcpStream::connect("0.0.0.0:1234").unwrap();
///
/// let (mut tx, mut rx) = channels::channel(conn.try_clone().unwrap(), conn);
///
/// tx.try_send(42_i32).unwrap();
/// let received: i32 = rx.try_recv().unwrap();
/// ```
pub fn channel<T, Reader, Writer>(
	r: Reader,
	w: Writer,
) -> Pair<T, Reader, Writer> {
	channel2::<T, T, Reader, Writer>(r, w)
}

/// Creates a new channel that sends and receives different types.
///
/// # Usage
/// ```no_run
/// use std::net::TcpStream;
///
/// let conn = TcpStream::connect("0.0.0.0:1234").unwrap();
///
/// let (mut tx, mut rx) = channels::channel2(conn.try_clone().unwrap(), conn);
///
/// tx.try_send(42_i32).unwrap();
/// let received: i64 = rx.try_recv().unwrap();
/// ```
pub fn channel2<S, R, Reader, Writer>(
	r: Reader,
	w: Writer,
) -> Pair2<S, R, Reader, Writer> {
	(Sender::new(w), Receiver::new(r))
}

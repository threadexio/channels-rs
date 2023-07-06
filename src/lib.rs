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

/// Serialization/Deserialization traits and types.
pub mod serdes;

#[cfg(feature = "statistics")]
/// Structures that hold statistic information about channels.
///
/// See: [`statistics`](crate#features) feature.
pub mod stats;

/// Error module.
pub mod error;

mod sender;
pub use sender::Sender;

mod receiver;
pub use receiver::Receiver;

use io::{Read, Write};

/// A tuple containing a [`Sender`] and a [`Receiver`].
pub type Pair<T, R, W, S, D> = (Sender<T, W, S>, Receiver<T, R, D>);

#[cfg(feature = "serde")]
/// Create a new channel.
///
/// **NOTE:** If you need a [`Sender`] and a [`Receiver`] that use
/// different types, the `new` or the `with_serializer` and `with_deserializer` methods on
/// [`Sender`] and [`Receiver`].
///
/// # Usage
/// ```no_run
/// use std::net::TcpStream;
///
/// let conn = TcpStream::connect("0.0.0.0:1234").unwrap();
///
/// let (mut tx, mut rx) = channels::channel(conn.try_clone().unwrap(), conn);
///
/// tx.send(42_i32).unwrap();
/// let received: i32 = rx.recv().unwrap();
/// ```
pub fn channel<T, R, W>(
	r: R,
	w: W,
) -> Pair<T, R, W, serdes::Bincode, serdes::Bincode>
where
	T: serde::Serialize,
	T: for<'de> serde::Deserialize<'de>,
	R: Read,
	W: Write,
{
	(Sender::new(w), Receiver::new(r))
}

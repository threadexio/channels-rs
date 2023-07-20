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
	rustdoc::all,
	rustdoc::broken_intra_doc_links
)]
#![deny(missing_docs)]

mod io;
mod mem;
mod packet;
mod util;

/// Adapter types for [`std::io::Read`]/[`std::io::Write`] streams.
pub mod adapter;

/// Serialization/Deserialization traits and types.
pub mod serdes;

#[cfg(feature = "statistics")]
/// Structures that hold statistic information about channels.
///
/// See: [`statistics`] feature.
///
/// [`statistics`]: crate#features
pub mod stats;

/// Error module.
pub mod error;

/// Module containing the implementation for [`Sender`].
pub mod sender;
pub use sender::Sender;

/// Module containing the implementation for [`Receiver`].
pub mod receiver;
pub use receiver::Receiver;

use io::{Read, Write};

/// A tuple containing a [`Sender`] and a [`Receiver`].
pub type Pair<T, R, W, S, D> = (Sender<T, W, S>, Receiver<T, R, D>);

#[cfg(feature = "serde")]
/// Create a new channel.
///
/// If your reader and writer are one type that does not support splitting
/// its 2 halves, use the `split` function from [`adapter::unsync`]
/// or [`adapter::sync`].
///
/// **NOTE:** If you need a [`Sender`] and a [`Receiver`] that use
/// different types, the `new` or the [`Sender::with_serializer`] and
/// [`Receiver::with_deserializer`] methods.
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

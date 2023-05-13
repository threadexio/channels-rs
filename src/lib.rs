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
	clippy::missing_const_for_fn,
	clippy::missing_panics_doc,
	clippy::missing_safety_doc,
	clippy::missing_doc_code_examples,
	clippy::cast_lossless,
	clippy::cast_possible_wrap,
	rustdoc::all
)]
#![deny(missing_docs)]

mod crc;
mod io;
mod packet;
mod util;

#[cfg(feature = "serde")]
mod serde;

#[cfg(feature = "statistics")]
/// Structures that hold statistic information about channels.
///
/// See: [`statistics`](crate#features) feature.
pub mod stats;

mod error;
pub use error::{Error, Result};

mod sender;
pub use sender::Sender;

mod receiver;
pub use receiver::Receiver;

use std::io::{Read, Write};

/// A tuple containing a [`Sender`] and a [`Receiver`].
pub type Pair<'r, 'w, T> = (Sender<'w, T>, Receiver<'r, T>);

/// A tuple containing a [`Sender`] and a [`Receiver`] that use different types.
pub type Pair2<'r, 'w, S, R> = (Sender<'w, S>, Receiver<'r, R>);

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
/// let (mut tx, mut rx) = channels::channel::<i32>(conn.try_clone().unwrap(), conn);
/// ```
pub fn channel<'r, 'w, T>(
	r: impl Read + 'r,
	w: impl Write + 'w,
) -> Pair<'r, 'w, T> {
	channel2::<T, T>(r, w)
}

/// Creates a new channel that sends and receives different types.
///
/// # Usage
/// ```no_run
/// use std::net::TcpStream;
///
/// let conn = TcpStream::connect("0.0.0.0:1234").unwrap();
///
/// let (mut tx, mut rx) = channels::channel2::<i32, i64>(conn.try_clone().unwrap(), conn);
/// ```
pub fn channel2<'r, 'w, S, R>(
	r: impl Read + 'r,
	w: impl Write + 'w,
) -> Pair2<'r, 'w, S, R> {
	(Sender::new(w), Receiver::new(r))
}

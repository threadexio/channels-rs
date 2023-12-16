#![doc = include_str!("../README.md")]
#![allow(
	unknown_lints,
	clippy::new_without_default,
	clippy::needless_doctest_main
)]
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
	clippy::useless_conversion,
	clippy::wrong_self_convention,
	rustdoc::all,
	rustdoc::broken_intra_doc_links
)]
#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

// TODO: Packet middleware

mod common;

pub mod error;

pub mod receiver;
pub mod sender;

#[cfg(feature = "statistics")]
pub use self::common::Statistics;

pub use self::receiver::Receiver;
pub use self::sender::Sender;

pub use channels_serdes as serdes;

/// A tuple containing a [`Sender`] and a [`Receiver`].
pub type Pair<T, R, W, S, D> = (Sender<T, W, S>, Receiver<T, R, D>);

#[cfg(feature = "bincode")]
/// Create a new synchronous channel.
///
/// # Example
/// ```no_run
/// use std::net::TcpStream;
///
/// let conn = TcpStream::connect("127.0.0.1:1234").unwrap();
///
/// let (mut tx, mut rx) = channels::channel(conn.try_clone().unwrap(), conn);
///
/// tx.send_blocking(42_i32).unwrap();
/// let received: i32 = rx.recv_blocking().unwrap();
/// ```
pub fn channel<T, R, W>(
	r: R,
	w: W,
) -> Pair<T, R, W, channels_serdes::Bincode, channels_serdes::Bincode>
where
	for<'de> T: serde::Serialize + serde::Deserialize<'de>,
{
	use channels_serdes::Bincode;

	(
		Sender::builder()
			.writer(w)
			.serializer(Bincode::new())
			.build(),
		Receiver::builder()
			.reader(r)
			.deserializer(Bincode::new())
			.build(),
	)
}

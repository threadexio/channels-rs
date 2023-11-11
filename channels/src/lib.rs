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

pub use channels_io as io;
pub use channels_serdes as serdes;

use channels_io::prelude::*;

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
	r: impl IntoReader<R>,
	w: impl IntoWriter<W>,
) -> Pair<T, R, W, channels_serdes::Bincode, channels_serdes::Bincode>
where
	for<'de> T: serde::Serialize + serde::Deserialize<'de>,
	R: Read,
	W: Write,
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

#[cfg(feature = "bincode")]
/// Create a new asynchronous channel.
///
/// # Example
/// ```no_run
/// use tokio::net::TcpStream;
///
/// #[tokio::main]
/// async fn main() {
///     let conn = TcpStream::connect("127.0.0.1:1234").await.unwrap();
///     let (r, w) = conn.into_split();
///     let (mut tx, mut rx) = channels::channel_async(r, w);
///
///     tx.send(42_i32).await.unwrap();
///     let received: i32 = rx.recv().await.unwrap();
/// }
/// ```
pub fn channel_async<T, R, W>(
	r: impl IntoAsyncReader<R>,
	w: impl IntoAsyncWriter<W>,
) -> Pair<T, R, W, channels_serdes::Bincode, channels_serdes::Bincode>
where
	for<'de> T: serde::Serialize + serde::Deserialize<'de>,
	R: AsyncRead,
	W: AsyncWrite,
{
	use channels_serdes::Bincode;

	(
		Sender::builder()
			.async_writer(w)
			.serializer(Bincode::new())
			.build(),
		Receiver::builder()
			.async_reader(r)
			.deserializer(Bincode::new())
			.build(),
	)
}

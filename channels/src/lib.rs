//! <style>
//! .rustdoc-hidden { display: none; }
//! </style>
#![doc = include_str!("../README.md")]
#![cfg_attr(channels_nightly, feature(doc_auto_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::print_stdout, clippy::print_stderr)]

extern crate alloc;

mod protocol;
mod statistics;
mod util;

pub mod error;

pub mod receiver;
pub mod sender;

#[cfg(feature = "statistics")]
pub use self::statistics::Statistics;

pub use self::receiver::Receiver;
pub use self::sender::Sender;

#[doc(inline)]
pub use {channels_io as io, channels_serdes as serdes};

/// A tuple containing a [`Sender`] and a [`Receiver`].
pub type Pair<T, R, W, Sd> = (Sender<T, W, Sd>, Receiver<T, R, Sd>);

#[cfg(feature = "bincode")]
/// Create a new channel.
///
/// This function is just a shorthand for [`Sender::new`] and [`Receiver::new`].
///
/// # Examples
///
/// Synchronous version:
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
///
/// Asynchronous version:
/// ```no_run
/// use tokio::net::TcpStream;
///
/// # #[tokio::main]
/// # async fn main() {
/// let conn = TcpStream::connect("127.0.0.1:1234").await.unwrap();
/// let (r, w) = conn.into_split();
/// let (mut tx, mut rx) = channels::channel(r, w);
///
/// tx.send(42_i32).await.unwrap();
/// let received: i32 = rx.recv().await.unwrap();
/// # }
/// ```
#[inline]
pub fn channel<T, R, W>(
	r: impl io::IntoRead<R>,
	w: impl io::IntoWrite<W>,
) -> Pair<T, R, W, channels_serdes::Bincode>
where
	for<'de> T: serde::Serialize + serde::Deserialize<'de>,
{
	(Sender::new(w), Receiver::new(r))
}

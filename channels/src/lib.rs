//! Channel communication across generic data streams.
//!
//! ## Quick start
//!
//! ### Async
//!
//! ```no_run
//! use tokio::net::TcpStream;
//! use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
//!
//!
//! #[tokio::main]
//! async fn main() {
//!     let stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
//!     let (r, w) = stream.into_split();
//!     let (mut tx, mut rx) = channels::channel::<i32, _, _>(r, w);
//!
//!     let r = rx.recv().await.unwrap();
//!     println!("{r}");
//!     tx.send(r).await.unwrap();
//! }
//! ```
//!
//! ### Sync
//!
//! ```no_run
//! use std::net::TcpStream;
//!
//! let stream = TcpStream::connect("127.0.0.1:8080").unwrap();
//! let (mut tx, mut rx) = channels::channel::<i32, _, _>(stream.try_clone().unwrap(), stream);
//!
//! let r = rx.recv_blocking().unwrap();
//! println!("{r}");
//! tx.send_blocking(r).unwrap();
//! ```
//!
//! ## Examples
//!
//! See: [examples/](https://github.com/threadexio/channels-rs/tree/master/examples)
//!
//! ## cargo features
//!
//! |   Feature    | Description                                                                                     |
//! | :----------: | :---------------------------------------------------------------------------------------------- |
//! | `statistics` | Capture statistic data like: total bytes sent/received, number of send/receive operations, etc. |
//! | `tokio`      | Adds support for sending/receiving types asynchronously with tokio.                             |
//! | `futures`    | Adds support for sending/receiving types asynchronously with futures.                           |
//! | `bincode`    | Support for serializing/deserializing types with `bincode`.                                     |
//! | `cbor`       | Support for serializing/deserializing types with `ciborium`.                                    |
//! | `json`       | Support for serializing/deserializing types with `serde_json`.                                  |
//! | `full`       | All of the above except `tokio` and `futures`.                                                  |
//!
//! **NOTE:** Because of the subtle differences in the APIs of `tokio` and
//!           `futures`, it means that these 2 features must be exclusive. You
//!           cannot have both `tokio` and `futures` enabled at once.
#![deny(missing_docs)]
#![warn(
	clippy::all,
	clippy::style,
	clippy::cargo,
	clippy::perf,
	clippy::correctness,
	clippy::complexity,
	clippy::pedantic,
	clippy::suspicious,
	arithmetic_overflow,
	missing_debug_implementations,
	clippy::cast_lossless,
	clippy::cast_possible_wrap,
	clippy::useless_conversion,
	clippy::wrong_self_convention,
	clippy::missing_assert_message,
	clippy::unwrap_used,
	// Docs
	rustdoc::all,
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	clippy::missing_panics_doc,
	clippy::missing_safety_doc,
)]
#![allow(
	clippy::new_without_default,
	clippy::module_name_repetitions,
	clippy::missing_errors_doc
)]

extern crate alloc;

mod common;
mod util;

pub mod error;

pub mod receiver;
pub mod sender;

#[cfg(feature = "statistics")]
pub use self::common::Statistics;

pub use self::receiver::Receiver;
pub use self::sender::Sender;

pub use channels_serdes as serdes;

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
/// #[tokio::main]
/// async fn main() {
///     let conn = TcpStream::connect("127.0.0.1:1234").await.unwrap();
///     let (r, w) = conn.into_split();
///     let (mut tx, mut rx) = channels::channel(r, w);
///
///     tx.send(42_i32).await.unwrap();
///     let received: i32 = rx.recv().await.unwrap();
/// }
/// ```
pub fn channel<T, R, W>(
	r: R,
	w: W,
) -> Pair<T, R, W, channels_serdes::Bincode>
where
	for<'de> T: serde::Serialize + serde::Deserialize<'de>,
{
	(Sender::new(w), Receiver::new(r))
}

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
//! |    Feature    | Description                                                                                     |
//! | :-----------: | :---------------------------------------------------------------------------------------------- |
//! | `statistics`  | Capture statistic data like: total bytes sent/received, number of send/receive operations, etc. |
//! | `aead`        | Middleware that encrypts data for transport.                                                    |
//! | `bincode`     | Support for serializing/deserializing types with [`bincode`].                                   |
//! | `borsh`       | Support for serializing/deserializing types with [`borsh`].                                     |
//! | `cbor`        | Support for serializing/deserializing types with [`ciborium`].                                  |
//! | `crc`         | Middleware that verifies data with a CRC checksum.                                              |
//! | `deflate`     | Middleware that compresses data with DEFLATE.                                                   |
//! | `hmac`        | Middleware that verifies data with HMAC.                                                        |
//! | `json`        | Support for serializing/deserializing types with [`serde_json`].                                |
//! | `full-serdes` | Features: `aead`, `bincode`, `borsh`, `cbor`, `crc`, `deflate`, `hmac`, `json`                  |
//! | `core2`       | Adds support for sending/receiving types with [`core2`].                                        |
//! | `futures`     | Adds support for sending/receiving types asynchronously with [`futures`].                       |
//! | `std`         | Adds support for sending/receiving types over [`Read`] and [`Write`].                           |
//! | `tokio`       | Adds support for sending/receiving types asynchronously with [`tokio`].                         |
//! | `full-io`     | Features: `core2`, `futures`, `std`, `tokio`                                                    |
//! | `full`        | Every feature in this table.                                                                    |
//!
//! [`core2`]: https://docs.rs/core2
//! [`bincode`]: https://docs.rs/bincode
//! [`ciborium`]: https://docs.rs/ciborium
//! [`serde_json`]: https://docs.rs/serde_json
//! [`borsh`]: https://docs.rs/borsh
//! [`tokio`]: https://docs.rs/tokio
//! [`futures`]: https://docs.rs/futures
//! [`Read`]: std::io::Read
//! [`Write`]: std::io::Write
#![cfg_attr(channels_nightly, feature(doc_auto_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

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

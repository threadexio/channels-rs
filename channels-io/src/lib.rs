//! Abstractions on top of synchronous and asynchronous IO interfaces.
//!
//! This crate provides a generic interface to work with synchronous or
//! asynchronous IO provided by many other crates. Using this crate on top of,
//! say [`tokio`] or [`std`], allows you not be vendor-locked to each crate's
//! ecosystem. For example, code written with this crate can work with both
//! [`tokio`] and [`futures`] with no additional code and no hacky workarounds.
//!
//! ```rust,no_run
//! use channels_io::{IntoWrite, AsyncWrite, AsyncWriteExt, Futures};
//!
//! async fn write_data<W>(writer: impl IntoWrite<W>) -> Result<(), W::Error>
//! where
//!     W: AsyncWrite + Unpin
//! {
//!     let mut writer = writer.into_write();
//!
//!     let data: Vec<u8> = (0..255).collect();
//!
//!     writer.write_buf(&mut data.as_slice()).await
//! }
//!
//! async fn my_fn_tokio() {
//!     use tokio::fs::OpenOptions;
//!
//!     let mut file = OpenOptions::new()
//!         .write(true)
//!         .truncate(true)
//!         .create(true)
//!         .open("/tmp/some_file")
//!         .await
//!         .unwrap();
//!
//!     write_data(&mut file).await.unwrap();
//! }
//!
//! async fn my_fn_futures() {
//!     use async_std::fs::OpenOptions;
//!
//!     let mut file = OpenOptions::new()
//!         .write(true)
//!         .truncate(true)
//!         .create(true)
//!         .open("/tmp/some_file")
//!         .await
//!         .unwrap();
//!
//!     // If there is a compiler error here about multiple impls that satisfying
//!     // a bound, you might have to specify explicitly which implementation to
//!     // use with the turbofish syntax, like bellow:
//!     write_data::<Futures<_>>(&mut file).await.unwrap();
//! }
//! ```
//!
//! As you can see `write_data` is called both with types from [`tokio`] and
//! [`async-std`] (aka [`futures`]). The same logic applies to synchronous code.
//!
//! [`async-std`]: https://docs.rs/async-std
//! [`futures`]: https://docs.rs/futures
//! [`tokio`]: https://docs.rs/tokio
#![cfg_attr(channels_nightly, feature(doc_auto_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod convert;
mod util;

mod async_read;
mod async_write;
mod read;
mod write;

pub mod buf;
pub mod error;

#[cfg(feature = "alloc")]
pub mod transaction;

#[cfg(feature = "alloc")]
pub mod framed;

pub use self::buf::{Buf, BufMut, Cursor};
pub use self::convert::{Container, IntoRead, IntoWrite};

pub use self::async_read::{AsyncRead, AsyncReadExt};
pub use self::async_write::{AsyncWrite, AsyncWriteExt};
pub use self::read::{Read, ReadExt};
pub use self::write::{Write, WriteExt};

mod impls;

#[allow(unused_imports)]
pub use self::impls::*;

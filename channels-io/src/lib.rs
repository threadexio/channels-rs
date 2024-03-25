//! Abstractions on top of synchronous and asynchronous IO interfaces and buffers.
//!
//! This crate provides a generic interface to work with synchronous or
//! asynchronous IO provided by many other crates. Using this crate on top of,
//! say [`tokio`] or [`std`], allows you not be vendor-locked to each crate's
//! ecosystem. For example, code written with this crate can work with both
//! [`tokio`] and [`futures`] with no additional code and no hacky workarounds.
//!
//! ```rust,no_run
//! use channels_io::{IntoWriter, AsyncWrite};
//!
//! async fn write_data<W>(writer: impl IntoWriter<W>) -> Result<(), W::Error>
//! where
//!     W: AsyncWrite
//! {
//!     let mut writer = writer.into_writer();
//!
//!     let data: Vec<u8> = (0..255).collect();
//!
//!     writer.write(data.as_slice()).await
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
//!     write_data(&mut file).await.unwrap();
//! }
//! ```
//!
//! As you can see `write_data` is called both with types from [`tokio`] and
//! [`async-std`] (aka [`futures`]). The same logic applies to synchronous code.
#![cfg_attr(channels_nightly, feature(doc_auto_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod util;

mod bytes;

pub mod buf;
pub mod buf_mut;

pub use self::buf::{Buf, Contiguous, Walkable};
pub use self::buf_mut::{BufMut, ContiguousMut, WalkableMut};
pub use self::bytes::{AsBytes, AsBytesMut};

pub mod chain;
pub mod cursor;
pub mod limit;
pub mod take;

pub use self::chain::{chain, Chain};
pub use self::cursor::Cursor;
pub use self::limit::{limit, Limit};
pub use self::take::{take, Take};

mod read;
mod write;

pub use self::read::{AsyncRead, IntoReader, Read, Reader};
pub use self::write::{AsyncWrite, IntoWriter, Write, Writer};

mod impls;

#[allow(unused_imports)]
pub use self::impls::*;

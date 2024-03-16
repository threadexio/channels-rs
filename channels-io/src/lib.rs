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
#![cfg_attr(channels_nightly, feature(doc_auto_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod buf;
mod io;

pub use self::buf::*;
pub use self::io::*;

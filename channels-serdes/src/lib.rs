//! Utilities to serialize/deserialize types.
//!
//! # Implementing a serializer/deserializer
//!
//! ```rust
//! use std::convert::Infallible;
//! use channels_serdes::{Serializer, Deserializer};
//! use channels_io::{Walkable, Contiguous, Cursor, Buf};
//!
//! struct MyI32;
//!
//! #[derive(Debug, PartialEq, Eq)]
//! enum I32DeserializeError {
//!     NotEnough,
//! }
//!
//! impl Serializer<i32> for MyI32 {
//!     type Error = Infallible; // serializing an i32 cannot fail
//!
//!     fn serialize(&mut self, t: &i32) -> Result<impl Walkable, Self::Error> {
//!         let vec = t.to_be_bytes().to_vec();
//!         let buf = Cursor::new(vec);
//!         Ok(buf)
//!     }
//! }
//!
//! impl Deserializer<i32> for MyI32 {
//!     type Error = I32DeserializeError;
//!
//!     fn deserialize(&mut self, buf: impl Contiguous) -> Result<i32, Self::Error> {
//!         buf.chunk().get(..4)
//!            .map(|slice| -> [u8; 4] { slice.try_into().unwrap() })
//!            .map(i32::from_be_bytes)
//!            .ok_or(I32DeserializeError::NotEnough)
//!     }
//! }
//!
//! let mut sd = MyI32;
//!
//! let mut serialized = sd.serialize(&42)
//!                        .unwrap()
//!                        .copy_to_contiguous();
//!
//! assert_eq!(serialized.chunk(), [0, 0, 0, 42]);
//!
//! let deserialized = sd.deserialize(serialized);
//! assert_eq!(deserialized, Ok(42));
//! ```
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
#![cfg_attr(not(needs_std), no_std)]
#![cfg_attr(channels_nightly, feature(doc_auto_cfg))]

use channels_io::{Contiguous, Walkable};

/// The [`Serializer`] trait allows converting a type `T` to safe-to-transport
/// byte sequences.
///
/// Types implementing this trait are called 'serializers'.
pub trait Serializer<T> {
	/// The error returned by [`Serializer::serialize()`].
	type Error;

	/// Serialize `t` to a buffer.
	fn serialize(
		&mut self,
		t: &T,
	) -> Result<impl Walkable, Self::Error>;
}

/// The [`Deserializer`] trait allows converting a byte slice to a type `T`.
///
/// Types implementing this trait are called 'deserializers'.
pub trait Deserializer<T> {
	/// The error returned by [`Deserializer::deserialize()`].
	type Error;

	/// Deserialize bytes from `buf` to a type `T`.
	///
	/// `buf` is passed with a mutable reference so implementations can do
	/// in-place modification of the data if needed.
	fn deserialize(
		&mut self,
		buf: impl Contiguous,
	) -> Result<T, Self::Error>;
}

use cfg_if::cfg_if;

cfg_if! {
	if #[cfg(feature = "bincode")] {
		mod bincode;
		pub use self::bincode::Bincode;
	}
}

cfg_if! {
	if #[cfg(feature = "cbor")] {
		mod cbor;
		pub use self::cbor::Cbor;
	}
}

cfg_if! {
	if #[cfg(feature = "json")] {
		mod json;
		pub use self::json::Json;
	}
}

cfg_if! {
	if #[cfg(feature = "borsh")] {
		mod borsh;
		pub use self::borsh::Borsh;
	}
}

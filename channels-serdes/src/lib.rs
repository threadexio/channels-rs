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
#![cfg_attr(channels_nightly, feature(doc_auto_cfg))]
#![cfg_attr(not(needs_std), no_std)]

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

macro_rules! serde_impl {
	($module:ident :: $impl:ident if $feature:literal) => {
		#[cfg(feature = $feature)]
		mod $module;
		#[cfg(feature = $feature)]
		pub use self::$module::$impl;
	};
}

serde_impl! { bincode::Bincode if "bincode" }
serde_impl! { cbor::Cbor       if "cbor"    }
serde_impl! { json::Json       if "json"    }
serde_impl! { borsh::Borsh     if "borsh"   }

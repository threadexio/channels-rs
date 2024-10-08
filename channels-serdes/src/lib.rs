//! Utilities to serialize/deserialize types.
//!
//! # Implementing a serializer/deserializer
//!
//! ```rust
//! use std::convert::Infallible;
//! use channels_serdes::{Serializer, Deserializer};
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
//!     fn serialize(&mut self, t: &i32) -> Result<Vec<u8>, Self::Error> {
//!         let vec = t.to_be_bytes().to_vec();
//!         Ok(vec)
//!     }
//! }
//!
//! impl Deserializer<i32> for MyI32 {
//!     type Error = I32DeserializeError;
//!
//!     fn deserialize(&mut self, buf: &mut [u8]) -> Result<i32, Self::Error> {
//!         buf.get(..4)
//!            .map(|slice| -> [u8; 4] { slice.try_into().unwrap() })
//!            .map(i32::from_be_bytes)
//!            .ok_or(I32DeserializeError::NotEnough)
//!     }
//! }
//!
//! let mut sd = MyI32;
//!
//! let mut serialized: Vec<u8> = sd.serialize(&42).unwrap();
//! assert_eq!(serialized, [0, 0, 0, 42]);
//!
//! let deserialized = sd.deserialize(&mut serialized[..]);
//! assert_eq!(deserialized, Ok(42));
//! ```
#![cfg_attr(channels_nightly, feature(doc_auto_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod util;

use alloc::vec::Vec;

/// The [`Serializer`] trait allows converting a type `T` to safe-to-transport
/// byte sequences.
///
/// Types implementing this trait are called 'serializers'.
pub trait Serializer<T> {
	/// The error returned by [`Serializer::serialize()`].
	type Error;

	/// Serialize `t` to a buffer.
	fn serialize(&mut self, t: &T) -> Result<Vec<u8>, Self::Error>;
}

macro_rules! forward_serializer_impl {
	($to:ty) => {
		type Error = <$to>::Error;

		fn serialize(
			&mut self,
			t: &T,
		) -> Result<Vec<u8>, Self::Error> {
			<$to>::serialize(self, t)
		}
	};
}

/// The [`Deserializer`] trait allows converting a byte slice to a type `T`.
///
/// Types implementing this trait are called 'deserializers'.
pub trait Deserializer<T> {
	/// The error returned by [`Deserializer::deserialize()`].
	type Error;

	/// Deserialize bytes from `buf` to a type `T`.
	///
	/// Implementations can freely modify `buf` if needed.
	fn deserialize(
		&mut self,
		buf: &mut [u8],
	) -> Result<T, Self::Error>;
}

macro_rules! forward_deserializer_impl {
	($to:ty) => {
		type Error = <$to>::Error;

		fn deserialize(
			&mut self,
			buf: &mut [u8],
		) -> Result<T, Self::Error> {
			<$to>::deserialize(self, buf)
		}
	};
}

impl<T, U: Serializer<T> + ?Sized> Serializer<T> for &mut U {
	forward_serializer_impl!(U);
}

impl<T, U: Deserializer<T> + ?Sized> Deserializer<T> for &mut U {
	forward_deserializer_impl!(U);
}

#[cfg(feature = "alloc")]
impl<T, U: Serializer<T> + ?Sized> Serializer<T>
	for alloc::boxed::Box<U>
{
	forward_serializer_impl!(U);
}

#[cfg(feature = "alloc")]
impl<T, U: Deserializer<T> + ?Sized> Deserializer<T>
	for alloc::boxed::Box<U>
{
	forward_deserializer_impl!(U);
}

macro_rules! serdes_impl {
	($module:ident :: $impl:ident if $($cfg:tt)+) => {
		#[cfg($($cfg)+)]
		pub mod $module;
		#[cfg($($cfg)+)]
		pub use self::$module::$impl;
	};
}

#[cfg(feature = "aead")]
pub mod aead;

serdes_impl! { bincode::Bincode if feature = "bincode" }
serdes_impl! { cbor::Cbor       if feature = "cbor"    }
serdes_impl! { json::Json       if feature = "json"    }
serdes_impl! { borsh::Borsh     if feature = "borsh"   }
serdes_impl! { crc::Crc         if feature = "crc"     }
serdes_impl! { deflate::Deflate if feature = "deflate" }
serdes_impl! { hmac::Hmac       if feature = "hmac"    }

//! Utilities to serialize/deserialize types.
#![allow(
	unknown_lints,
	clippy::new_without_default,
	clippy::needless_doctest_main
)]
#![warn(
	clippy::all,
	clippy::style,
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

extern crate alloc;

use alloc::vec::Vec;

mod buf;

pub use self::buf::PayloadBuffer;

/// The [`Serializer`] trait allows converting a type `T` to safe-to-transport
/// byte sequences.
///
/// Types implementing this trait are called 'serializers'.
pub trait Serializer<T> {
	/// The error returned by [`Serializer::serialize()`].
	type Error;

	/// Serialize `t` into a `Vec<u8>`.
	fn serialize(
		&mut self,
		t: &T,
	) -> Result<PayloadBuffer, Self::Error>;
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
		buf: &mut Vec<u8>,
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

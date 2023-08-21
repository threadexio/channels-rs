//! Custom serializers/deserializers.
//!
//! # Middleware
//!
//! Middleware are types that sit in between the actual serializer or
//! deserializer, such as [`Bincode`], [`Cbor`] and [`Json`], and modify
//! the serialized data before it gets sent out or extend the functionality
//! of the packet.
//!
//! Some examples of the first type, which modify the serialized data, are:
//! - [`deflate`] and [`gzip`], which compress the serialized data, this
//! saving precious bandwidth.
//!
//! Some examples of the second type, which add functionality, are:
//! - [`crc`], which adds a simple CRC checksum at the end, so it can verify
//! on the receiving side that no corruption has occurred.
//! - [`hmac`], which adds a signature at the end, so it can verify on the
//! receiving side whether the packet was tampered with.
//!
//! **NOTE:** The 2 types discussed above are purely logical and only exist
//! as a model to understand what middleware can do. Nothing prevents you
//! from doing both of those things.
//!
//! ## Usage
//!
//! Let's assume that we need to communicate over an unreliable channel
//! that has a lot of interference and corrupts data (also assuming 0% loss).
//!
//! We can use the [`crc::Crc`] middleware in order to detect whether our
//! data has been corrupted. The following code is how we can do this.
//!
//! ```no_run
//! use channels::serdes::crc::{algorithm, Crc};
//! use channels::serdes::Bincode;
//!
//! fn main() {
//!     let serdes = Crc::builder()
//!         .algorithm(&algorithm::CRC_32_CKSUM) // you can skip this and it will use the default
//!         .build(Bincode::default());
//!
//!     let mut tx = channels::Sender::<i32, _, _>::with_serializer(todo!(), serdes.clone());
//!     let mut rx = channels::Receiver::<i32, _, _>::with_deserializer(todo!(), serdes);
//!
//!     // continue normally...
//! }
//! ```
//!
//! As it turns out, the channel also does not like transmitting a lot of data.
//! Now what? We can also use the [`deflate`] middleware and on top of that add
//! the CRC checksum.
//!
//! ```no_run
//! use channels::serdes::crc::{algorithm, Crc};
//! use channels::serdes::deflate::{Deflate, Compression};
//! use channels::serdes::Bincode;
//!
//! fn main() {
//!     let serdes = Bincode::default();
//!
//!     let serdes = Deflate::builder()
//!         .level(Compression::best()) // you can skip this and it will use the default
//!         .build(serdes);
//!
//!     let serdes = Crc::builder()
//!         .algorithm(&algorithm::CRC_32_CKSUM) // you can skip this and it will use the default
//!         .build(serdes);
//!
//!     let mut tx = channels::Sender::<i32, _, _>::with_serializer(todo!(), serdes.clone());
//!     let mut rx = channels::Receiver::<i32, _, _>::with_deserializer(todo!(), serdes);
//!
//!     // continue normally...
//! }
//! ```
//!
//! We have now effectively compressed our data and protected ourselves from
//! corruption in a couple lines of code. And we can do even more with the
//! rest of the middleware. And it is all transparent.
//!
//! ## How to make your own
//!
//! A middleware type is nothing more than a type that implements [`Serializer`]
//! and [`Deserializer`] but then delegates each call to the next type. Unfortunately,
//! for now, this requires some boilerplate, but you may have seen the pattern
//! in other crates as well.
//!
//! A simple middleware that prints to the screen can be implemented like this:
//!
//! ```no_run
//! use channels::serdes::prelude::*;
//! use channels::serdes::Bincode;
//!
//! #[derive(Debug, Clone)]
//! struct MyMiddleware<U> {
//!     next: U
//! }
//!
//! impl<T, U> Serializer<T> for MyMiddleware<U>
//! where
//!     U: Serializer<T>
//! {
//!     type Error = U::Error;
//!
//!     fn serialize<W: Write>(&mut self, buf: W, t: &T) -> Result<(), Self::Error> {
//!         println!("my serialize middleware");
//!         self.next.serialize(buf, t)
//!     }
//! }
//!
//! impl<T, U> Deserializer<T> for MyMiddleware<U>
//! where
//!     U: Deserializer<T>
//! {
//!     type Error = U::Error;
//!
//!     fn deserialize<R: Read>(&mut self, buf: R) -> Result<T, Self::Error> {
//!         println!("my deserialize middleware");
//!         self.next.deserialize(buf)
//!     }
//! }
//!
//! fn main() {
//!     let serdes = MyMiddleware { next: Bincode::default() };
//!
//!     let mut tx = channels::Sender::<i32, _, _>::with_serializer(todo!(), serdes.clone());
//!     let mut rx = channels::Receiver::<i32, _, _>::with_deserializer(todo!(), serdes);
//!
//!     // continue normally...
//! }
//! ```
//!
//! For the more complex middleware, see the implementations of the other
//! middleware from this crate.

use std::error::Error as StdError;

/// Traits and types needed to implement [`Serializer`] and [`Deserializer`].
pub mod prelude {
	pub use super::{Deserializer, Serializer};
	pub use std::io::{Read, Write};
}
use prelude::*;

/// A trait describing a simple type serializer.
///
/// Types implementing this trait are able to serialize an object of
/// type `T` into a bytes `Vec`.
///
/// # Example
///
/// ```rust
/// use std::fmt;
/// use std::error::Error;
/// use std::io::Write;
///
/// use channels::serdes::Serializer;
///
/// #[derive(Debug)]
/// struct I32SerializeError(String);
///
/// impl fmt::Display for I32SerializeError {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "failed to serialize i32: {}", self.0)
///     }
/// }
///
/// impl Error for I32SerializeError {}
///
/// struct I32Serializer;
///
/// impl Serializer<i32> for I32Serializer {
///     type Error = I32SerializeError;
///
///     fn serialize<W: Write>(&mut self, mut buf: W, t: &i32) -> Result<(), Self::Error> {
///         buf.write_all(&t.to_be_bytes())
///            .map_err(|e| I32SerializeError(e.to_string()))?;
///
///         Ok(())
///     }
///
///     fn size_hint(&mut self, _t: &i32) -> Option<usize> {
///         // We always serialize `i32`s as 4 bytes.
///         Some(4)
///     }
/// }
///
/// let mut ser = I32Serializer;
///
/// let data = 42_i32;
///
/// let mut buf = match ser.size_hint(&data) {
///     Some(size) => Vec::with_capacity(size),
///     None => Vec::new()
/// };
///
/// ser.serialize(&mut buf, &data).unwrap();
///
/// assert_eq!(&buf, &[0, 0, 0, 42]);
/// ```
pub trait Serializer<T> {
	/// The error type returned by [`Self::serialize`].
	type Error: StdError;

	/// Serialize the given object `t` and return the result as a `Vec<u8>`.
	fn serialize<W: Write>(
		&mut self,
		buf: W,
		t: &T,
	) -> Result<(), Self::Error>;

	/// Size approximation for the serialized object `t`.
	fn size_hint(&mut self, _t: &T) -> Option<usize> {
		None
	}
}

impl<T, U> Serializer<T> for &mut U
where
	U: Serializer<T>,
{
	type Error = U::Error;

	fn serialize<W: Write>(
		&mut self,
		buf: W,
		t: &T,
	) -> Result<(), Self::Error> {
		(**self).serialize(buf, t)
	}

	fn size_hint(&mut self, t: &T) -> Option<usize> {
		(**self).size_hint(t)
	}
}

/// A trait describing a simple type deserializer.
///
/// Types implementing this trait are able to deserialize an object of
/// type `T` from a series of bytes.
///
/// # Example
///
/// ```rust
/// use std::fmt;
/// use std::error::Error;
/// use std::io::Read;
///
/// use channels::serdes::Deserializer;
///
/// #[derive(Debug)]
/// struct I32DeserializeError(String);
///
/// impl fmt::Display for I32DeserializeError {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "failed to deserialize i32: {}", self.0)
///     }
/// }
///
/// impl Error for I32DeserializeError {}
///
/// struct I32Deserializer;
///
/// impl Deserializer<i32> for I32Deserializer {
///     type Error = I32DeserializeError;
///
///     fn deserialize<R: Read>(&mut self, mut buf: R) -> Result<i32, Self::Error> {
///         let mut be_bytes = [0u8; 4];
///         buf.read_exact(&mut be_bytes[..])
///            .map_err(|e| I32DeserializeError(e.to_string()))?;
///
///         let data = i32::from_be_bytes(be_bytes);
///         Ok(data)
///     }
/// }
///
/// let mut de = I32Deserializer;
///
/// let buf: [u8; 4] = [0, 0, 0, 42];
/// let data = de.deserialize(&buf[..]).unwrap();
///
/// assert_eq!(data, 42_i32);
/// ```
pub trait Deserializer<T> {
	/// The error type returned by [`Self::deserialize`].
	type Error: StdError;

	/// Deserializer an object of type `T` from `buf` and return it.
	fn deserialize<R: Read>(
		&mut self,
		buf: R,
	) -> Result<T, Self::Error>;
}

impl<T, U> Deserializer<T> for &mut U
where
	U: Deserializer<T>,
{
	type Error = U::Error;

	fn deserialize<R: Read>(
		&mut self,
		buf: R,
	) -> Result<T, Self::Error> {
		(**self).deserialize(buf)
	}
}

cfg_serde! {
	cfg_bincode! {
		mod bincode;
		pub use self::bincode::Bincode;
	}

	cfg_cbor! {
		mod cbor;
		pub use self::cbor::Cbor;
	}

	cfg_json! {
		mod json;
		pub use self::json::Json;
	}
}

cfg_flate2! {
	pub mod gzip;
	pub mod deflate;
}

cfg_crc! {
	pub mod crc;
}

cfg_hmac! {
	pub mod hmac;
}

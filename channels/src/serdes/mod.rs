//! Custom serializers/deserializers.

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
	#[path = "flate2/gzip.rs"]
	pub mod gzip;

	#[path = "flate2/deflate.rs"]
	pub mod deflate;
}

cfg_crc! {
	pub mod crc;
}

cfg_hmac! {
	pub mod hmac;
}

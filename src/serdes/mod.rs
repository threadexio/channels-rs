//! Custom serializers/deserializers.

use std::error::Error as StdError;

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
///
/// use channels::serdes::Serializer;
///
/// #[derive(Debug)]
/// struct I32SerializeError;
///
/// impl fmt::Display for I32SerializeError {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "failed to serialize i32")
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
///     fn serialize(&mut self, t: &i32) -> Result<Vec<u8>, Self::Error> {
///         let buf = t.to_be_bytes().to_vec();
///         Ok(buf)
///     }
/// }
///
/// let mut ser = I32Serializer;
///
/// let data = 42_i32;
/// let buf = ser.serialize(&data).unwrap();
///
/// assert_eq!(&buf, &[0, 0, 0, 42]);
/// ```
pub trait Serializer<T> {
	/// The error type returned by [`Self::serialize`].
	type Error: StdError + 'static;

	/// Serialize the given object `t` and return the result as a `Vec<u8>`.
	fn serialize(&mut self, t: &T) -> Result<Vec<u8>, Self::Error>;
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
///
/// use channels::serdes::Deserializer;
///
/// #[derive(Debug)]
/// struct I32DeserializeError(String);
///
/// impl fmt::Display for I32DeserializeError {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "{}", self.0)
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
///     fn deserialize(&mut self, buf: &[u8]) -> Result<i32, Self::Error> {
///         let be_bytes: &[u8; 4] = buf.try_into().map_err(|_| I32DeserializeError("failed to deserialize i32".to_string()))?;
///
///         let data = i32::from_be_bytes(*be_bytes);
///         Ok(data)
///     }
/// }
///
/// let mut de = I32Deserializer;
///
/// let buf = &[0, 0, 0, 42];
/// let data = de.deserialize(buf).unwrap();
///
/// assert_eq!(data, 42_i32);
/// ```
pub trait Deserializer<T> {
	/// The error type returned by [`Self::deserialize`].
	type Error: StdError + 'static;

	/// Deserializer an object of type `T` from `buf` and return it.
	fn deserialize(&mut self, buf: &[u8]) -> Result<T, Self::Error>;
}

#[cfg(feature = "serde")]
mod bincode;
#[cfg(feature = "serde")]
pub use self::bincode::Bincode;

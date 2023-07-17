use std::error::Error as StdError;

/// A trait describing a simple type serializer.
pub trait Serializer<T> {
	/// The error type returned by [`Self::serialize`].
	type Error: StdError + 'static;

	/// Serialize the given object `t` and return the result as a
	/// preallocated byte vec.
	fn serialize(&mut self, t: &T) -> Result<Vec<u8>, Self::Error>;
}

/// A trait describing a simple type deserializer.
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

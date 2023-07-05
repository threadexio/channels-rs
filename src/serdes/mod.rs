mod error;
pub use error::Error;

/// A trait describing a simple type serializer.
pub trait Serializer<T> {
	/// Serialize the given object `t` and return the result as a
	/// preallocated byte vec.
	fn serialize(&mut self, t: &T) -> Result<Vec<u8>, Error>;
}

/// A trait describing a simple type deserializer.
pub trait Deserializer<T> {
	/// Deserializer an object of type `T` from `buf` and return it.
	fn deserialize(&mut self, buf: &[u8]) -> Result<T, Error>;
}

pub(crate) mod impl_prelude {
	pub(super) use super::error::Error;
	pub use super::{Deserializer, Serializer};
}

#[cfg(feature = "serde")]
mod bincode;
#[cfg(feature = "serde")]
pub use self::bincode::Bincode;

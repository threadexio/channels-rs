use alloc::vec::Vec;

/// Encode an item into a buffer of bytes.
pub trait Encoder {
	/// The type of items the encoder accepts.
	type Item;

	/// Error type returned by the encoder.
	type Error;

	/// Encode `item` into `buf`.
	///
	/// Implementations should only append data to `buf`. Additionally, it is not guaranteed
	/// that `buf` is empty on each call of this method.
	fn encode(
		&mut self,
		item: Self::Item,
		buf: &mut Vec<u8>,
	) -> Result<(), Self::Error>;
}

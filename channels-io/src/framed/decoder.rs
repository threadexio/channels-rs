use alloc::vec::Vec;

/// Decode an item from a buffer of bytes.
pub trait Decoder {
	/// The type of items the decoder accepts.
	type Output;

	/// Error type returned by the decoder.
	type Error;

	/// Decode an item from `buf`.
	///
	/// Implementations should remove data from `buf` as each item is decoded.
	fn decode(
		&mut self,
		buf: &mut Vec<u8>,
	) -> Result<Option<Self::Output>, Self::Error>;
}

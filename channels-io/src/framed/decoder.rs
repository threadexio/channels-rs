use alloc::vec::Vec;

/// TODO: docs
pub trait Decoder {
	/// TODO: docs
	type Output;

	/// TODO: docs
	type Error;

	/// TODO: docs
	fn decode(
		&mut self,
		buf: &mut Vec<u8>,
	) -> Option<Result<Self::Output, Self::Error>>;
}

use alloc::vec::Vec;

/// TODO: docs
pub trait Encoder {
	/// TODO: docs
	type Item: ?Sized;

	/// TODO: docs
	type Error;

	/// TODO: docs
	fn encode(
		&mut self,
		item: &Self::Item,
		buf: &mut Vec<u8>,
	) -> Result<(), Self::Error>;
}

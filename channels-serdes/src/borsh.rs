use alloc::vec::Vec;

use borsh::{BorshDeserialize, BorshSerialize};

use crate::{Deserializer, Serializer};

/// The [`mod@borsh`] serializer which automatically works with all
/// types that implement [`borsh::BorshSerialize`] and [`borsh::BorshDeserialize`].
#[derive(Debug, Default, Clone)]
pub struct Borsh {}

impl Borsh {
	/// Create a new [`Borsh`].
	#[must_use]
	pub fn new() -> Self {
		Self {}
	}
}

impl<T> Serializer<T> for Borsh
where
	T: BorshSerialize,
{
	type Error = borsh::io::Error;

	fn serialize(&mut self, t: &T) -> Result<Vec<u8>, Self::Error> {
		borsh::to_vec(t)
	}
}

impl<T> Deserializer<T> for Borsh
where
	T: BorshDeserialize,
{
	type Error = borsh::io::Error;

	fn deserialize(
		&mut self,
		buf: &mut Vec<u8>,
	) -> Result<T, Self::Error> {
		borsh::from_slice(buf)
	}
}

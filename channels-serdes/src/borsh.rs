//! The [`mod@borsh`] serializer which automatically works with all
//! types that implement [`borsh::BorshSerialize`] and [`borsh::BorshDeserialize`].

use borsh::{BorshDeserialize, BorshSerialize};

use channels_io::{ContiguousMut, Walkable};

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

	fn serialize(
		&mut self,
		t: &T,
	) -> Result<impl Walkable, Self::Error> {
		let vec = borsh::to_vec(t)?;
		Ok(channels_io::Cursor::new(vec))
	}
}

impl<T> Deserializer<T> for Borsh
where
	T: BorshDeserialize,
{
	type Error = borsh::io::Error;

	fn deserialize(
		&mut self,
		mut buf: impl ContiguousMut,
	) -> Result<T, Self::Error> {
		borsh::from_slice(buf.chunk_mut())
	}
}

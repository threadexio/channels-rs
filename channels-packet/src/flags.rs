bitflags::bitflags! {
	/// Header flags.
	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
	pub struct Flags: u8 {
		/// More data flag.
		const MORE_DATA = 1 << 7;
	}
}

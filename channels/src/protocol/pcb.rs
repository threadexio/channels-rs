use channels_packet::id::IdSequence;

#[derive(Clone)]
pub struct Pcb {
	pub(super) seq: IdSequence,
}

impl Pcb {
	pub fn new() -> Self {
		Self { seq: IdSequence::new() }
	}
}

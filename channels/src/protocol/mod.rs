mod recv;
mod send;

pub use self::recv::{recv_async, recv_sync, RecvPcb};
pub use self::send::{send_async, send_sync, SendPcb};

use channels_packet::id::IdSequence;

#[derive(Clone, Default)]
pub struct Pcb<S> {
	pub seq: IdSequence,
	pub state: S,
}

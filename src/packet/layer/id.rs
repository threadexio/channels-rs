use super::prelude::*;

/// Structure:
///
///
/// `id`: u16
pub struct Id<T> {
	next: T,
	seq_id: u16,
}

impl<T> Id<T> {
	const HEADER_SIZE: usize = 2;

	pub fn new(next: T) -> Self {
		Self { next, seq_id: 0 }
	}

	fn get_next_id(&self) -> u16 {
		self.seq_id.wrapping_add(1)
	}
}

impl<T> Layer for Id<T>
where
	T: Layer,
{
	fn payload<'a>(&mut self, buf: &'a mut [u8]) -> &'a mut [u8] {
		self.next.payload(&mut buf[Self::HEADER_SIZE..])
	}

	fn on_send<'a>(
		&mut self,
		buf: &'a mut [u8],
	) -> Result<&'a mut [u8]> {
		write_offset(buf, 0, self.seq_id.to_be());
		self.seq_id = self.get_next_id();

		self.next.on_send(&mut buf[Self::HEADER_SIZE..])
	}

	fn on_recv<'a>(
		&mut self,
		buf: &'a mut [u8],
	) -> Result<&'a mut [u8]> {
		let read_seq_id = u16::from_be(read_offset(buf, 0));
		if read_seq_id != self.seq_id {
			return Err(Error::OutOfOrder);
		}

		self.seq_id = self.get_next_id();
		self.next.on_send(&mut buf[Self::HEADER_SIZE..])
	}
}

use crate::error::*;

pub trait Layer: Sized {
	/// Return the slice that holds this layer's payload. This method
	/// _**must**_ propagate the call to the next layer.
	///
	/// **Note**: The slice must not be end-bounded.
	fn payload<'a>(&mut self, buf: &'a mut [u8]) -> &'a mut [u8];

	/// Called after the payload has been written to the slice
	/// returned by [`Layer::payload()`]. This method _**must**_
	/// propagate the call to the next layer unless it encounters
	/// an error, in which case it should immediately return.
	///
	/// The return value is the slice that contains this layer's payload.
	///
	/// **Note**: The slice must not be end-bounded.
	fn on_send<'a>(
		&mut self,
		buf: &'a mut [u8],
	) -> Result<&'a mut [u8]>;

	/// Called after the entire packet has been read. This method
	/// _**must**_ propagate the call to the next layer unless it
	/// encounters an error, in which case it should immediately
	/// return.
	///
	/// The return value is the slice that contains this layer's payload.
	///
	/// **Note**: The slice must not be end-bounded.
	fn on_recv<'a>(
		&mut self,
		buf: &'a mut [u8],
	) -> Result<&'a mut [u8]>;
}

// `()` is the final layer
impl Layer for () {
	fn payload<'a>(&mut self, buf: &'a mut [u8]) -> &'a mut [u8] {
		buf
	}

	fn on_send<'a>(
		&mut self,
		buf: &'a mut [u8],
	) -> Result<&'a mut [u8]> {
		Ok(buf)
	}

	fn on_recv<'a>(
		&mut self,
		buf: &'a mut [u8],
	) -> Result<&'a mut [u8]> {
		Ok(buf)
	}
}

pub(self) mod prelude {
	pub use super::Layer;
	pub use crate::error::*;
	pub use crate::util::*;
}

mod id;
pub use id::Id;

use super::consts::*;
use super::types::*;
use crate::error::VerifyError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
	pub length: PacketLength,
	pub flags: Flags,
	pub id: Id,
}

pub type HeaderRaw = [u8; HEADER_SIZE];

impl Header {
	/// Write the header to `buf`.
	///
	/// This function also calculates the checksum for the header.
	pub fn write_to(&self, buf: &mut HeaderRaw) {
		unsafe {
			private::unsafe_set_version(buf, private::HEADER_HASH);
			private::unsafe_set_packet_length(
				buf,
				self.length.into(),
			);
			private::unsafe_set_flags(buf, self.flags.0);
			private::unsafe_set_packet_id(buf, self.id.0);

			private::unsafe_set_header_checksum(buf, 0);
			let checksum = Checksum::calculate(buf);
			private::unsafe_set_header_checksum(buf, checksum.0);
		}
	}

	/// Read the header from `buf` without performing any checks.
	pub unsafe fn read_from_unchecked(buf: &HeaderRaw) -> Self {
		unsafe {
			Self {
				length: PacketLength::try_from(
					private::unsafe_get_packet_length(buf),
				)
				.unwrap(),
				flags: Flags(private::unsafe_get_flags(buf)),
				id: Id(private::unsafe_get_packet_id(buf)),
			}
		}
	}

	/// Read the header from `buf`.
	///
	/// **NOTE:** This method does not verify the `id` field.
	pub fn read_from(buf: &HeaderRaw) -> Result<Header, VerifyError> {
		unsafe {
			if private::unsafe_get_version(buf)
				!= private::HEADER_HASH
			{
				return Err(VerifyError::VersionMismatch);
			}

			if !Checksum::verify(buf) {
				return Err(VerifyError::VersionMismatch);
			}

			if usize::from(private::unsafe_get_packet_length(buf))
				< HEADER_SIZE
			{
				return Err(VerifyError::InvalidHeader);
			}
		}

		Ok(unsafe { Self::read_from_unchecked(buf) })
	}
}

#[allow(dead_code, clippy::all)]
pub mod private {
	use crate::mem::*;
	include!("generated.rs");
}

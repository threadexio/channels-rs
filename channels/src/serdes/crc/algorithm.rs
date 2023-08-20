//! A collection of crc algorithms for use with the middleware.

// TODO: Find a way to make `Width` generic.
pub(crate) type Width = u32;

// TODO: If `Width` is made generic, this module should just reexport
//       every algorithm.
pub use crc::{
	CRC_32_AIXM, CRC_32_AUTOSAR, CRC_32_BASE91_D, CRC_32_BZIP2,
	CRC_32_CD_ROM_EDC, CRC_32_CKSUM, CRC_32_ISCSI, CRC_32_ISO_HDLC,
	CRC_32_JAMCRC, CRC_32_MEF, CRC_32_MPEG_2, CRC_32_XFER,
};

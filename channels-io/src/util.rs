use core::cmp::min;

/// Copy the maximum number of bytes possible from `src` into `dst`.
///
/// Returns the number of bytes copied.
pub fn copy_slice(src: &[u8], dst: &mut [u8]) -> usize {
	let n = min(src.len(), dst.len());
	if n != 0 {
		dst[..n].copy_from_slice(&src[..n]);
	}
	n
}

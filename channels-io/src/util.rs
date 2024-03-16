/// Copy as many bytes as possible from `src` into `dst`.
///
/// Returns the amount of bytes copied.
pub fn copy_min_len(src: &[u8], dst: &mut [u8]) -> usize {
	let n = core::cmp::min(src.len(), dst.len());
	if n != 0 {
		dst[..n].copy_from_slice(&src[..n]);
	}
	n
}

pub use core::future::Future;

use alloc::vec::Vec;

/// Grow `vec` by `n` bytes and return the newly allocated bytes as a mutable
/// slice.
#[inline]
pub fn grow_vec_by_n(vec: &mut Vec<u8>, n: usize) -> &mut [u8] {
	let old_len = vec.len();
	let new_len = usize::saturating_add(old_len, n);

	vec.resize(new_len, 0);
	&mut vec[old_len..new_len]
}

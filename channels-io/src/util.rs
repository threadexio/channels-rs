#![allow(dead_code)]

use core::cmp::min;
use core::future::Future;
use core::mem::MaybeUninit;

/// Convert a `&mut [MaybeUninit<T>]` to a `&mut [T]`.
///
/// # Safety
///
/// The caller must ensure that all elements in `x` are properly initialized.
pub unsafe fn slice_uninit_assume_init_mut<T>(
	x: &mut [MaybeUninit<T>],
) -> &mut [T] {
	// SAFETY: `MaybeUninit` is `repr(transparent)` and thus identical, in term of ABI, to
	//         a `T`. Casting one to the other is completely safe as long as `T` is properly
	//         initialized.
	let data = x.as_mut_ptr().cast::<T>();
	let len = x.len();

	core::slice::from_raw_parts_mut(data, len)
}

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

#[inline]
pub fn assert_future<T, F>(f: F) -> F
where
	F: Future<Output = T>,
{
	f
}

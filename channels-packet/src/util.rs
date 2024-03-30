//! Miscellaneous utilities that don't fit anywhere else.

/// Assert a condition at compile-time.
///
/// This macro accepts exactly the same arguments as [`assert!`].
macro_rules! static_assert {
	($($tt:tt)*) => {
		#[allow(dead_code, clippy::assertions_on_constants)]
		const _: () = assert!($($tt)*);
	}
}
pub(crate) use static_assert;

/// Convert a `&[T]` to a `&[T; N]`.
///
/// # Safety
///
/// The caller must ensure that `slice` is valid for at least `N` elements.
///
/// # Panics
///
/// Panics if the length of the slice is less than `N`.
#[track_caller]
pub unsafe fn slice_to_array<T, const N: usize>(
	slice: &[T],
) -> &[T; N] {
	assert!(slice.len() >= N, "slice is smaller than N");
	slice
		.as_ptr()
		.cast::<[T; N]>()
		.as_ref()
		.expect("failed to cast N-sized slice to N-sized array")
}

/// Convert a `&mut [T]` to a `&mut [T; N]`.
///
/// # Safety
///
/// The caller must ensure that `slice` is valid for at least `N` elements.
///
/// # Panics
///
/// Panics if the length of the slice is less than `N`.
#[track_caller]
pub unsafe fn slice_to_array_mut<T, const N: usize>(
	slice: &mut [T],
) -> &mut [T; N] {
	assert!(slice.len() >= N, "slice is smaller than N");
	slice
		.as_mut_ptr()
		.cast::<[T; N]>()
		.as_mut()
		.expect("failed to cast N-sized slice to N-sized array")
}

use core::cmp::min;
use core::mem::MaybeUninit;
use core::task::Poll;

/// Extension trait for [`Poll`].
///
/// This traits provides convinience methods like `unwrap` but for [`Poll`].
pub trait PollExt: Sized {
	type Inner;

	/// Unwrap a `Poll::Ready(T)`.
	fn unwrap(self) -> Self::Inner;
}

impl<T> PollExt for Poll<T> {
	type Inner = T;

	fn unwrap(self) -> Self::Inner {
		#[inline(never)]
		#[track_caller]
		#[cold]
		fn poll_pending_fail() -> ! {
			panic!("unwrapped a Poll::Pending");
		}

		match self {
			Poll::Ready(x) => x,
			Poll::Pending => poll_pending_fail(),
		}
	}
}

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

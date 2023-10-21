//! Abstractions on sync & async IO traits.
#![allow(
	unknown_lints,
	clippy::new_without_default,
	clippy::needless_doctest_main
)]
#![warn(
	clippy::all,
	clippy::style,
	clippy::perf,
	clippy::correctness,
	clippy::complexity,
	clippy::deprecated,
	clippy::missing_doc_code_examples,
	clippy::missing_panics_doc,
	clippy::missing_safety_doc,
	clippy::missing_doc_code_examples,
	clippy::cast_lossless,
	clippy::cast_possible_wrap,
	clippy::useless_conversion,
	clippy::wrong_self_convention,
	rustdoc::all,
	rustdoc::broken_intra_doc_links
)]
#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

mod buf;
mod util;

mod r#async;
mod sync;

pub use self::buf::{IoSlice, IoSliceMut, IoSliceRef};
pub use self::util::{copy_slice, Bytes, BytesMut, PollExt};

pub use self::r#async::{
	AsyncRead, AsyncWrite, IntoAsyncReader, IntoAsyncWriter,
};
pub use self::sync::{IntoReader, IntoWriter, Read, Write};

/// Common trait imports.
pub mod prelude {
	pub use super::{
		AsyncRead, AsyncWrite, IntoAsyncReader, IntoAsyncWriter,
		IntoReader, IntoWriter, PollExt, Read, Write,
	};
}

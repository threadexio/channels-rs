#[cfg(has_core_error)]
pub use core::error::Error;

#[cfg(all(not(has_core_error), feature = "std"))]
pub use std::error::Error;

#[cfg(all(not(has_core_error), not(feature = "std")))]
#[allow(unused)]
pub trait Error: core::fmt::Debug + core::fmt::Display {}

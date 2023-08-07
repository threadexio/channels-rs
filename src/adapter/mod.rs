//! Split functionality for generic [`Read`] and [`Write`] types.
//!
//! **NOTE:** Because of the generic nature of this API it is impossible to achieve
//! true splitting of the 2 halves. Should you use a type which supports
//! this functionality natively, use its own API for splitting. One such
//! type that implements this is [`std::net::TcpStream`], which has `try_clone()`.
//!
//! There are 2 variants of the API:
//! - [`unsync`]
//! - [`sync`]
//!
//! The difference between the two is the synchronization primitive used.
//! The [`unsync`], unlike the [`sync`], variant does not implement [`Send`]
//! or [`Sync`] for any of its types, and as such cannot be shared or sent to
//! other threads.
//!
//! # Example
//! ```no_run
//! use channels::adapter::unsync::split;
//! // or: use channels::adapter::sync::split;
//!
//! use std::io::{Read, Write, Cursor};
//!
//! let rw = Cursor::new(vec![0u8; 32]);
//! let (mut r, mut w) = split(rw);
//!
//! let mut buf = vec![0u8; 16];
//!
//! // Read from the read half
//! let _ = r.read(&mut buf).unwrap();
//!
//! // Write to the write half
//! let _ = w.write(&mut buf).unwrap();
//! ```
//!
//! [`Read`]: std::io::Read
//! [`Write`]: std::io::Write

pub mod sync;
pub mod unsync;

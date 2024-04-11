# channels-io

This crate contains generic IO traits that abstract over little implementation
differences of different libraries. With this crate it is possible to write IO
code that is agnostic over sync/async operation and, if using async, over the
runtime that is used. Additionally, this crate implements a generic interface
for buffers, a bit like [`bytes`](https://docs.rs/bytes/latest/bytes) but with
some subtle differences.

## Features

|  Feature  | Description                                                       |
|:---------:|:------------------------------------------------------------------|
|  `alloc`  | Enable implementations for `alloc` structures (`Box`, `Vec`, ...) |
|   `std`   | Abstract over `std::io::Read` and `std::io::Write`                |
|  `tokio`  | Abstract over `tokio::io::AsyncRead` and `tokio::io::AsyncWrite`  |
| `futures` | Abstract over `futures::AsyncRead` and `futures::AsyncWrite`      |
|  `core2`  | Abstract over `core2::io::Read` and `core2::io::Write`            |

**Note:** The API of this crate is _not_ final and may change at any time
without necessarily a major version bump. If you must depend on it pin down the
full version of the crate to avoid future incompatibilities.

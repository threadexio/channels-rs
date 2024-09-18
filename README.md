[crates-io]: https://crates.io/crates/channels
[docs-rs]: https://docs.rs/channels/latest/channels
[github-actions]: https://github.com/threadexio/channels-rs/actions/workflows/ci.yaml

[license-badge]: https://img.shields.io/github/license/threadexio/channels-rs?style=for-the-badge&logo=github&label=license&labelColor=%23000&color=%236e00f2
[tests-badge]: https://img.shields.io/github/actions/workflow/status/threadexio/channels-rs/ci.yaml?style=for-the-badge&logo=github&label=tests&labelColor=%23000&color=%239500d6
[version-badge]: https://img.shields.io/crates/v/channels?style=for-the-badge&logo=rust&label=crates.io&labelColor=%23000&color=%23bc00ba
[docs-badge]: https://img.shields.io/docsrs/channels?style=for-the-badge&logo=docs.rs&labelColor=%23000&color=%23e2009e
[downloads-badge]: https://img.shields.io/crates/d/channels?style=for-the-badge&label=downloads&labelColor=%23000&color=%23ff0089

[examples]: https://github.com/threadexio/channels-rs/tree/master/examples
[spec]: https://github.com/threadexio/channels-rs/blob/master/spec/PROTOCOL.md
[license]: https://github.com/threadexio/channels-rs/blob/master/LICENSE
[art-license]: https://github.com/threadexio/channels-rs/blob/master/assets/LICENSE

<div class="rustdoc-hidden">

<div align="center">
  <img src="https://raw.githubusercontent.com/threadexio/channels-rs/master/assets/logo.transparent.svg" width="640" alt="logo">

  <p>
    Easy and fast communication between processes, threads and systems.
  </p>

  [![license-badge]][crates-io]
  [![tests-badge]][github-actions]
  [![version-badge]][crates-io]
  [![docs-badge]][docs-rs]
  [![downloads-badge]][crates-io]

</div>

<br>

</div>

Sender/Receiver types for communicating with a channel-like API across generic IO streams.
It takes the burden on serializing, deserializing and transporting data off your back and
let's you focus on the important logic of your project.

## Contents

* [Contents](#contents)
* [Examples](#examples)
* [Features](#features)
* [How it works](#how-it-works)
* [License](#license)

## Examples

```toml
[dependencies.channels]
version = "0.12"
features = ["full"]
```

```rust no_run
use tokio::net::TcpStream;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
enum Message {
    Ping,
    Pong
}

#[tokio::main]
async fn main() {
    let stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    let (r, w) = stream.into_split();
    let (mut tx, mut rx) = channels::channel::<Message, _, _>(r, w);

    loop {
        match rx.recv().await.unwrap() {
            Message::Ping => {
                println!("pinged!");
                tx.send(Message::Pong).await.unwrap();
            }
            Message::Pong => {
                println!("ponged!");
            }
        }
    }
}
```

For more, see: [examples/][examples]

## Features

| Flag          | Description                                                                                               |
|---------------|-----------------------------------------------------------------------------------------------------------|
| `aead`        | Encrypt and authenticate data with [`ring::aead`].                                                        |
| `bincode`     | Serialize/Deserialize data with [`bincode`]. (Enabled by default)                                         |
| `borsh`       | Serialize/Deserialize data with [`borsh`].                                                                |
| `cbor`        | Serialize/Deserialize data with [`ciborium`].                                                             |
| `core2`       | Support for [`core2::io::{Read, Write}`][].                                                               |
| `crc`         | Validate data with a CRC checksum.                                                                        |
| `deflate`     | Compress data with DEFLATE.                                                                               |
| `embedded-io` | Support for [`embedded_io::{Read, Write}`][].                                                             |
| `full-io`     | Enable support for all of the IO traits.                                                                  |
| `full-serdes` | Enable features: `aead`, `bincode`, `borsh`, `cbor`, `crc`, `deflate`, `hmac`, `json`.                    |
| `futures`     | Support for [`futures::io::{AsyncRead, AsyncWrite}`][].                                                   |
| `hmac`        | Authenticate data with a HMAC using [`ring::hmac`].                                                       |
| `json`        | Serialize/Deserialize data with [`serde_json`].                                                           |
| `smol`        | Support for [`smol::io::{AsyncRead, AsyncWrite}`][].                                                      |
| `statistics`  | Collect IO metrics such as total bytes sent/received. See: [`Statistics`][].                              |
| `std`         | Support for [`std::io::{Read, Write}`][]. If disabled also makes the crate `no_std`. (Enabled by default) |
| `tokio`       | Support for [`tokio::io::{AsyncRead, AsyncWrite}`][].                                                     |

No two features of the crate are mutually exclusive. Instead, everything is implemented in
a way to be infinitely extensible. This means, that even if you have other IO traits or another
way to serialize or deserialize data, you can either add support for them directly in your own
project by using the rich type system.

## How it works

Channels implements a communication protocol that allows sending and receiving data in
frames. The main API of the crate is intended to work over IO traits. However, if desired,
the logic of the underlying protocol is available standalone without coupling it to the
usage of any IO traits. It works over _any_ stream synchronous or asynchronous with first
class support for following IO traits:

* [`std::io::{Read, Write}`][]
* [`tokio::io::{AsyncRead, AsyncWrite}`][]
* [`futures::io::{AsyncRead, AsyncWrite}`][]
* [`core2::io::{Read, Write}`][]
* [`smol::io::{AsyncRead, AsyncWrite}`][]
* [`embedded_io::{Read, Write}`][]

Support for each IO trait can be enabled via the corresponding feature flag. See: [Features](#features).
You can read more about how the underlying communication protocol works [here][spec].

## License

* All code in this repository is licensed under the MIT license, a copy of which can be
  found [here][license].

* All artwork in this repository is licensed under [Creative Commons Attribution-NonCommercial 4.0 International](https://creativecommons.org/licenses/by-nc/4.0/). A copy of the license can be found [here][art-license].

[`std::io::{Read, Write}`]: https://doc.rust-lang.org/stable/std/io
[`tokio::io::{AsyncRead, AsyncWrite}`]: https://docs.rs/tokio/latest/tokio/io
[`futures::io::{AsyncRead, AsyncWrite}`]: https://docs.rs/futures/latest/futures/io
[`core2::io::{Read, Write}`]: https://docs.rs/core2
[`smol::io::{AsyncRead, AsyncWrite}`]: https://docs.rs/smol
[`embedded_io::{Read, Write}`]: https://docs.rs/embedded-io
[`Statistics`]: https://docs.rs/channels/latest/channels/struct.Statistics.html

[`ring::aead`]: https://docs.rs/ring/latest/ring/aead/index.html
[`bincode`]: https://github.com/bincode-org/bincode
[`borsh`]: https://github.com/near/borsh-rs
[`ciborium`]: https://github.com/enarx/ciborium
[`ring::hmac`]: https://docs.rs/ring/latest/ring/hmac/index.html
[`serde_json`]: https://github.com/serde-rs/json

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

Sender/Receiver types for communicating with a channel-like API across generic IO streams. It takes the burden on serializing, deserializing and transporting data off your back and let's you focus on the important logic of your project. It is:

- **Fast**: The simple protocol allows low-overhead transporting of data.

- **Modular**: Channels' _sans-io_ approach means it can be used on top of any medium, be it a network socket, a pipe, a shared memory region, a file, anything.

- **Ergonomic**: The API offered empowers you to use your time on building the logic of your application instead of worrying about data transport.

- **Async & sync first**: Channels natively supports both synchronous and asynchronous operation with no hacky workarounds like spawning threads or running a separate runtime.

# In action

```toml
[dependencies.channels]
version = "0.12.0"
features = ["full"]
```

```rust
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

# How it works

Channels implements a communication protocol that allows sending and receiving data across any medium. It works over _any_ stream synchronous or asynchronous. Currently it can work with any of the following IO traits:

- [`std::io::{Read, Write}`](https://doc.rust-lang.org/stable/std/io)
- [`tokio::io::{AsyncRead, AsyncWrite}`](https://docs.rs/tokio/latest/tokio/io)
- [`futures::io::{AsyncRead, AsyncWrite}`](https://docs.rs/futures/latest/futures/io)
- [`core2::io::{Read, Write}`](https://docs.rs/core2)
- [`smol::io::{AsyncRead, AsyncWrite}`](https://docs.rs/smol)

You can find out more about how the underlying communication protocol works [here][spec].

# License

- All code in this repository is licensed under the MIT license, a copy of which can be found [here][license].

- All artwork in this repository is licensed under [Creative Commons Attribution-NonCommercial 4.0 International](https://creativecommons.org/licenses/by-nc/4.0/). A copy of the license can be found [here][art-license].

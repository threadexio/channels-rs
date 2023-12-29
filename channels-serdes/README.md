[`channels`]: (htpts://github.com/threadexio/channels-rs)
[`serde`]: https://github.com/serde-rs/serde
[`bincode`]: https://github.com/bincode-org/bincode
[`ciborium`]: https://github.com/enarx/ciborium
[`serde_json`]: https://github.com/serde-rs/json

# channels-serdes

This crate exposes the interface used by [`channels`] to serialize and deserialize arbitrary types.

It is simply an abstraction layer for different implementations that might not necessarily rely on [`serde`].

The crate contains 3 reference implementations that are all usable under [`channels`] and can be enabled with feature flags.

| Name      | Implemented By | Feature flag |
|:----------|:--------------:|:------------:|
| `Bincode` |  [`bincode`]   |  `bincode`   |
| `Cbor`    |  [`ciborium`]  |    `cbor`    |
| `Json`    | [`serde_json`] |    `json`    |

`Bincode` is the default implementation used by [`channels`].

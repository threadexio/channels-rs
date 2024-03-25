[`channels`]: (htpts://github.com/threadexio/channels-rs)
[`serde`]: https://github.com/serde-rs/serde
[`bincode`]: https://github.com/bincode-org/bincode
[`ciborium`]: https://github.com/enarx/ciborium
[`serde_json`]: https://github.com/serde-rs/json
[`borsh`]: https://github.com/near/borsh-rs
[`crc`]: https://github.com/mrhooray/crc-rs

# channels-serdes

This crate exposes the interface used by [`channels`] to serialize and deserialize arbitrary types.

It is simply an abstraction layer for different implementations that might not necessarily rely on [`serde`].

The crate contains reference implementations that are all usable under [`channels`] and can be enabled with feature flags.

## Serializers/Deserializers

| Name      | Implemented By | Feature flag |
|:----------|:--------------:|:------------:|
| `Bincode` |  [`bincode`]   |  `bincode`   |
| `Cbor`    |  [`ciborium`]  |    `cbor`    |
| `Json`    | [`serde_json`] |    `json`    |
| `Borsh`   |   [`borsh`]    |   `borsh`    |

`Bincode` is the default implementation used by [`channels`].

## Middleware

| Name  | Implemented By | Feature Flag |
|:------|:--------------:|:------------:|
| `Crc` |    [`crc`]     |    `crc`     |

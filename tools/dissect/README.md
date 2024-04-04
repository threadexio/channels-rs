# What's this?

This tool is a packet dissector and can be used to find errors in implementations of the protocol, as well as debug them. It is a standalone binary that can be
built with:

```bash
cargo build --package dissect
```

And ran withL

```bash
cargo run --package dissect
```

## Usage

The program expects a `Sender` be hooked up to its standard input. When the sender sends a packet, this programs will:

1. Try to parse it
2. Dump the header in a nice tree
3. Highlight any errors the header has (incorrect checksum, invalid version, etc)
4. Write the packet back to its standard output

This permits the following:

```bash
./sender | cargo -q run --package dissect | ./receiver
```

Or you can also save the stream for further analyzing with `tee`:

```bash
./sender | cargo -q run --package dissect | tee stream.bin | ./receiver
```

The `examples/` directory contains 2 example binaries for testing out this tool. Simply do:

```bash
cargo -q run --package examples --example=send_stdout | cargo -q run --package dissect | cargo -q run --package examples --example=recv_stdin
```

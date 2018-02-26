# protoc-grpcio
[![crates.io](https://img.shields.io/crates/v/protoc-grpcio.svg)](https://crates.io/crates/protoc-grpcio) 
[![Build Status](https://travis-ci.org/mtp401/protoc-grpcio.svg?branch=master)](https://travis-ci.org/mtp401/protoc-grpcio)
[![License](https://img.shields.io/crates/l/protoc-grpcio.svg)](https://github.com/mtp401/protoc-grpcio/blob/master/LICENSE)

A programmatic API to the
[grpc-rs compiler](https://github.com/pingcap/grpc-rs).

## Requirements

- You must have Google's Protocol Buffer compiler (`protoc`) installed and in
  `PATH`.

## Example `build.rs`

```rust
extern crate protoc_grpcio;

protoc_grpcio::compile_grpc_protos(
    &["example/protobuf.proto"],
    &["example"],
    "output"
).expect("failed to compile gRPC definitions");
```

## Example `Cargo.toml`

```yaml
[package]
# ...
build = "build.rs"

[build-dependencies]
protoc-grpcio = "0.1.0"
```

## Credits

Credit to both the TiKV project developers for
([grpc-rs](https://github.com/pingcap/grpc-rs)) and Stepan Koltsov
(@stepancheg, [rust-protobuf](https://github.com/stepancheg/rust-protobuf))
for their amazing work bringing Protocol Buffers and gRPC support to Rust.

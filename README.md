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

For a project laid out like so:
```
$ tree
.
├── build.rs
├── Cargo.toml
└── src
    ├── client.rs
    ├── protos
    │   ├── example
    │   │   └── diner.proto
    │   └── mod.rs
    └── server.rs

3 directories, 7 files
```

The `build.rs` might look like:
```rust
extern crate protoc_grpcio;

fn main() {
    let proto_root = "src/protos";
    println!("cargo:rerun-if-changed={}", proto_root);
    protoc_grpcio::compile_grpc_protos(
        &["example/diner.proto"],
        &[proto_root],
        &proto_root,
        None
    ).expect("Failed to compile gRPC definitions!");
}
```

## Example `Cargo.toml`

And the `Cargo.toml` might look like:
```toml
[package]
# ...
build = "build.rs"

[lib]
name = "protos"
path = "src/protos/mod.rs"

[[bin]]
name = "server"
path = "src/server.rs"

[[bin]]
name = "client"
path = "src/client.rs"

[dependencies]
futures = "0.1.16"
grpcio = "0.4.3"
protobuf = "~2"

[build-dependencies]
protoc-grpcio = "1.0.2"
```

You can inspect this example under [`example/`](example) by compiling and running the example
server in one shell session:
```
cargo run --manifest-path example/Cargo.toml --bin server
...
    Finished dev [unoptimized + debuginfo] target(s) in 27.97 secs
     Running `example/target/debug/server`
listening on 127.0.0.1:34431
```

And then running the client in another:
```
$ cargo run --manifest-path example/Cargo.toml --bin client 34431
...
    Finished dev [unoptimized + debuginfo] target(s) in 1.28 secs
     Running `example/target/debug/client 34431`
Ate items: SPAM items: EGGS and got charged $0.30
```

## Credits

Credit to both the TiKV project developers for
([grpc-rs](https://github.com/pingcap/grpc-rs)) and Stepan Koltsov
(@stepancheg, [rust-protobuf](https://github.com/stepancheg/rust-protobuf))
for their amazing work bringing Protocol Buffers and gRPC support to Rust.

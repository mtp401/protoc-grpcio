// Copyright 2018, 2021. Matthew Pelland <matt@pelland.io>.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// See the License for the specific language governing permissions and
// limitations under the License.
#![deny(warnings)]
#![warn(missing_docs)]
//! An API for programmatically invoking the grpcio gRPC compiler in the same vein as the
//! [rust-protoc-grpc](https://crates.io/crates/protoc-rust-grpc) crate from Stepan Koltsov.

mod compile;
mod error;

/// Re-export of the codegen crate used internally. This can be used by downstream consumers to
/// provide customized options to `compile_grpc_protos` via the `Customize` struct.
pub use protobuf_codegen;

pub use crate::{compile::compile_grpc_protos, error::{CompileError, CompileResult}};

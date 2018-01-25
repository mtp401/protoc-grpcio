// Copyright 2018. Matthew Pelland <matt@pelland.io>.
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
//
// Parts of this work are derived from the `protoc-rust-grpc` crate by
// Stepan Koltsov <stepan.koltsov@gmail.com>.
//
// Copyright 2016, Stepan Koltsov <stepan.koltsov@gmail.com>.
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
//! An API for programattically invoking the grpcio gRPC compiler in the same vein as the
//! [rust-protoc-grpc](https://crates.io/crates/protoc-rust-grpc) crate from Stepan Koltsov.

extern crate grpcio_compiler;

#[macro_use]
extern crate failure;

extern crate mktemp;

extern crate protobuf;
extern crate protoc;

use std::convert::AsRef;
use std::ffi::OsString;
use std::fs::File;
use std::io::{Read, Write};
use std::iter::Iterator;
use std::path::Path;
use std::vec::Vec;

use failure::ResultExt;

use mktemp::Temp;

use protobuf::{compiler_plugin, descriptor};
use protoc::{DescriptorSetOutArgs, Protoc};

/// Custom error type used throughout this crate.
pub type CompileError = ::failure::Error;
/// Custom result type used throughout this crate.
pub type CompileResult<T> = Result<T, CompileError>;

fn stringify_paths<P>(paths: &[P]) -> CompileResult<Vec<String>>
where
    P: AsRef<Path>
{
    paths
        .iter()
        .map(|input| match input.as_ref().to_str() {
            Some(s) => Ok(s.to_owned()),
            None => Err(format_err!(
                "failed to convert {:?} to string",
                input.as_ref()
            ))
        })
        .collect()
}

fn write_out_generated_files<P>(
    generation_results: Vec<compiler_plugin::GenResult>,
    output_dir: P
) -> CompileResult<()>
where
    P: AsRef<Path>
{
    for result in generation_results {
        let file = output_dir.as_ref().join(result.name);
        File::create(&file)
            .context(format!("failed to create {:?}", &file))?
            .write_all(&result.content)
            .context(format!("failed to write {:?}", &file))?;
    }

    Ok(())
}

/// Compiles a list a gRPC definitions to rust modules.
///
/// # Arguments
///
/// * `inputs` - A list of protobuf definitions to compile. Corresponds directly to what you'd
///     typically feed into `protoc`.
/// * `includes` - A list of of include directories to pass to `protoc`. Note that the directory
///     each member of `inputs` is in must be included in this parameter.
/// * `output` - Directory to place the generated rust modules into.
pub fn compile_grpc_protos<Input, Include, Output>(
    inputs: &[Input],
    includes: &[Include],
    output: Output
) -> CompileResult<()>
where
    Input: AsRef<Path>,
    Include: AsRef<Path>,
    Output: AsRef<Path>
{
    let protoc = Protoc::from_env_path();

    protoc
        .check()
        .context("failed to find `protoc`, `protoc` must be availabe in `PATH`")?;

    let stringified_inputs = stringify_paths(inputs)?;
    let stringified_includes = stringify_paths(includes)?;

    let descriptor_set = Temp::new_file()?;

    protoc
        .write_descriptor_set(DescriptorSetOutArgs {
            out: match descriptor_set.as_ref().to_str() {
                Some(s) => s,
                None => bail!("failed to convert descriptor set path to string")
            },
            input: stringified_inputs
                .iter()
                .map(String::as_str)
                .collect::<Vec<&str>>()
                .as_slice(),
            includes: stringified_includes
                .iter()
                .map(String::as_str)
                .collect::<Vec<&str>>()
                .as_slice(),
            include_imports: true
        })
        .context("failed to write descriptor set")?;

    let mut serialized_descriptor_set = Vec::new();
    File::open(&descriptor_set)
        .context("failed to open descriptor set")?
        .read_to_end(&mut serialized_descriptor_set)
        .context("failed to read descriptor set")?;

    let descriptor_set = protobuf::parse_from_bytes::<descriptor::FileDescriptorSet>(
        &serialized_descriptor_set
    ).context("failed to parse descriptor set")?;

    let files_to_generate = inputs
        .iter()
        .map(
            |input| match input.as_ref().file_name().map(OsString::from) {
                Some(i) => Ok(i.into_string()
                    .map_err(|e| format_err!("failed to convert {:?} to string", e))?),
                None => Err(format_err!(
                    "failed to find file name for {:?}",
                    input.as_ref()
                ))
            }
        )
        .collect::<CompileResult<Vec<String>>>()?;

    write_out_generated_files(
        grpcio_compiler::codegen::gen(descriptor_set.get_file(), files_to_generate.as_slice()),
        &output
    ).context("failed to write generated grpc definitions")?;

    write_out_generated_files(
        protobuf::codegen::gen(descriptor_set.get_file(), files_to_generate.as_slice()),
        &output
    ).context("failed to write out generated protobuf definitions")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_grpc_protos() {
        let temp_dir = Temp::new_dir().unwrap();
        let proto_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("test/assets/protos");

        compile_grpc_protos(
            vec![proto_dir.join("helloworld.proto")].as_slice(),
            vec![proto_dir].as_slice(),
            &temp_dir
        ).unwrap();

        assert!(temp_dir.as_ref().join("helloworld_grpc.rs").is_file());
        assert!(temp_dir.as_ref().join("helloworld.rs").is_file());
    }
}

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
//! An API for programmatically invoking the grpcio gRPC compiler in the same vein as the
//! [rust-protoc-grpc](https://crates.io/crates/protoc-rust-grpc) crate from Stepan Koltsov.

extern crate grpcio_compiler;

#[macro_use]
extern crate failure;

extern crate mktemp;

extern crate protobuf;
extern crate protoc;

use std::convert::AsRef;
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

fn stringify_paths<Paths>(paths: Paths) -> CompileResult<Vec<String>>
where
    Paths: IntoIterator,
    Paths::Item: AsRef<Path>
{
    paths
        .into_iter()
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
/// * `inputs` - A list of protobuf definitions to compile.
/// * `includes` - A list of of include directories to pass to `protoc`. Note that the directory
///     each member of `inputs` is in must be included in this parameter.
/// * `output` - Directory to place the generated rust modules into.
pub fn compile_grpc_protos<Inputs, Includes, Output>(
    inputs: Inputs,
    includes: Includes,
    output: Output
) -> CompileResult<()>
where
    Inputs: IntoIterator,
    Inputs::Item: AsRef<Path>,
    Includes: IntoIterator,
    Includes::Item: AsRef<Path>,
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

    let files_to_generate = stringify_paths(
        stringified_inputs
            .iter()
            .map(Path::new)
            .map(|input| {
                if input.is_absolute() {
                    input
                        .file_name()
                        .map(Path::new)
                        .unwrap_or_else(|| input)
                } else {
                    input
                }
            })
    )?;

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

    const PROTO_ASSETS_DIR: &'static str = "test/assets/protos";
    const EXPECTED_HELLO_WORLD_OUTPUTS: &[&'static str] = &["helloworld_grpc.rs", "helloworld.rs"];

    fn assert_compile_grpc_protos<Input, Output>(input: Input, expected_outputs: Output)
    where
        Input: AsRef<Path>,
        Output: IntoIterator + Copy,
        Output::Item: AsRef<Path>
    {
        let rel_include_path = Path::new(PROTO_ASSETS_DIR);
        let abs_include_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel_include_path);
        for include_path in &[rel_include_path, abs_include_path.as_ref()] {
            let temp_dir = Temp::new_dir().unwrap();
            compile_grpc_protos(&[&input], &[include_path], &temp_dir).unwrap();

            for output in expected_outputs {
                assert!(temp_dir.as_ref().join(output).is_file());
            }
        }
    }

    #[test]
    fn test_compile_grpc_protos() {
        assert_compile_grpc_protos("helloworld.proto", EXPECTED_HELLO_WORLD_OUTPUTS);
    }

    #[test]
    fn test_compile_grpc_protos_subdir() {
        assert_compile_grpc_protos("foo/bar/baz.proto", &["baz_grpc.rs", "baz.rs"])
    }

    #[test]
    fn test_compile_grpc_protos_absolute_path() {
        let temp_dir = Temp::new_dir().unwrap();
        let proto_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(PROTO_ASSETS_DIR);

        compile_grpc_protos(
            &[proto_dir.join("helloworld.proto")],
            &[proto_dir],
            &temp_dir
        ).unwrap();

        for output in EXPECTED_HELLO_WORLD_OUTPUTS {
            assert!(temp_dir.as_ref().join(output).is_file());
        }
    }
}

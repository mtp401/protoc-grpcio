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

use std::{
    convert::AsRef,
    fs::File,
    io::{Read as _, Write as _},
    iter::Iterator as _,
    path::{Path, PathBuf},
    vec::Vec
};

use protobuf::{compiler_plugin, descriptor, Message as _};
use protobuf_codegen::Customize;
use protoc::{DescriptorSetOutArgs, Protoc};
use tempfile::NamedTempFile;

mod error;
pub use crate::error::{CompileError, CompileResult};

fn stringify_paths<Paths>(paths: Paths) -> CompileResult<Vec<String>>
where
    Paths: IntoIterator,
    Paths::Item: AsRef<Path>,
{
    paths
        .into_iter()
        .map(|input| {
            input
                .as_ref()
                .to_str()
                .ok_or_else(|| {
                    CompileError::PathConversionError(format!(
                        "failed to convert {} to string",
                        input.as_ref().display()
                    ))
                })
                .map(Into::into)
        })
        .collect()
}

fn write_out_generated_files<P>(
    generation_results: Vec<compiler_plugin::GenResult>,
    output_dir: P,
) -> CompileResult<()>
where
    P: AsRef<Path>,
{
    for result in generation_results {
        let file = output_dir.as_ref().join(result.name);
        File::create(file)?.write_all(&result.content)?;
    }

    Ok(())
}

fn absolutize<P>(path: P) -> CompileResult<PathBuf>
where
    P: AsRef<Path>,
{
    let p = path.as_ref().to_path_buf();

    Ok(if p.is_relative() {
        std::env::current_dir()?.join(p)
    } else {
        p
    })
}

fn normalize<Paths, Bases>(
    paths: Paths,
    bases: Bases,
) -> CompileResult<(Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>)>
where
    Paths: IntoIterator,
    Paths::Item: AsRef<Path>,
    Bases: IntoIterator,
    Bases::Item: AsRef<Path>,
{
    let absolutized_bases = bases
        .into_iter()
        .map(absolutize)
        .collect::<CompileResult<Vec<PathBuf>>>()?;

    // We deal with the following cases:
    // a.) absolute paths
    // b.) paths relative to CWD
    // c.) paths relative to bases
    //
    // We take the strategy of transforming the relative path cases (b & c) into absolute paths (a)
    // and use the strip_prefix API from there.
    let absolutized_paths = paths
        .into_iter()
        .map(|p| {
            let rel_path = p.as_ref().to_path_buf();
            absolutize(&rel_path).map(|abs_path| (rel_path, abs_path))
        })
        .flatten()
        .map(|(rel_path, abs_path)| {
            if abs_path.exists() {
                // Case a or b.
                Ok(abs_path)
            } else {
                // Case c.
                for b in &absolutized_bases {
                    let absolutized_path = b.join(&rel_path);
                    if absolutized_path.exists() {
                        return Ok(absolutized_path);
                    }
                }
                Err(CompileError::PathConversionError(format!(
                    "Failed to find the absolute path of input {}",
                    rel_path.display()
                )))
            }
        })
        .collect::<CompileResult<Vec<PathBuf>>>()?;

    let relativized_paths: Vec<PathBuf> = absolutized_paths
        .iter()
        .map(|p| {
            for b in &absolutized_bases {
                if let Ok(rel_path) = p.strip_prefix(&b) {
                    return Ok(rel_path.to_path_buf())
                }
            }
            Err(CompileError::PathConversionError(format!(
                "The input path {} is not contained by any of the include paths {:?}",
                p.display(),
                absolutized_bases
            )))
        })
        .collect::<CompileResult<Vec<PathBuf>>>()?;

    Ok((absolutized_bases, absolutized_paths, relativized_paths))
}

/// Compiles a list a gRPC definitions to rust modules.
///
/// # Arguments
///
/// * `inputs` - A list of protobuf definition paths to compile. Paths can be specified as absolute,
///    relative to the CWD or relative to one of the `includes` paths. Note that the directory each
///    member of `inputs` is found under must be included in the `includes` parameter.
/// * `includes` - A list of of include directory paths to pass to `protoc`. Include paths can be
///    specified either as absolute or relative to the CWD. Note that the directory each member of
///    `inputs` is found under must be included in this parameter.
/// * `output` - Directory to place the generated rust modules into.
/// * `customizations` - An Option<protobuf_codegen::Customize> allowing customization options to be
///    passed to protobuf_codegen
pub fn compile_grpc_protos<Inputs, Includes, Output>(
    inputs: Inputs,
    includes: Includes,
    output: Output,
    customizations: Option<Customize>,
) -> CompileResult<()>
where
    Inputs: IntoIterator,
    Inputs::Item: AsRef<Path>,
    Includes: IntoIterator,
    Includes::Item: AsRef<Path>,
    Output: AsRef<Path>,
{
    let protoc = Protoc::from_env_path();

    protoc.check()?;

    let (absolutized_includes, absolutized_paths, relativized_inputs) =
        normalize(inputs, includes)?;
    let stringified_inputs_absolute = stringify_paths(absolutized_paths)?;
    let stringified_inputs = stringify_paths(relativized_inputs)?;
    let stringified_includes = stringify_paths(absolutized_includes)?;

    let descriptor_set = NamedTempFile::new()?;

    protoc
        .write_descriptor_set(DescriptorSetOutArgs {
            out: match descriptor_set.as_ref().to_str() {
                Some(s) => s,
                None => return Err(CompileError::GenericError(
                    "failed to convert descriptor set path to string".to_owned()
                ))
            },
            input: stringified_inputs_absolute
                .iter()
                .map(String::as_str)
                .collect::<Vec<&str>>()
                .as_slice(),
            includes: stringified_includes
                .iter()
                .map(String::as_str)
                .collect::<Vec<&str>>()
                .as_slice(),
            include_imports: true,
        })?;

    let mut serialized_descriptor_set = Vec::new();
    File::open(&descriptor_set)?
        .read_to_end(&mut serialized_descriptor_set)?;

    let descriptor_set =
        descriptor::FileDescriptorSet::parse_from_bytes(&serialized_descriptor_set)?;

    let customize = customizations.unwrap_or_default();

    write_out_generated_files(
        grpcio_compiler::codegen::gen(descriptor_set.get_file(), stringified_inputs.as_slice()),
        &output,
    )?;

    write_out_generated_files(
        protobuf_codegen::gen(
            descriptor_set.get_file(),
            stringified_inputs.as_slice(),
            &customize,
        ),
        &output,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn assert_compile_grpc_protos<Input, Output>(input: Input, expected_outputs: Output)
    where
        Input: AsRef<Path>,
        Output: IntoIterator + Copy,
        Output::Item: AsRef<Path>,
    {
        let rel_include_path = PathBuf::from("test/assets/protos");
        let abs_include_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(&rel_include_path);
        for include_path in &[&rel_include_path, &abs_include_path] {
            for inputs in &[vec![input.as_ref()], vec![&include_path.join(&input)]] {
                let temp_dir = tempdir().unwrap();
                compile_grpc_protos(inputs, &[include_path], &temp_dir, None).unwrap();

                for output in expected_outputs {
                    assert!(temp_dir.as_ref().join(output).is_file());
                }
            }
        }
    }

    #[test]
    fn test_compile_grpc_protos() {
        assert_compile_grpc_protos("helloworld.proto", &["helloworld_grpc.rs", "helloworld.rs"])
    }

    #[test]
    fn test_compile_grpc_protos_subdir() {
        assert_compile_grpc_protos("foo/bar/baz.proto", &["baz_grpc.rs", "baz.rs"])
    }
}

// Copyright 2021. Matthew Pelland <matt@pelland.io>.
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

/// Custom result type used throughout this crate.
pub type CompileResult<T> = Result<T, CompileError>;

/// An enum capturing all of the different error types returned by this crate.
#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    /// Indicates a generic error occured.
    #[error("Generic error: {0}")]
    GenericError(String),
    /// Indicates an IO error occurred.
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    /// Due to the prototypes for some of the crates used internally paths must be converted to and
    /// from strings as well as absolute paths. This error variant is returned when one of the
    /// internal conversion routines fails.
    #[error("Path conversion error: {0}")]
    PathConversionError(String),
    /// Wraps an error returned by the `protobuf` crate.
    #[error(transparent)]
    ProtobufError(#[from] protobuf::ProtobufError)
}

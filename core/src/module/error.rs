// Copyright (c) 2025, BlockProject 3D
//
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without modification,
// are permitted provided that the following conditions are met:
//
//     * Redistributions of source code must retain the above copyright notice,
//       this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above copyright notice,
//       this list of conditions and the following disclaimer in the documentation
//       and/or other materials provided with the distribution.
//     * Neither the name of BlockProject 3D nor the names of its contributors
//       may be used to endorse or promote products derived from this software
//       without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
// EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
// PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::str::Utf8Error;

/// Type of error when using modules.
pub enum Error {
    /// The module was not found (argument: module name).
    NotFound(String),

    /// An unexpected NULL character was found.
    Null,

    /// Missing DEPS metadata key for a Rust based module.
    MissingDepsForRust,

    /// Missing RUSTC version key for a Rust based module.
    MissingVersionForRust,

    /// The given string was not UTF8.
    InvalidUtf8(Utf8Error),

    /// The RUSTC version in the module metadata does not match the RUSTC version used to build
    /// this [ModuleLoader](super::ModuleLoader).
    RustcVersionMismatch {
        /// The expected RUSTC version.
        expected: &'static str,

        /// The RUSTC version stored in the module which failed to load.
        actual: String
    },

    /// Invalid format for the DEPS metadata key.
    InvalidDepFormat,

    /// Incompatible dependency API found.
    IncompatibleDep {
        /// The name of the dependency which is incompatible.
        name: String,

        /// The version of the dependency imported by the module which failed to load.
        actual_version: String,

        /// The version of the dependency used by this [ModuleLoader](super::ModuleLoader).
        expected_version: String
    },

    /// An IO error.
    Io(std::io::Error),

    /// The module does not contain a valid metadata string.
    MissingMetadata,

    /// The metadata stored in the module has an invalid format.
    InvalidMetadata
}

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

//! This module describes possible errors when attempting to load modules.

use bp3d_util::simple_error;
use std::fmt::{Display, Formatter};
use std::str::Utf8Error;

#[derive(Debug, Clone, Eq, PartialEq)]
/// Describes an incompatible RUSTC version when attempting to load Rust based modules.
pub struct IncompatibleRustc {
    /// The expected RUSTC version.
    pub expected: &'static str,

    /// The RUSTC version stored in the module which failed to load.
    pub actual: String,
}

impl Display for IncompatibleRustc {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "expected version {}, got version {}",
            self.expected, self.actual
        )
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
/// Describes an incompatible dependency (a dependency which may produce ABI issues) when attempting
/// to load Rust based modules.
pub struct IncompatibleDependency {
    /// The name of the dependency which is incompatible.
    pub name: String,

    /// The version of the dependency imported by the module which failed to load.
    pub actual_version: String,

    /// The version of the dependency used by this [ModuleLoader](super::ModuleLoader).
    pub expected_version: String,
}

impl Display for IncompatibleDependency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "expected version {}, got version {} for dependency '{}'",
            self.expected_version, self.actual_version, self.name
        )
    }
}

simple_error! {
    /// Type of error when using modules.
    pub Error {
        /// The module was not found (argument: module name).
        NotFound(String) => "module not found ({})",

        /// An unexpected NULL character was found.
        Null => "unexpected null character in string",

        /// Missing DEPS metadata key for a Rust based module.
        MissingDepsForRust => "missing DEPS metadata key for a Rust module",

        /// Missing FEATURES metadata key for a Rust based module.
        MissingFeaturesForRust => "missing FEATURES metadata key for a Rust module",

        /// Missing RUSTC version key for a Rust based module.
        MissingVersionForRust => "missing RUSTC metadata key for a Rust module",

        /// Missing NAME key for a module.
        MissingModuleName => "missing NAME metadata key",

        /// Missing VERSION key for a module.
        MissingModuleVersion => "missing VERSION metadata key",

        /// The given string was not UTF8.
        InvalidUtf8(Utf8Error) => "invalid utf8: {}",

        /// The RUSTC version in the module metadata does not match the RUSTC version used to build
        /// this [ModuleLoader](super::ModuleLoader).
        RustcVersionMismatch(IncompatibleRustc) => "incompatible rustc version: {}",

        /// Invalid format for the DEPS metadata key.
        InvalidDepFormat => "invalid dependency format",

        /// Incompatible dependency API found.
        IncompatibleDep(IncompatibleDependency) => "incompatible dependency: {}",

        /// Unmatched feature-set.
        IncompatibleFeatureSet(String) => "incompatible feature-set for dependency: {}",

        /// An IO error.
        Io(std::io::Error) => "io error: {}",

        /// The module does not contain a valid metadata string.
        MissingMetadata => "missing module metadata",

        /// The metadata stored in the module has an invalid format.
        InvalidMetadata => "invalid module metadata format"
    }
}

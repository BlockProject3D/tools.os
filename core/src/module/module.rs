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

use crate::module::library::Library;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

/// This represents a module shared object.
#[derive(Debug)]
pub struct Module<L> {
    lib: L,
    metadata: HashMap<String, String>,
}

impl<L> Display for Module<L> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.metadata.fmt(f)
    }
}

impl<L: Library> Module<L> {
    /// Constructs a new [Module] from an existing [Library] handle.
    ///
    /// # Arguments
    ///
    /// * `lib`: the library to wrap.
    /// * `metadata`: module metadata.
    ///
    /// returns: Module
    pub fn new(lib: L, metadata: HashMap<String, String>) -> Self {
        Module { lib, metadata }
    }

    /// Gets a metadata key by its name.
    ///
    /// # Arguments
    ///
    /// * `key`: the key to read from the metadata (ex: NAME).
    ///
    /// returns: Option<&str>
    pub fn get_metadata_key(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }

    /// Returns the library attached to this module.
    pub fn lib(&self) -> &L {
        &self.lib
    }
}

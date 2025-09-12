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

//! Utilities around module metadata.

use crate::module::error::Error;
use std::collections::HashMap;

/// The metadata map type.
pub type Metadata = HashMap<String, Value>;

/// A wrapper for a metadata value with utility methods.
#[derive(Debug, Eq, PartialEq)]
pub struct Value(String);

impl Value {
    /// Creates a new instance of metadata value.
    pub fn new(value: String) -> Value {
        Value(value)
    }

    /// Returns the underlying string content.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the len of this [Value].
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if this [Value] is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Assumes this [Value] is a list and return an iterator over it.
    pub fn as_list(&self) -> Option<impl Iterator<Item = &str>> {
        // Amazingly broken split function that cannot figure out that empty strings should be
        // ignored...
        if self.0.is_empty() {
            None
        } else {
            Some(self.as_str().split(','))
        }
    }

    /// Assumes this [Value] is a list of key value pairs and return an iterator over it.
    pub fn parse_key_value_pairs(
        &self,
    ) -> Option<impl Iterator<Item = super::Result<(String, &str)>>> {
        Some(self.as_list()?.map(|item| {
            let mut iter = item.split("=");
            let name = iter
                .next()
                .ok_or(Error::InvalidDepFormat)?
                .replace("-", "_");
            let version = iter.next().ok_or(Error::InvalidDepFormat)?;
            Ok((name, version))
        }))
    }
}

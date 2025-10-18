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

use crate::module::error::Error;
use crate::module::library::symbol::Symbol;
use crate::module::library::Library;
use std::ffi::c_void;

/// This represents a virtual library to be used in full statically-linked applications.
#[derive(Copy, Clone)]
pub struct VirtualLibrary {
    name: &'static str,
    symbols: &'static [(&'static str, *const c_void)],
}

unsafe impl Send for VirtualLibrary {}

unsafe impl Sync for VirtualLibrary {}

impl VirtualLibrary {
    /// Creates a new [VirtualLibrary] from a name and an array of symbols.
    ///
    /// This function is const to allow statics declaration from build tool.
    pub const fn new(
        name: &'static str,
        symbols: &'static [(&'static str, *const c_void)],
    ) -> Self {
        Self { name, symbols }
    }

    /// Returns the name of this [VirtualLibrary].
    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl Library for VirtualLibrary {
    unsafe fn load_symbol<T>(
        &self,
        name: impl AsRef<str>,
    ) -> crate::module::Result<Option<Symbol<'_, T>>> {
        if name.as_ref().find('\0').is_some() {
            return Err(Error::Null);
        }
        for (name1, symbol) in self.symbols {
            if *name1 == name.as_ref() {
                return Ok(Some(Symbol::from_raw(*symbol)));
            }
        }
        Ok(None)
    }
}

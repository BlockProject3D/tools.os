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

//! This module contains various library types.

mod symbol;
pub mod types;
#[cfg(unix)]
mod unix;
mod r#virtual;
#[cfg(windows)]
mod windows;

/// The extension of a module.
#[cfg(unix)]
pub const OS_EXT: &str = unix::EXT;

/// The extension of a module.
#[cfg(windows)]
pub const OS_EXT: &str = windows::EXT;

/// Represents a library.
pub trait Library {
    /// Attempts to load the given symbol from this library.
    ///
    /// # Arguments
    ///
    /// * `name`: the name of the symbol.
    ///
    /// returns: Result<Symbol<T>, Error>
    ///
    /// # Safety
    ///
    /// This function assumes the returned symbol is of the correct type and does not use any ABI
    /// incompatible types. If this condition is not maintained then this function is UB.
    unsafe fn load_symbol<T>(
        &self,
        name: impl AsRef<str>,
    ) -> crate::module::Result<Option<types::Symbol<T>>>;
}

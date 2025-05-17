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

use std::ffi::{c_void, CString};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use libc::{dlclose, dlopen, dlsym, RTLD_LAZY};
use crate::module::Error;

/// The extension of a module.
#[cfg(target_vendor = "apple")]
pub const MODULE_EXT: &str = "dylib";

/// The extension of a module.
#[cfg(all(unix, not(target_vendor = "apple")))]
pub const MODULE_EXT: &str = "so";

/// This represents a symbol from a module.
pub struct Symbol<T>(*const T);

impl<T> Symbol<T> {
    /// Returns the raw pointer of this symbol.
    pub fn as_ptr(&self) -> *const T {
        self.0
    }

    /// Returns a reference to this symbol.
    pub fn as_ref(&self) -> &T {
        unsafe { &*self.0 }
    }
}

/// This represents a module shared object.
pub struct Module(*mut c_void);

impl Module {
    /// Loads a module from the given path.
    ///
    /// # Arguments
    ///
    /// * `path`: full path to the shared library including extension.
    ///
    /// returns: Result<Module, Error>
    ///
    /// # Safety
    ///
    /// This function is unsafe as it assumes the module to be loaded is trusted code. If the module
    /// contains any constructor which causes UB then this function causes UB.
    pub unsafe fn load(path: impl AsRef<Path>) -> super::Result<Self> {
        let path = CString::new(path.as_ref().as_os_str().as_bytes()).map_err(|_| Error::Null)?;
        Ok(Module(dlopen(path.as_ptr(), RTLD_LAZY)))
    }

    /// Attempts to load the given symbol from this module.
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
    pub unsafe fn load_symbol<T>(&self, name: impl AsRef<str>) -> super::Result<Option<Symbol<T>>> {
        let name = CString::new(name.as_ref().as_bytes()).map_err(|_| Error::Null)?;
        let sym = dlsym(self.0, name.as_ptr());
        if sym.is_null() {
            Ok(None)
        } else {
            Ok(Some(Symbol(sym as *const T)))
        }
    }

    /// Unloads the current module.
    ///
    /// # Safety
    ///
    /// This function assumes no Symbols from this module are currently in scope, if not this
    /// function is UB.
    pub unsafe fn unload(self) {
        dlclose(self.0);
    }
}

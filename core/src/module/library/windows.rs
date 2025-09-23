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

use crate::module;
use crate::module::error::Error;
use crate::module::library::symbol::Symbol;
use std::ffi::CString;
use std::fmt::Debug;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows_sys::Win32::Foundation::{FreeLibrary, HMODULE};
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress, LoadLibraryW};

pub const EXT: &str = "dll";

/// This represents a module shared object.
#[derive(Debug)]
pub struct Library(HMODULE);

unsafe impl Send for Library {}

impl Library {
    /// Attempts to open a handle to the current running program.
    pub fn open_self() -> module::Result<Self> {
        let handle = unsafe { GetModuleHandleW(std::ptr::null()) };
        if handle.is_null() {
            return Err(Error::Io(std::io::Error::last_os_error()));
        }
        Ok(Library(handle))
    }

    /// Loads a dynamic library from the given path.
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
    /// contains any constructor which causes UB then this function causes UB. Additionally, it is
    /// UB to load a module with a DllMain function inside, if you absolutely need a DllMain function
    /// use `bp3d_os_module_<name>_open` and `bp3d_os_module_<name>_close`.
    pub unsafe fn load(path: impl AsRef<Path>) -> module::Result<Self> {
        let mut path = path.as_ref().as_os_str().encode_wide().collect::<Vec<_>>();
        if path.iter().any(|v| *v == 0x0) {
            return Err(Error::Null);
        }
        path.push(0);
        let handle = LoadLibraryW(path.as_ptr());
        if handle.is_null() {
            return Err(Error::Io(std::io::Error::last_os_error()));
        }
        Ok(Library(handle))
    }
}

impl super::Library for Library {
    unsafe fn load_symbol<T>(
        &self,
        name: impl AsRef<str>,
    ) -> module::Result<Option<Symbol<'_, T>>> {
        let name = CString::new(name.as_ref().as_bytes()).map_err(|_| Error::Null)?;
        let sym = GetProcAddress(self.0, name.as_ptr() as _);
        if sym.is_none() {
            Ok(None)
        } else {
            Ok(Some(Symbol::from_raw(std::mem::transmute(sym))))
        }
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        unsafe { FreeLibrary(self.0) };
    }
}

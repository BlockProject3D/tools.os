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

use std::collections::HashMap;
use std::ffi::{c_char, CStr};
use std::path::{Path, PathBuf};
use crate::module::{Error, MODULE_EXT, RUSTC_VERSION_C};
use crate::module::unix::Module;

/// Represents a module loader which can support loading multiple related modules.
pub struct ModuleLoader {
    paths: Vec<PathBuf>,
    modules: HashMap<String, Module>,
    deps: HashMap<String, String>
}

unsafe fn load_lib(deps2: &mut HashMap<String, String>, name: &str, path: &Path) -> super::Result<Module> {
    let module = Module::load(path)?;
    // This symbol is optional and will not exist on C/C++ modules, only on Rust based modules.
    // The main reason the rustc version is checked on Rust modules is for interop with user
    // data types declared by other modules as well as the destructor system which isn't C/C++
    // compatible.
    let rustc_const = format!("BP3D_MODULE_{}_RUSTC_VERSION", name.to_uppercase());
    if let Some(rustc_version) = module.load_symbol::<c_char>(rustc_const)? {
        // This is the list of dependencies of the module to be loaded.
        // This is optional for C/C++ modules but required for rust modules.
        // In rust modules this is used to ensure the module to be loaded does not present an
        // incompatible ABI with another module.
        let deps_const = format!("BP3D_MODULE_{}_DEPS", name.to_uppercase());
        let deps = module.load_symbol::<c_char>(deps_const)?.ok_or(Error::MissingDepsForRust)?;
        let rustc_version = CStr::from_ptr(rustc_version.as_ptr());
        let deps = std::str::from_utf8(CStr::from_ptr(deps.as_ptr()).to_bytes())
            .map_err(Error::InvalidUtf8)?;
        if rustc_version != RUSTC_VERSION_C {
            //mismatch between rust versions
            return Err(Error::RustcVersionMismatch {
                expected: RUSTC_VERSION_C,
                actual: rustc_version.into()
            });
        }
        for dep in deps.split(",") {
            let mut iter = dep.split("=");
            let name = iter.next().ok_or(Error::InvalidDepFormat)?;
            let version = iter.next().ok_or(Error::InvalidDepFormat)?;
            if let Some(expected_version) = deps2.get(name) {
                if version != expected_version {
                    return Err(Error::IncompatibleDep {
                        name: name.into(),
                        expected_version: expected_version.into(),
                        actual_version: version.into()
                    });
                }
            } else {
                deps2.insert(name.into(), version.into());
            }
        }
    }
    Ok(module)
}

impl ModuleLoader {
    /// Attempts to load a module from the specified path and name.
    ///
    /// This function already does check for the version of rustc and dependencies for Rust based
    /// modules to ensure maximum ABI compatibility.
    ///
    /// This function assumes the code to be loaded is trusted and delegates this operation to the
    /// underlying OS.
    ///
    /// # Arguments
    ///
    /// * `name`: the name of the module to be loaded.
    ///
    /// returns: ()
    ///
    /// # Safety
    ///
    /// It is assumed that the module is intended to be used with this instance of [ModuleLoader];
    /// if not, this function is UB. Additionally, if some dependency used in public facing APIs
    /// for the module are not added with [add_public_dependency](Self::add_public_dependency),
    /// this is also UB.
    pub unsafe fn load(&mut self, name: &str) -> super::Result<&Module> {
        if self.modules.contains_key(name) {
            Ok(self.modules.get(name).unwrap_unchecked())
        } else {
            let name = format!("{}.{}", name, MODULE_EXT);
            let name2 = format!("lib{}.{}", name, MODULE_EXT);
            for path in self.paths.iter() {
                let search = path.join(&name);
                let search2 = path.join(&name2);
                let mut module = None;
                if search.exists() {
                    module = Some(load_lib(&mut self.deps, &name, &search)?);
                } else if search2.exists() {
                    module = Some(load_lib(&mut self.deps, &name, &search2)?);
                }
                if let Some(module) = module {
                    self.modules.insert(name.clone(), module);
                    return Ok(&self.modules[&name]);
                }
            }
            Err(Error::NotFound(name.into()))
        }
    }

    /// Attempts to unload the given module.
    ///
    /// # Arguments
    ///
    /// * `name`: the name of the module to unload.
    ///
    /// returns: ()
    ///
    /// # Safety
    ///
    /// This function assumes no Symbols from this module are currently in scope, if not this
    /// function is UB.
    pub unsafe fn unload(&mut self, name: &str) -> super::Result<()> {
        let module = self.modules.remove(name).ok_or(Error::NotFound(name.into()))?;
        module.unload();
        Ok(())
    }

    /// Adds the given path to the path search list.
    ///
    /// # Arguments
    ///
    /// * `path`: the path to include.
    ///
    /// returns: ()
    pub fn add_search_path(&mut self, path: impl AsRef<Path>) {
        self.paths.push(path.as_ref().into());
    }

    /// Adds a public facing API dependency to the list of dependency for version checks.
    ///
    /// This is used to check if there are any ABI incompatibilities between dependency versions
    /// when loading a Rust based module.
    ///
    /// # Arguments
    ///
    /// * `name`: the name of the dependency.
    /// * `version`: the version of the dependency.
    ///
    /// returns: ()
    pub fn add_public_dependency(&mut self, name: &str, version: &str) {
        self.deps.insert(name.into(), version.into());
    }
}

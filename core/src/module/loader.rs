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

use crate::module::error::{Error, IncompatibleDependency, IncompatibleRustc};
use crate::module::Module;
use crate::module::RUSTC_VERSION;
use std::collections::HashMap;
use std::ffi::{c_char, CStr};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use crate::module::library::{Library, OS_EXT};
use crate::module::library::types::{OsLibrary, VirtualLibrary};

/// Represents a module loader which can support loading multiple related modules.
pub struct ModuleLoader {
    paths: Vec<PathBuf>,
    modules: HashMap<String, Module<OsLibrary>>,
    builtin_modules: HashMap<String, Module<VirtualLibrary>>,
    deps: HashMap<String, String>,
    builtins: &'static [&'static VirtualLibrary]
}

const MOD_HEADER: &[u8] = b"BP3D_OS_MODULE|";

fn parse_metadata(bytes: &[u8]) -> super::Result<HashMap<String, String>> {
    // Remove terminator NULL.
    let bytes = &bytes[..bytes.len() - 1];
    let mut map = HashMap::new();
    let data = std::str::from_utf8(bytes).map_err(Error::InvalidUtf8)?;
    let mut vars = data.split("|");
    vars.next();
    for var in vars {
        let pos = var.find('=').ok_or(Error::InvalidMetadata)?;
        let key = &var[..pos];
        let value = &var[pos + 1..];
        map.insert(key.into(), value.into());
    }
    Ok(map)
}

fn load_metadata(path: &Path) -> super::Result<HashMap<String, String>> {
    let mut file = File::open(path).map_err(Error::Io)?;
    let mut buffer: [u8; 8192] = [0; 8192];
    let mut v = Vec::new();
    while file.read(&mut buffer).map_err(Error::Io)? > 0 {
        let mut slice = &buffer[..];
        while let Some(pos) = slice.iter().position(|v| *v == b'B') {
            let inner = &slice[pos..];
            let end = inner.iter().position(|v| *v == 0).unwrap_or(inner.len());
            v.extend_from_slice(&inner[..end + 1]);
            if v[v.len() - 1] == 0 {
                if v.starts_with(MOD_HEADER) {
                    // We found the module metadata.
                    return parse_metadata(&v);
                }
                v.clear();
                slice = &inner[end + 1..];
            } else {
                break;
            }
        }
    }
    Err(Error::MissingMetadata)
}

fn check_metadata(metadata: &HashMap<String, String>, deps2: &mut HashMap<String, String>) -> super::Result<()> {
    if metadata.get("TYPE").ok_or(Error::InvalidMetadata)? == "RUST" {
        // This symbol is optional and will not exist on C/C++ modules, only on Rust based modules.
        // The main reason the rustc version is checked on Rust modules is for interop with user
        // data types declared by other modules as well as the destructor system which isn't C/C++
        // compatible.
        let rustc_version = metadata.get("RUSTC").ok_or(Error::MissingVersionForRust)?;
        // This is the list of dependencies of the module to be loaded.
        // This is optional for C/C++ modules but required for rust modules.
        // In rust modules this is used to ensure the module to be loaded does not present an
        // incompatible ABI with another module.
        let deps = metadata.get("DEPS").ok_or(Error::MissingDepsForRust)?;
        if rustc_version != RUSTC_VERSION {
            //mismatch between rust versions
            return Err(Error::RustcVersionMismatch(IncompatibleRustc {
                expected: RUSTC_VERSION,
                actual: rustc_version.into(),
            }));
        }
        // Amazingly broken split function that cannot figure out that empty strings should be
        // ignored...
        if !deps.is_empty() {
            for dep in deps.split(",") {
                let mut iter = dep.split("=");
                let name = iter.next().ok_or(Error::InvalidDepFormat)?.replace("-", "_");
                let version = iter.next().ok_or(Error::InvalidDepFormat)?;
                if let Some(expected_version) = deps2.get(&name) {
                    if version != expected_version {
                        return Err(Error::IncompatibleDep(IncompatibleDependency {
                            name,
                            expected_version: expected_version.into(),
                            actual_version: version.into(),
                        }));
                    }
                } else {
                    deps2.insert(name, version.into());
                }
            }
        }
    }
    Ok(())
}

unsafe fn load_lib(
    deps2: &mut HashMap<String, String>,
    name: &str,
    path: &Path,
) -> super::Result<Module<OsLibrary>> {
    let metadata = load_metadata(path)?;
    check_metadata(&metadata, deps2)?;
    let module = Module::new(OsLibrary::load(path)?, metadata);
    module_open(name, module.lib())?;
    Ok(module)
}

unsafe fn module_open<L: Library>(name: &str, lib: &L) -> super::Result<()> {
    let main_name = format!("bp3d_os_module_{}_open", name);
    if let Some(main) = lib.load_symbol::<extern "C" fn()>(main_name)? {
        main.call();
    }
    Ok(())
}

unsafe fn load_by_symbol<L: Library>(lib: L, name: &str, deps: &mut HashMap<String, String>) -> super::Result<Module<L>> {
    let mod_const_name = format!("BP3D_OS_MODULE_{}", name.to_uppercase());
    if let Some(sym) = lib.load_symbol::<*const c_char>(mod_const_name)? {
        let bytes = CStr::from_ptr((*sym.as_ptr()).offset(1)).to_bytes_with_nul();
        let metadata = parse_metadata(bytes)?;
        check_metadata(&metadata, deps)?;
        let module = Module::new(lib, metadata);
        module_open(name, module.lib())?;
        return Ok(module);
    }
    Err(Error::NotFound(name.into()))
}

impl Default for ModuleLoader {
    fn default() -> Self {
        let mut this = ModuleLoader {
            paths: Default::default(),
            modules: Default::default(),
            deps: Default::default(),
            builtin_modules: Default::default(),
            builtins: &[]
        };
        this.add_public_dependency(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        this
    }
}

impl ModuleLoader {
    /// Create a new instance of a [ModuleLoader].
    // Apparently clippy prefers code duplication, well I said no...
    #[allow(clippy::field_reassign_with_default)]
    pub fn new(builtins: &'static [&'static VirtualLibrary]) -> ModuleLoader {
        let mut def = Self::default();
        def.builtins = builtins;
        def
    }

    /// Attempts to load the given builtin module from its name.
    ///
    /// # Arguments
    ///
    /// * `name`: the name of the builtin module to load.
    ///
    /// returns: Result<&Module<VirtualLibrary>, Error>
    ///
    /// # Safety
    ///
    /// This function assumes the module to be loaded, if it exists has the correct format otherwise
    /// this function is UB.
    pub unsafe fn load_builtin(&mut self, name: &str) -> super::Result<&Module<VirtualLibrary>> {
        let name = name.replace("-", "_");
        if self.builtin_modules.contains_key(&name) {
            Ok(unsafe { self.builtin_modules.get(&name).unwrap_unchecked() })
        } else {
            for builtin in self.builtins {
                if builtin.name() == name {
                    let module = unsafe { load_by_symbol(**builtin, &name, &mut self.deps) }
                        .map_err(|e| match e {
                            Error::NotFound(_) => Error::MissingMetadata,
                            e => e
                        })?;
                    return Ok(self.builtin_modules.entry(name).or_insert(module));
                }
            }
            Err(Error::NotFound(name))
        }
    }

    /// Attempts to load a module from the specified name which is dynamically linked in the current
    /// running software.
    ///
    /// # Arguments
    ///
    /// * `name`: the name of the module to be loaded.
    ///
    /// returns: Result<&Module, Error>
    ///
    /// # Safety
    ///
    /// This function assumes the module to be loaded, if it exists has the correct format otherwise
    /// this function is UB.
    pub unsafe fn load_self(&mut self, name: &str) -> super::Result<&Module<OsLibrary>> {
        let name = name.replace("-", "_");
        if self.modules.contains_key(&name) {
            unsafe { Ok(self.modules.get(&name).unwrap_unchecked()) }
        } else {
            let this = OsLibrary::open_self()?;
            let module = unsafe { load_by_symbol(this, &name, &mut self.deps) }?;
            Ok(self.modules.entry(name).or_insert(module))
        }
    }

    /// Attempts to load a module from the specified name.
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
    pub unsafe fn load(&mut self, name: &str) -> super::Result<&Module<OsLibrary>> {
        let name = name.replace("-", "_");
        if self.modules.contains_key(&name) {
            Ok(self.modules.get(&name).unwrap_unchecked())
        } else {
            let name2 = format!("{}.{}", name, OS_EXT);
            let name3 = format!("lib{}.{}", name, OS_EXT);
            for path in self.paths.iter() {
                let search = path.join(&name2);
                let search2 = path.join(&name3);
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
            Err(Error::NotFound(name))
        }
    }

    /// Attempts to unload the given module.
    ///
    /// # Arguments
    ///
    /// * `name`: the name of the module to unload.
    ///
    /// returns: ()
    pub fn unload(&mut self, name: &str) -> super::Result<()> {
        let name = name.replace("-", "_");
        if self.modules.contains_key(&name) {
            let module = unsafe { self.modules.remove(&name).unwrap_unchecked() };
            let main_name = format!("bp3d_os_module_{}_close", &name);
            if let Some(main) = unsafe { module.lib().load_symbol::<extern "C" fn()>(main_name)? } {
                main.call();
            }
            drop(module);
        } else {
            let module = self
                .builtin_modules
                .remove(&name)
                .ok_or_else(|| Error::NotFound(name.clone()))?;
            let main_name = format!("bp3d_os_module_{}_close", &name);
            if let Some(main) = unsafe { module.lib().load_symbol::<extern "C" fn()>(main_name)? } {
                main.call();
            }
        }
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
        self.deps.insert(name.replace("-", "_"), version.into());
    }
}

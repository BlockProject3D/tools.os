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
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};
use bp3d_debug::debug;
use crate::module::error::Error;
use crate::module::library::{Library, OS_EXT};
use crate::module::library::types::{OsLibrary, VirtualLibrary};
use crate::module::loader::Lock;
use crate::module::loader::util::{load_by_symbol, load_lib, Dependency, DepsMap};
use crate::module::Module;

static MODULE_LOADER: OnceLock<Mutex<ModuleLoader>> = OnceLock::new();

/// Represents a module loader which can support loading multiple related modules.
pub struct ModuleLoader {
    paths: Vec<PathBuf>,
    pub(super) modules: HashMap<usize, Module<OsLibrary>>,
    pub(super) builtin_modules: HashMap<usize, Module<VirtualLibrary>>,
    deps: DepsMap,
    builtins: &'static [&'static VirtualLibrary],
    module_name_to_id: HashMap<String, usize>,
    last_module_id: usize
}

impl ModuleLoader {
    /// Create a new instance of a [ModuleLoader] and installs it as this application's
    /// [ModuleLoader].
    pub fn install(builtins: &'static [&'static VirtualLibrary]) {
        let mut this = ModuleLoader {
            paths: Default::default(),
            modules: Default::default(),
            deps: DepsMap::new(),
            builtin_modules: Default::default(),
            builtins,
            module_name_to_id: Default::default(),
            last_module_id: 0
        };
        this._add_public_dependency(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"), ["*"]);
        this._add_public_dependency("bp3d-debug", "1.0.0", ["*"]);
        if let Err(_) = MODULE_LOADER.set(Mutex::new(this)) {
            panic!("attempt to initialize module loader twice");
        }
    }

    /// Installs a default [ModuleLoader] for this application.
    pub fn install_default() {
        Self::install(&[]);
    }

    fn _lock<'a>() -> MutexGuard<'a, ModuleLoader> {
        if MODULE_LOADER.get().is_none() {
            Self::install_default();
        }
        MODULE_LOADER.get().unwrap().lock().unwrap()
    }

    fn _next_module_id(&mut self) -> usize {
        let id = self.last_module_id;
        self.last_module_id += 1;
        id
    }

    pub(super) fn _get_builtin(&self, name: &str) -> Option<usize> {
        let name = name.replace("-", "_");
        if let Some(id) = self.module_name_to_id.get(&name) {
            self.builtin_modules.get(id).map(|v| v.id)
        } else {
            None
        }
    }
    pub(super) fn _get_module(&self, name: &str) -> Option<usize> {
        let name = name.replace("-", "_");
        if let Some(id) = self.module_name_to_id.get(&name) {
            self.modules.get(id).map(|v| v.id)
        } else {
            None
        }
    }

    pub(super) unsafe fn _load_builtin(&mut self, name: &str) -> crate::module::Result<usize> {
        debug!("Loading builtin module: {}", name);
        let name = name.replace("-", "_");
        if let Some(id) = self.module_name_to_id.get(&name) {
            match self.builtin_modules.get_mut(id) {
                Some(v) => {
                    v.ref_count += 1;
                    Ok(*id)
                },
                None => Err(Error::NotFound(name))
            }
        } else {
            for builtin in self.builtins {
                if builtin.name() == name {
                    let mut module = unsafe { load_by_symbol(**builtin, &name, &mut self.deps) }
                        .map_err(|e| match e {
                            Error::NotFound(_) => Error::MissingMetadata,
                            e => e,
                        })?;
                    let id = self._next_module_id();
                    module.id = id;
                    self.module_name_to_id.insert(name, id);
                    self.builtin_modules.entry(id).or_insert(module);
                    return Ok(id);
                }
            }
            Err(Error::NotFound(name))
        }
    }

    pub(super) unsafe fn _load_self(&mut self, name: &str) -> crate::module::Result<usize> {
        debug!("Loading static module: {}", name);
        let name = name.replace("-", "_");
        if let Some(id) = self.module_name_to_id.get(&name) {
            match self.modules.get_mut(id) {
                Some(v) => {
                    v.ref_count += 1;
                    Ok(*id)
                },
                None => Err(Error::NotFound(name))
            }
        } else {
            let this = OsLibrary::open_self()?;
            let mut module = unsafe { load_by_symbol(this, &name, &mut self.deps) }?;
            let id = self._next_module_id();
            module.id = id;
            self.module_name_to_id.insert(name, id);
            self.modules.entry(id).or_insert(module);
            Ok(id)
        }
    }

    pub(super) unsafe fn _load(&mut self, name: &str) -> crate::module::Result<usize> {
        debug!("Loading dynamic module: {}", name);
        let name = name.replace("-", "_");
        if let Some(id) = self.module_name_to_id.get(&name) {
            match self.modules.get_mut(id) {
                Some(v) => {
                    v.ref_count += 1;
                    Ok(*id)
                },
                None => Err(Error::NotFound(name))
            }
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
                if let Some(mut module) = module {
                    let id = self._next_module_id();
                    module.id = id;
                    self.module_name_to_id.insert(name, id);
                    self.modules.insert(id, module);
                    return Ok(id);
                }
            }
            Err(Error::NotFound(name))
        }
    }

    pub(super) fn _unload(&mut self, name: &str) -> crate::module::Result<()> {
        debug!("Closing module: {}", name);
        let name = name.replace("-", "_");
        let id = self.module_name_to_id.get(&name).map(|v| *v).ok_or_else(|| Error::NotFound(name.clone()))?;
        if self.modules.contains_key(&id) {
            let module = self.modules.get_mut(&id).unwrap();
            module.ref_count -= 1;
            if module.ref_count == 0 {
                self.module_name_to_id.remove(&name);
                let module = unsafe { self.modules.remove(&id).unwrap_unchecked() };
                let main_name = format!("bp3d_os_module_{}_close", &name);
                if let Some(main) = unsafe { module.lib().load_symbol::<extern "C" fn()>(main_name)? } {
                    main.call();
                }
                drop(module);
            }
        } else {
            let module = self.builtin_modules.get_mut(&id).ok_or_else(|| Error::NotFound(name.clone()))?;
            module.ref_count -= 1;
            if module.ref_count == 0 {
                self.module_name_to_id.remove(&name);
                let module = unsafe { self.builtin_modules.remove(&id).unwrap_unchecked() };
                let main_name = format!("bp3d_os_module_{}_close", &name);
                if let Some(main) = unsafe { module.lib().load_symbol::<extern "C" fn()>(main_name)? } {
                    main.call();
                }
                drop(module);
            }
        }
        Ok(())
    }

    pub(super) fn _add_search_path(&mut self, path: impl AsRef<Path>) {
        self.paths.push(path.as_ref().into());
    }

    pub(super) fn _add_public_dependency<'a>(
        &mut self,
        name: &str,
        version: &str,
        features: impl IntoIterator<Item = &'a str>,
    ) {
        let mut negative_features = Vec::new();
        let features = features
            .into_iter()
            .filter_map(|s| {
                if s.starts_with("-") {
                    negative_features.push(s.into());
                    return None;
                }
                if s != "*" {
                    Some(String::from(name) + s)
                } else {
                    Some("*".into())
                }
            })
            .collect();
        self.deps.add_dep(
            name.replace("-", "_"),
            Dependency {
                version: version.into(),
                features,
                negative_features,
            },
        )
    }

    /// Lock the [ModuleLoader] installed for the application and returns a lock which is used to
    /// operate the [ModuleLoader].
    pub fn lock<'a>() -> Lock<'a> {
        Lock {
            lock: Self::_lock()
        }
    }
}

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
use crate::module::library::types::{OsLibrary, VirtualLibrary};
use crate::module::library::OS_EXT;
use crate::module::loader::util::{load_by_symbol, load_lib, module_close, Dependency, DepsMap};
use crate::module::loader::Lock;
use crate::module::Module;
use bp3d_debug::{debug, error};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering::SeqCst;
use std::sync::atomic::{AtomicBool, AtomicPtr};
use std::sync::{Mutex, MutexGuard};

struct Data {
    loader: AtomicPtr<Mutex<ModuleLoader>>,
    is_root: AtomicBool,
}

impl Data {
    fn install(&self, loader: ModuleLoader) -> bool {
        let ptr = self.loader.load(SeqCst);
        if ptr.is_null() {
            self.loader
                .store(Box::leak(Box::new(Mutex::new(loader))), SeqCst);
            self.is_root.store(true, SeqCst);
            true
        } else {
            false
        }
    }

    fn install_existing(&self, loader: &'static Mutex<ModuleLoader>) -> bool {
        let ptr = self.loader.load(SeqCst);
        if ptr.is_null() {
            self.is_root.store(false, SeqCst);
            self.loader
                .store(loader as *const Mutex<ModuleLoader> as *mut _, SeqCst);
            true
        } else {
            false
        }
    }

    fn is_root(&self) -> bool {
        self.is_root.load(SeqCst)
    }

    fn reset(&self) {
        self.loader.store(std::ptr::null_mut(), SeqCst);
        self.is_root.store(false, SeqCst);
    }

    fn is_set(&self) -> bool {
        !self.loader.load(SeqCst).is_null()
    }

    // This is only safe if this is set.
    unsafe fn get(&self) -> &'static Mutex<ModuleLoader> {
        let ptr = self.loader.load(SeqCst);
        unsafe { &*ptr }
    }
}

static MODULE_LOADER: Data = Data {
    loader: AtomicPtr::new(std::ptr::null_mut()),
    is_root: AtomicBool::new(false),
};

/// Represents a module loader which can support loading multiple related modules.
pub struct ModuleLoader {
    paths: Vec<PathBuf>,
    pub(super) modules: HashMap<usize, Module<OsLibrary>>,
    pub(super) builtin_modules: HashMap<usize, Module<VirtualLibrary>>,
    deps: DepsMap,
    builtins: &'static [&'static VirtualLibrary],
    module_name_to_id: HashMap<String, usize>,
    last_module_id: usize,
}

impl ModuleLoader {
    /// Create a new instance of a [ModuleLoader] and installs it as this application's
    /// [ModuleLoader].
    pub fn install(builtins: &'static [&'static VirtualLibrary]) {
        debug!("Installing new ModuleLoader...");
        let mut this = ModuleLoader {
            paths: Default::default(),
            modules: Default::default(),
            deps: DepsMap::new(),
            builtin_modules: Default::default(),
            builtins,
            module_name_to_id: Default::default(),
            last_module_id: 0,
        };
        this._add_public_dependency(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"), ["*"]);
        this._add_public_dependency("bp3d-debug", "1.0.0", ["*"]);
        if !MODULE_LOADER.install(this) {
            panic!("attempt to initialize module loader twice");
        }
    }

    /// Uninstall the application's [ModuleLoader]. This function will panic if this module did not
    /// install a [ModuleLoader] but is rather sharing the instance of a different application.
    pub fn uninstall() {
        debug!("Uninstalling ModuleLoader...");
        if !MODULE_LOADER.is_set() {
            panic!("attempt to uninstall a non-existent ModuleLoader");
        }
        if !MODULE_LOADER.is_root() {
            MODULE_LOADER.reset()
        } else {
            debug!("Unloading modules...");
            let mut loader = Self::_lock();
            let map = loader.module_name_to_id.clone();
            for (name, _) in map {
                debug!("Unloading module {}...", name);
                if let Err(e) = loader._unload(&name) {
                    error!("Failed to unload module {}: {}", name, e);
                }
            }
            drop(loader);
            debug!("Deleting ModuleLoader...");
            unsafe {
                drop(Box::from_raw(
                    MODULE_LOADER.get() as *const Mutex<ModuleLoader> as *mut Mutex<ModuleLoader>,
                ));
            }
            MODULE_LOADER.reset();
        }
    }

    pub(crate) fn _instance() -> &'static Mutex<ModuleLoader> {
        unsafe { MODULE_LOADER.get() }
    }

    /// Installs a default [ModuleLoader] for this application.
    pub fn install_default() {
        Self::install(&[]);
    }

    /// Install the [ModuleLoader] of this module to an existing instance.
    pub fn install_from_existing(loader: &'static Mutex<ModuleLoader>) {
        if MODULE_LOADER.install_existing(loader) {
            debug!("Installed ModuleLoader from existing instance");
        }
        assert_eq!(loader as *const Mutex<ModuleLoader>, unsafe {
            MODULE_LOADER.get() as *const Mutex<ModuleLoader>
        });
    }

    fn _lock<'a>() -> MutexGuard<'a, ModuleLoader> {
        if !MODULE_LOADER.is_set() {
            Self::install_default();
        }
        unsafe { MODULE_LOADER.get().lock().unwrap() }
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
                }
                None => Err(Error::NotFound(name)),
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
                }
                None => Err(Error::NotFound(name)),
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
                }
                None => Err(Error::NotFound(name)),
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
        debug!("Unloading module: {}", name);
        let name = name.replace("-", "_");
        let id = self
            .module_name_to_id
            .get(&name)
            .copied()
            .ok_or_else(|| Error::NotFound(name.clone()))?;
        if self.modules.contains_key(&id) {
            let module = self.modules.get_mut(&id).unwrap();
            module.ref_count -= 1;
            if module.ref_count == 0 {
                self.module_name_to_id.remove(&name);
                let module = unsafe { self.modules.remove(&id).unwrap_unchecked() };
                unsafe { module_close(&name, false, &module) }?;
                drop(module);
            }
        } else {
            let module = self
                .builtin_modules
                .get_mut(&id)
                .ok_or_else(|| Error::NotFound(name.clone()))?;
            module.ref_count -= 1;
            if module.ref_count == 0 {
                self.module_name_to_id.remove(&name);
                let module = unsafe { self.builtin_modules.remove(&id).unwrap_unchecked() };
                unsafe { module_close(&name, true, &module) }?;
                drop(module);
            }
        }
        Ok(())
    }

    pub(super) fn _add_search_path(&mut self, path: impl AsRef<Path>) {
        self.paths.push(path.as_ref().into());
    }

    pub(super) fn _remove_search_path(&mut self, path: impl AsRef<Path>) {
        self.paths.retain(|p| p != path.as_ref());
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
            lock: Self::_lock(),
        }
    }
}

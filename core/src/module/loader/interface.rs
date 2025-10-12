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
use crate::module::library::Library;
use crate::module::loader::ModuleLoader;
use crate::module::Module;
use std::ops::Deref;
use std::path::Path;
use std::sync::MutexGuard;

/// Represents a handle to a [Module] stored in the application's [ModuleLoader].
pub struct ModuleHandle<'a, L: Library, F: Fn(&ModuleLoader, usize) -> &Module<L>> {
    loader: &'a ModuleLoader,
    id: usize,
    f: F,
}

impl<'a, L: Library, F: Fn(&ModuleLoader, usize) -> &Module<L>> Deref for ModuleHandle<'a, L, F> {
    type Target = Module<L>;

    fn deref(&self) -> &Self::Target {
        (self.f)(self.loader, self.id)
    }
}

macro_rules! module_handle {
    ($l: lifetime, $t: ty) => { ModuleHandle<$l, $t, impl Fn(&ModuleLoader, usize) -> &Module<$t>> };
}

/// A structure that represents a lock to the application's [ModuleLoader].
pub struct Lock<'a> {
    pub(super) lock: MutexGuard<'a, ModuleLoader>,
}

impl<'a> Lock<'a> {
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
    pub unsafe fn load_builtin(&mut self, name: &str) -> Result<module_handle!('_, VirtualLibrary), Error> {
        self.lock._load_builtin(name).map(|id| ModuleHandle {
            loader: &self.lock,
            id,
            f: |loader, id| loader.builtin_modules.get(&id).unwrap(),
        })
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
    pub unsafe fn load_self(&mut self, name: &str) -> crate::module::Result<module_handle!('_, OsLibrary)> {
        self.lock._load_self(name).map(|id| ModuleHandle {
            loader: &self.lock,
            id,
            f: |lock, id| lock.modules.get(&id).unwrap(),
        })
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
    pub unsafe fn load(&mut self, name: &str) -> crate::module::Result<module_handle!('_, OsLibrary)> {
        self.lock._load(name).map(|id| ModuleHandle {
            loader: &self.lock,
            id,
            f: |lock, id| lock.modules.get(&id).unwrap(),
        })
    }

    /// Attempts to unload the given module.
    ///
    /// # Arguments
    ///
    /// * `name`: the name of the module to unload.
    ///
    /// returns: ()
    pub fn unload(&mut self, name: &str) -> crate::module::Result<()> {
        self.lock._unload(name)
    }

    /// Adds the given path to the path search list.
    ///
    /// # Arguments
    ///
    /// * `path`: the path to include.
    ///
    /// returns: ()
    pub fn add_search_path(&mut self, path: impl AsRef<Path>) {
        self.lock._add_search_path(path);
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
    pub fn add_public_dependency<'b>(&mut self, name: &str, version: &str, features: impl IntoIterator<Item = &'b str>) {
        self.lock._add_public_dependency(name, version, features);
    }

    /// Returns the builtin module identified by the name `name`, returns [None] if the module is
    /// not loaded.
    pub fn get_builtin(&self, name: &str) -> Option<module_handle!('_, VirtualLibrary)> {
        self.lock._get_builtin(name).map(|id| ModuleHandle {
            loader: &self.lock,
            id,
            f: |lock, id| lock.builtin_modules.get(&id).unwrap(),
        })
    }

    /// Returns the module identified by the name `name`, returns [None] if the module is
    /// not loaded.
    pub fn get_module(&self, name: &str) -> Option<module_handle!('_, OsLibrary)> {
        self.lock._get_module(name).map(|id| ModuleHandle {
            loader: &self.lock,
            id,
            f: |lock, id| lock.modules.get(&id).unwrap(),
        })
    }
}

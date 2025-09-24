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

use std::collections::{HashMap, HashSet};
use std::ffi::{c_char, CStr};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Mutex;
use bp3d_debug::{debug, info, trace};
use crate::module::error::{Error, IncompatibleDependency, IncompatibleRustc};
use crate::module::library::types::{OsLibrary, Symbol};
use crate::module::metadata::Value;
use crate::module::{Module, RUSTC_VERSION};
use crate::module::library::Library;
use crate::module::loader::ModuleLoader;

pub struct Dependency {
    pub version: String,
    pub features: Vec<String>,
    pub negative_features: Vec<String>,
}

pub struct DepsMap {
    pub deps_by_module: HashMap<String, HashMap<String, Dependency>>,
    pub module_by_dep: HashMap<String, Vec<String>>,
    pub module_version: HashMap<String, String>,
    pub master: HashMap<String, Dependency>,
    dummy: HashMap<String, Dependency>,
}

impl DepsMap {
    pub fn new() -> Self {
        Self {
            deps_by_module: HashMap::new(),
            module_by_dep: HashMap::new(),
            module_version: HashMap::new(),
            master: HashMap::new(),
            dummy: HashMap::new(),
        }
    }

    pub fn add_dep(&mut self, name: String, dep: Dependency) {
        self.master.insert(name, dep);
    }

    pub fn insert_module(
        &mut self,
        name: &Value,
        version: &Value,
        deps: &Value,
        features: &Value,
    ) -> crate::module::Result<()> {
        let mut deps3 = Vec::new();
        let mut features1: Vec<String> = Vec::new();
        if let Some(deps) = deps.parse_key_value_pairs() {
            for dep in deps {
                let (name, version) = dep?;
                deps3.push((name, version.into()));
            }
        }
        if let Some(features) = features.as_list() {
            features1 = features.map(String::from).collect();
        }
        let name: String = name.as_str().into();
        for (name1, _) in &deps3 {
            self.module_by_dep
                .entry(name1.clone())
                .or_default()
                .push(name.clone());
        }
        let mut deps2 = HashMap::new();
        for (name, version) in deps3 {
            let features = features1
                .iter()
                .filter_map(|v| match v.starts_with(&name) {
                    true => Some(v.clone()),
                    false => None,
                })
                .collect();
            deps2.insert(
                name,
                Dependency {
                    version,
                    features,
                    negative_features: Vec::new(),
                },
            );
        }
        self.deps_by_module.insert(name.clone(), deps2);
        self.module_version.insert(name, version.as_str().into());
        Ok(())
    }

    pub fn get_module_by_dep(
        &self,
        name: &str,
    ) -> Option<impl Iterator<Item = &HashMap<String, Dependency>>> {
        Some(
            self.module_by_dep
                .get(name)?
                .iter()
                .map(|v| self.deps_by_module.get(v).unwrap_or(&self.dummy)),
        )
    }
}

type DebugInit = extern "Rust" fn(engine: &'static dyn bp3d_debug::engine::Engine);

type ModuleInit = extern "Rust" fn(engine: &'static Mutex<ModuleLoader>);

type ModuleUninit = extern "Rust" fn();

const MOD_HEADER: &[u8] = b"BP3D_OS_MODULE|";

fn parse_metadata(bytes: &[u8]) -> crate::module::Result<crate::module::metadata::Metadata> {
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
        map.insert(key.into(), Value::new(value.into()));
    }
    Ok(map)
}

fn load_metadata(path: &Path) -> crate::module::Result<crate::module::metadata::Metadata> {
    let mut file = File::open(path).map_err(Error::Io)?;
    let mut buffer: [u8; 8192] = [0; 8192];
    let mut v = Vec::new();
    while file.read(&mut buffer).map_err(Error::Io)? > 0 {
        let mut slice = &buffer[..];
        while let Some(pos) = slice.iter().position(|v| *v == b'B') {
            let inner = &slice[pos..];
            let end = inner
                .iter()
                .position(|v| *v == 0)
                .unwrap_or(inner.len() - 1);
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

fn check_deps(
    deps: &Value,
    features: &Value,
    deps2: &HashMap<String, Dependency>,
) -> crate::module::Result<()> {
    if let Some(deps) = deps.parse_key_value_pairs() {
        for res in deps {
            let (name, version) = res?;
            if let Some(dep) = deps2.get(&name) {
                if version != dep.version {
                    return Err(Error::IncompatibleDep(IncompatibleDependency {
                        name,
                        expected_version: dep.version.clone(),
                        actual_version: version.into(),
                    }));
                }
                if let Some(features) = features.as_list() {
                    let features: HashSet<&str> =
                        features.filter(|v| v.starts_with(&name)).collect();
                    let mut flag = true;
                    for feature in dep.negative_features.iter() {
                        if features.contains(&**feature) {
                            return Err(Error::IncompatibleFeatureSet(name));
                        }
                    }
                    for feature in dep.features.iter() {
                        if feature == "*" {
                            //Once a '*' is received; break as this is considered
                            // as a match all pattern.
                            flag = false;
                            break;
                        }
                        if !features.contains(&**feature) {
                            return Err(Error::IncompatibleFeatureSet(name));
                        }
                    }
                    if flag && (features.len() != dep.features.len()) {
                        return Err(Error::IncompatibleFeatureSet(name));
                    }
                } else if !dep.features.is_empty() {
                    return Err(Error::IncompatibleFeatureSet(name));
                }
            }
        }
    }
    Ok(())
}

fn check_metadata(metadata: &crate::module::metadata::Metadata, deps3: &mut DepsMap) -> crate::module::Result<()> {
    if metadata.get("TYPE").ok_or(Error::InvalidMetadata)?.as_str() == "RUST" {
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
        // The name of the module.
        let module_name = metadata.get("NAME").ok_or(Error::MissingModuleName)?;
        // The version of the module.
        let module_version = metadata.get("VERSION").ok_or(Error::MissingModuleName)?;
        // The list of all features enabled on all deps.
        let features = metadata
            .get("FEATURES")
            .ok_or(Error::MissingFeaturesForRust)?;
        if rustc_version.as_str() != RUSTC_VERSION {
            //mismatch between rust versions
            return Err(Error::RustcVersionMismatch(IncompatibleRustc {
                expected: RUSTC_VERSION,
                actual: rustc_version.as_str().into(),
            }));
        }
        check_deps(deps, features, &deps3.master)?;
        if let Some(modules) = deps3.get_module_by_dep(module_name.as_str()) {
            debug!(
                "Checking dependencies for {} against other modules...",
                module_name.as_str()
            );
            for deps2 in modules {
                check_deps(deps, features, deps2)?;
            }
        }
        if let Some(deps1) = deps.parse_key_value_pairs() {
            for res in deps1 {
                let (name, actual_version) = res?;
                if let Some(deps2) = deps3.deps_by_module.get(&name) {
                    debug!(
                        "Checking dependencies for {} against module {}...",
                        module_name.as_str(),
                        name
                    );
                    let version = deps3.module_version.get(&name).unwrap();
                    trace!(
                        "expected_version: {}, actual_version: {}",
                        version,
                        actual_version
                    );
                    if version != actual_version {
                        return Err(Error::IncompatibleDep(IncompatibleDependency {
                            name,
                            expected_version: version.clone(),
                            actual_version: module_version.as_str().into(),
                        }));
                    }
                    check_deps(deps, features, deps2)?;
                }
            }
        }
        deps3.insert_module(module_name, module_version, deps, features)?;
    }
    Ok(())
}

pub unsafe fn load_lib(deps3: &mut DepsMap, name: &str, path: &Path) -> crate::module::Result<Module<OsLibrary>> {
    let metadata = load_metadata(path)?;
    check_metadata(&metadata, deps3)?;
    let module = Module::new(OsLibrary::load(path)?, metadata);
    module_open(name, &module)?;
    Ok(module)
}

unsafe fn module_open<L: Library>(name: &str, module: &Module<L>) -> crate::module::Result<()> {
    let name = module.get_metadata_key("NAME").unwrap_or(name);
    let version = module.get_metadata_key("VERSION").unwrap_or("UNKNOWN");
    info!("Opening module {}-{}...", name, version);
    if module
        .get_metadata_key("TYPE")
        .ok_or(Error::InvalidMetadata)?
        == "RUST"
    {
        let debug_init_name = format!("bp3d_os_module_{}_init_bp3d_debug", name);
        if let Some(debug_init) = module.lib().load_symbol::<DebugInit>(debug_init_name)? {
            debug!("Initializing bp3d-debug for module: {}", name);
            debug_init.call(bp3d_debug::engine::get())
        }
        let init_name = format!("bp3d_os_module_{}_init", name);
        let sym: Symbol<ModuleInit> = module.lib().load_symbol(init_name)?.ok_or(Error::MissingModuleInitForRust)?;
        sym.call(ModuleLoader::_instance());
    }
    let main_name = format!("bp3d_os_module_{}_open", name);
    if let Some(main) = module.lib().load_symbol::<extern "C" fn()>(main_name)? {
        debug!("Running module_open for module: {}", name);
        main.call();
    }
    Ok(())
}

pub unsafe fn module_close<L: Library>(name: &str, builtin: bool, module: &Module<L>) -> crate::module::Result<()> {
    let name = module.get_metadata_key("NAME").unwrap_or(name);
    let version = module.get_metadata_key("VERSION").unwrap_or("UNKNOWN");
    info!("Closing module {}-{}...", name, version);
    if !builtin && module
        .get_metadata_key("TYPE")
        .ok_or(Error::InvalidMetadata)?
        == "RUST"
    {
        let init_name = format!("bp3d_os_module_{}_uninit", name);
        let sym: Symbol<ModuleUninit> = module.lib().load_symbol(init_name)?.ok_or(Error::MissingModuleInitForRust)?;
        debug!("module_uninit");
        sym.call();
    }
    let main_name = format!("bp3d_os_module_{}_close", &name);
    if let Some(main) = unsafe { module.lib().load_symbol::<extern "C" fn()>(main_name)? } {
        debug!("module_close");
        main.call();
    }
    Ok(())
}

pub unsafe fn load_by_symbol<L: Library>(lib: L, name: &str, deps: &mut DepsMap) -> crate::module::Result<Module<L>> {
    let mod_const_name = format!("BP3D_OS_MODULE_{}", name.to_uppercase());
    if let Some(sym) = lib.load_symbol::<*const c_char>(mod_const_name)? {
        let bytes = CStr::from_ptr((*sym.as_ptr()).offset(1)).to_bytes_with_nul();
        let metadata = parse_metadata(bytes)?;
        check_metadata(&metadata, deps)?;
        let module = Module::new(lib, metadata);
        module_open(name, &module)?;
        return Ok(module);
    }
    Err(Error::NotFound(name.into()))
}

#[cfg(test)]
mod tests {
    use super::{check_metadata, Dependency, DepsMap};
    use crate::module::metadata::{Metadata, Value};
    use crate::module::RUSTC_VERSION;

    #[test]
    fn test_basic() {
        let mut deps = DepsMap::new();
        let mut metadata = Metadata::new();
        metadata.insert("TYPE".into(), Value::new("RUST".into()));
        metadata.insert("RUSTC".into(), Value::new(RUSTC_VERSION.into()));
        metadata.insert("NAME".into(), Value::new("test".into()));
        metadata.insert("VERSION".into(), Value::new("1.0.0".into()));
        metadata.insert("DEPS".into(), Value::new("".into()));
        metadata.insert("FEATURES".into(), Value::new("".into()));
        check_metadata(&metadata, &mut deps).unwrap();
    }

    #[test]
    fn test_deps_no_relations() {
        let mut deps = DepsMap::new();
        let mut metadata = Metadata::new();
        metadata.insert("TYPE".into(), Value::new("RUST".into()));
        metadata.insert("RUSTC".into(), Value::new(RUSTC_VERSION.into()));
        metadata.insert("NAME".into(), Value::new("test".into()));
        metadata.insert("VERSION".into(), Value::new("1.0.0".into()));
        metadata.insert("DEPS".into(), Value::new("a=1.0.0,b=2.0.0".into()));
        metadata.insert("FEATURES".into(), Value::new("".into()));
        check_metadata(&metadata, &mut deps).unwrap();

        // We have a different named module with no relation to the other, no ABI incompatibility
        // exists.
        metadata.insert("DEPS".into(), Value::new("a=0.1.0,b=0.2.0".into()));
        metadata.insert("NAME".into(), Value::new("test1".into()));
        check_metadata(&metadata, &mut deps).unwrap();
    }

    #[test]
    fn test_deps_with_relation_1() {
        let mut deps = DepsMap::new();
        let mut metadata = Metadata::new();
        metadata.insert("TYPE".into(), Value::new("RUST".into()));
        metadata.insert("RUSTC".into(), Value::new(RUSTC_VERSION.into()));
        metadata.insert("NAME".into(), Value::new("test".into()));
        metadata.insert("VERSION".into(), Value::new("1.0.0".into()));
        metadata.insert("DEPS".into(), Value::new("a=1.0.0,b=2.0.0".into()));
        metadata.insert("FEATURES".into(), Value::new("".into()));
        check_metadata(&metadata, &mut deps).unwrap();

        metadata.insert("DEPS".into(), Value::new("a=1.0.0,test=1.0.0".into()));
        metadata.insert("NAME".into(), Value::new("test1".into()));
        check_metadata(&metadata, &mut deps).unwrap();

        metadata.insert("DEPS".into(), Value::new("b=2.0.0,test=1.0.0".into()));
        metadata.insert("NAME".into(), Value::new("test2".into()));
        check_metadata(&metadata, &mut deps).unwrap();

        metadata.insert(
            "DEPS".into(),
            Value::new("a=1.0.0,b=2.0.0,test=1.0.0".into()),
        );
        metadata.insert("NAME".into(), Value::new("test3".into()));
        check_metadata(&metadata, &mut deps).unwrap();

        metadata.insert("DEPS".into(), Value::new("test=1.0.0".into()));
        metadata.insert("NAME".into(), Value::new("test4".into()));
        check_metadata(&metadata, &mut deps).unwrap();
    }

    #[test]
    fn test_deps_with_relation_2() {
        let mut deps = DepsMap::new();
        let mut metadata = Metadata::new();
        metadata.insert("TYPE".into(), Value::new("RUST".into()));
        metadata.insert("RUSTC".into(), Value::new(RUSTC_VERSION.into()));
        metadata.insert("NAME".into(), Value::new("test".into()));
        metadata.insert("VERSION".into(), Value::new("1.0.0".into()));
        metadata.insert("DEPS".into(), Value::new("a=1.0.0,b=2.0.0".into()));
        metadata.insert("FEATURES".into(), Value::new("".into()));
        check_metadata(&metadata, &mut deps).unwrap();

        metadata.insert(
            "DEPS".into(),
            Value::new("a=1.0.0,b=1.2.0,test=1.0.0".into()),
        );
        metadata.insert("NAME".into(), Value::new("test1".into()));
        check_metadata(&metadata, &mut deps).unwrap_err();

        metadata.insert("DEPS".into(), Value::new("a=0.1.0,test=1.0.0".into()));
        metadata.insert("NAME".into(), Value::new("test2".into()));
        check_metadata(&metadata, &mut deps).unwrap_err();
    }

    #[test]
    fn test_deps_with_relation_incompatible_version() {
        let mut deps = DepsMap::new();
        let mut metadata = Metadata::new();
        metadata.insert("TYPE".into(), Value::new("RUST".into()));
        metadata.insert("RUSTC".into(), Value::new(RUSTC_VERSION.into()));
        metadata.insert("NAME".into(), Value::new("test".into()));
        metadata.insert("VERSION".into(), Value::new("1.0.0".into()));
        metadata.insert("DEPS".into(), Value::new("a=1.0.0,b=2.0.0".into()));
        metadata.insert("FEATURES".into(), Value::new("".into()));
        check_metadata(&metadata, &mut deps).unwrap();

        metadata.insert("DEPS".into(), Value::new("a=1.0.0,test=0.1.0".into()));
        metadata.insert("NAME".into(), Value::new("test1".into()));
        check_metadata(&metadata, &mut deps).unwrap_err();
    }

    #[test]
    fn test_deps_features() {
        let mut deps = DepsMap::new();
        let mut metadata = Metadata::new();
        metadata.insert("TYPE".into(), Value::new("RUST".into()));
        metadata.insert("RUSTC".into(), Value::new(RUSTC_VERSION.into()));
        metadata.insert("NAME".into(), Value::new("test".into()));
        metadata.insert("VERSION".into(), Value::new("1.0.0".into()));
        metadata.insert("DEPS".into(), Value::new("a=1.0.0,b=2.0.0".into()));
        metadata.insert("FEATURES".into(), Value::new("a/abc,a/def".into()));
        check_metadata(&metadata, &mut deps).unwrap();

        metadata.insert("DEPS".into(), Value::new("a=1.0.0,test=1.0.0".into()));
        metadata.insert("NAME".into(), Value::new("test1".into()));
        metadata.insert("FEATURES".into(), Value::new("a/abc,a/def".into()));
        check_metadata(&metadata, &mut deps).unwrap();
    }

    #[test]
    fn test_deps_features_incompatible() {
        let mut deps = DepsMap::new();
        let mut metadata = Metadata::new();
        metadata.insert("TYPE".into(), Value::new("RUST".into()));
        metadata.insert("RUSTC".into(), Value::new(RUSTC_VERSION.into()));
        metadata.insert("NAME".into(), Value::new("test".into()));
        metadata.insert("VERSION".into(), Value::new("1.0.0".into()));
        metadata.insert("DEPS".into(), Value::new("a=1.0.0,b=2.0.0".into()));
        metadata.insert("FEATURES".into(), Value::new("a/abc,a/def".into()));
        check_metadata(&metadata, &mut deps).unwrap();

        metadata.insert("DEPS".into(), Value::new("a=1.0.0,test=1.0.0".into()));
        metadata.insert("NAME".into(), Value::new("test1".into()));
        metadata.insert("FEATURES".into(), Value::new("a/abc".into()));
        check_metadata(&metadata, &mut deps).unwrap_err();

        metadata.insert(
            "DEPS".into(),
            Value::new("a=1.0.0,b=2.0.0,test=1.0.0".into()),
        );
        metadata.insert("NAME".into(), Value::new("test3".into()));
        metadata.insert("FEATURES".into(), Value::new("a/abc,a/def,a/ghi".into()));
        check_metadata(&metadata, &mut deps).unwrap_err();

        metadata.insert(
            "DEPS".into(),
            Value::new("a=1.0.0,b=2.0.0,test=1.0.0".into()),
        );
        metadata.insert("NAME".into(), Value::new("test2".into()));
        metadata.insert("FEATURES".into(), Value::new("a/abc,a/def,b/ghi".into()));
        check_metadata(&metadata, &mut deps).unwrap_err();
    }

    #[test]
    fn test_master_1() {
        let mut deps = DepsMap::new();
        deps.add_dep(
            "a".into(),
            Dependency {
                version: "1.0.0".into(),
                features: vec!["*".into()],
                negative_features: vec![],
            },
        );
        let mut metadata = Metadata::new();
        metadata.insert("TYPE".into(), Value::new("RUST".into()));
        metadata.insert("RUSTC".into(), Value::new(RUSTC_VERSION.into()));
        metadata.insert("NAME".into(), Value::new("test".into()));
        metadata.insert("VERSION".into(), Value::new("1.0.0".into()));
        metadata.insert("DEPS".into(), Value::new("a=1.0.0,b=2.0.0".into()));
        metadata.insert("FEATURES".into(), Value::new("a/abc,a/def".into()));
        check_metadata(&metadata, &mut deps).unwrap();
    }

    #[test]
    fn test_master_2() {
        let mut deps = DepsMap::new();
        deps.add_dep(
            "a".into(),
            Dependency {
                version: "1.0.0".into(),
                features: vec!["a/abc".into(), "*".into()],
                negative_features: vec![],
            },
        );
        let mut metadata = Metadata::new();
        metadata.insert("TYPE".into(), Value::new("RUST".into()));
        metadata.insert("RUSTC".into(), Value::new(RUSTC_VERSION.into()));
        metadata.insert("NAME".into(), Value::new("test".into()));
        metadata.insert("VERSION".into(), Value::new("1.0.0".into()));
        metadata.insert("DEPS".into(), Value::new("a=1.0.0,b=2.0.0".into()));
        metadata.insert("FEATURES".into(), Value::new("a/abc,a/def".into()));
        check_metadata(&metadata, &mut deps).unwrap();
    }

    #[test]
    fn test_master_3() {
        let mut deps = DepsMap::new();
        deps.add_dep(
            "a".into(),
            Dependency {
                version: "1.0.0".into(),
                features: vec!["a/abc".into(), "a/def".into(), "*".into()],
                negative_features: vec![],
            },
        );
        let mut metadata = Metadata::new();
        metadata.insert("TYPE".into(), Value::new("RUST".into()));
        metadata.insert("RUSTC".into(), Value::new(RUSTC_VERSION.into()));
        metadata.insert("NAME".into(), Value::new("test".into()));
        metadata.insert("VERSION".into(), Value::new("1.0.0".into()));
        metadata.insert("DEPS".into(), Value::new("a=1.0.0,b=2.0.0".into()));
        metadata.insert("FEATURES".into(), Value::new("a/abc,a/def".into()));
        check_metadata(&metadata, &mut deps).unwrap();
    }

    #[test]
    fn test_master_incompatible_version() {
        let mut deps = DepsMap::new();
        deps.add_dep(
            "a".into(),
            Dependency {
                version: "1.0.0".into(),
                features: vec!["a/abc".into(), "a/def".into(), "*".into()],
                negative_features: vec![],
            },
        );
        let mut metadata = Metadata::new();
        metadata.insert("TYPE".into(), Value::new("RUST".into()));
        metadata.insert("RUSTC".into(), Value::new(RUSTC_VERSION.into()));
        metadata.insert("NAME".into(), Value::new("test".into()));
        metadata.insert("VERSION".into(), Value::new("1.0.0".into()));
        metadata.insert("DEPS".into(), Value::new("a=0.1.0,b=2.0.0".into()));
        metadata.insert("FEATURES".into(), Value::new("a/abc,a/def".into()));
        check_metadata(&metadata, &mut deps).unwrap_err();
    }

    #[test]
    fn test_master_incompatible_feature_set_1() {
        let mut deps = DepsMap::new();
        deps.add_dep(
            "a".into(),
            Dependency {
                version: "1.0.0".into(),
                features: vec!["a/abc".into(), "a/def".into(), "*".into()],
                negative_features: vec![],
            },
        );
        let mut metadata = Metadata::new();
        metadata.insert("TYPE".into(), Value::new("RUST".into()));
        metadata.insert("RUSTC".into(), Value::new(RUSTC_VERSION.into()));
        metadata.insert("NAME".into(), Value::new("test".into()));
        metadata.insert("VERSION".into(), Value::new("1.0.0".into()));
        metadata.insert("DEPS".into(), Value::new("a=1.0.0,b=2.0.0".into()));
        metadata.insert("FEATURES".into(), Value::new("a/abc".into()));
        check_metadata(&metadata, &mut deps).unwrap_err();
    }

    #[test]
    fn test_master_4() {
        let mut deps = DepsMap::new();
        deps.add_dep(
            "a".into(),
            Dependency {
                version: "1.0.0".into(),
                features: vec!["a/abc".into(), "*".into()],
                negative_features: vec!["a/def".into()],
            },
        );
        let mut metadata = Metadata::new();
        metadata.insert("TYPE".into(), Value::new("RUST".into()));
        metadata.insert("RUSTC".into(), Value::new(RUSTC_VERSION.into()));
        metadata.insert("NAME".into(), Value::new("test".into()));
        metadata.insert("VERSION".into(), Value::new("1.0.0".into()));
        metadata.insert("DEPS".into(), Value::new("a=1.0.0,b=2.0.0".into()));
        metadata.insert("FEATURES".into(), Value::new("a/abc".into()));
        check_metadata(&metadata, &mut deps).unwrap();
    }

    #[test]
    fn test_master_incompatible_feature_set_2() {
        let mut deps = DepsMap::new();
        deps.add_dep(
            "a".into(),
            Dependency {
                version: "1.0.0".into(),
                features: vec!["a/abc".into(), "*".into()],
                negative_features: vec!["a/def".into()],
            },
        );
        let mut metadata = Metadata::new();
        metadata.insert("TYPE".into(), Value::new("RUST".into()));
        metadata.insert("RUSTC".into(), Value::new(RUSTC_VERSION.into()));
        metadata.insert("NAME".into(), Value::new("test".into()));
        metadata.insert("VERSION".into(), Value::new("1.0.0".into()));
        metadata.insert("DEPS".into(), Value::new("a=1.0.0,b=2.0.0".into()));
        metadata.insert("FEATURES".into(), Value::new("a/abc,a/def".into()));
        check_metadata(&metadata, &mut deps).unwrap_err();
    }
}

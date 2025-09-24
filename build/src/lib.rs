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

use cargo_lock::Lockfile;
use cargo_manifest::Manifest;
use itertools::Itertools;
use std::path::PathBuf;

pub struct ModuleMain {
    rust_code: String,
    out_path: PathBuf,
    crate_name: String,
    virtual_lib: String,
}

impl Default for ModuleMain {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleMain {
    pub fn new() -> Self {
        let crate_name = std::env::var("CARGO_PKG_NAME").unwrap().replace('-', "_");
        let crate_version = std::env::var("CARGO_PKG_VERSION").unwrap();
        let rustc_version = rustc_version::version().unwrap();
        let mod_const_name = format!("BP3D_OS_MODULE_{}", crate_name.to_uppercase());
        let mut manifest_path = PathBuf::from(
            std::env::var_os("CARGO_MANIFEST_PATH").expect("Failed to get CARGO_MANIFEST_PATH"),
        );
        let package =
            Manifest::from_path(&manifest_path).expect("Failed to read CARGO_MANIFEST_PATH");
        manifest_path.set_extension("lock");
        let lock_file = Lockfile::load(&manifest_path).ok();
        let mut features = Vec::new();
        let deps_list = package
            .dependencies
            .map(|v| {
                v.iter()
                    .map(|(k, v)| {
                        let dep_version = lock_file
                            .as_ref()
                            .and_then(|v| v.packages.iter().find(|v| v.name.as_ref() == *k))
                            .map(|v| &v.version);
                        for feature in v.req_features() {
                            features.push(format!("{}/{}", k, feature));
                        }
                        match dep_version {
                            Some(v) => format!("{}={}", k, v),
                            None => format!("{}={}", k, v.req()),
                        }
                    })
                    .join(",")
            })
            .unwrap_or("".into());
        let data = format!(
            "\"\0BP3D_OS_MODULE|TYPE=RUST|NAME={}|VERSION={}|RUSTC={}|DEPS={}|FEATURES={}\0\"",
            crate_name,
            crate_version,
            rustc_version,
            deps_list,
            features.join(",")
        );
        let rust_code = format!(
            "
    #[unsafe(no_mangle)]
    #[allow(clippy::manual_c_str_literals)] // The string is enclosed in NULLs and apparently clippy
    // does not like that...
    static mut {mod_const_name}: *const std::ffi::c_char = {data}.as_ptr() as _;
"
        );
        let virtual_lib = format!("
    #[allow(static_mut_refs)]
    pub static VIRTUAL_MODULE: bp3d_os::module::library::types::VirtualLibrary = bp3d_os::module::library::types::VirtualLibrary::new(\"{crate_name}\", &[
        (\"{mod_const_name}\", unsafe {{ &{mod_const_name} as *const *const i8 as *const std::ffi::c_void }})");
        let out_path =
            PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("bp3d_os_module.rs");
        let this = Self {
            rust_code,
            out_path,
            crate_name,
            virtual_lib,
        };
        this.add_init().add_uninit()
    }

    pub fn add_export(mut self, func_name: impl AsRef<str>) -> Self {
        let func_name = func_name.as_ref();
        self.virtual_lib += &format!(",\n        (\"{func_name}\", {func_name} as _)");
        self
    }

    fn add_init(mut self) -> Self {
        let motherfuckingrust = "extern \"C\"";
        let crate_name = &self.crate_name;
        let rust_code = format!(
            r"
    #[unsafe(no_mangle)]
    #[inline(never)]
    pub {motherfuckingrust} fn bp3d_os_module_{crate_name}_init(loader: &'static std::sync::Mutex<bp3d_os::module::loader::ModuleLoader>) {{
        bp3d_os::module::loader::ModuleLoader::install_from_existing(loader);
    }}
"
        );
        self.rust_code += &rust_code;
        let motherfuckingrust = format!("bp3d_os_module_{crate_name}_init");
        self.add_export(motherfuckingrust)
    }

    fn add_uninit(mut self) -> Self {
        let motherfuckingrust = "extern \"C\"";
        let crate_name = &self.crate_name;
        let rust_code = format!(
            r"
    #[unsafe(no_mangle)]
    #[inline(never)]
    pub {motherfuckingrust} fn bp3d_os_module_{crate_name}_uninit() {{
        bp3d_os::module::loader::ModuleLoader::uninstall();
    }}
"
        );
        self.rust_code += &rust_code;
        let motherfuckingrust = format!("bp3d_os_module_{crate_name}_uninit");
        self.add_export(motherfuckingrust)
    }

    pub fn add_open(mut self) -> Self {
        let motherfuckingrust = "extern \"C\"";
        let crate_name = &self.crate_name;
        let rust_code = format!(
            r"
    #[unsafe(no_mangle)]
    #[inline(never)]
    pub {motherfuckingrust} fn bp3d_os_module_{crate_name}_open() {{
        module_open();
    }}
"
        );
        self.rust_code += &rust_code;
        let motherfuckingrust = format!("bp3d_os_module_{crate_name}_open");
        self.add_export(motherfuckingrust)
    }

    pub fn add_close(mut self) -> Self {
        let motherfuckingrust = "extern \"C\"";
        let crate_name = &self.crate_name;
        let rust_code = format!(
            r"
    #[unsafe(no_mangle)]
    #[inline(never)]
    pub {motherfuckingrust} fn bp3d_os_module_{crate_name}_close() {{
        module_close();
    }}
"
        );
        self.rust_code += &rust_code;
        let motherfuckingrust = format!("bp3d_os_module_{crate_name}_close");
        self.add_export(motherfuckingrust)
    }

    pub fn add_bp3d_debug(mut self) -> Self {
        let motherfuckingrust = "extern \"Rust\"";
        let crate_name = &self.crate_name;
        let rust_code = format!(
            r"
    #[unsafe(no_mangle)]
    #[inline(never)]
    pub {motherfuckingrust} fn bp3d_os_module_{crate_name}_init_bp3d_debug(engine: &'static dyn bp3d_debug::engine::Engine) {{
        bp3d_debug::engine::set(engine);
    }}
"
        );
        self.rust_code += &rust_code;
        let motherfuckingrust = format!("bp3d_os_module_{crate_name}_init_bp3d_debug");
        self.add_export(motherfuckingrust)
    }

    pub fn build(mut self) {
        self.virtual_lib += "\n    ]);";
        self.rust_code += &self.virtual_lib;
        std::fs::write(&self.out_path, self.rust_code).unwrap();
        #[cfg(unix)]
        {
            let crate_name = self.crate_name;
            #[cfg(target_vendor = "apple")]
            println!("cargo::rustc-link-arg-cdylib=-Wl,-install_name,@rpath/lib{crate_name}.dylib");
            #[cfg(all(unix, not(target_vendor = "apple")))]
            println!("cargo::rustc-link-arg-cdylib=-Wl,-soname,lib{crate_name}.so");
        }
        println!(
            "cargo:rustc-env=BP3D_OS_MODULE_MAIN={}",
            self.out_path.display()
        );
    }
}

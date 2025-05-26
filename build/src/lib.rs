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

use cargo_manifest::Manifest;
use itertools::Itertools;
use std::path::PathBuf;

pub struct ModuleMain {
    rust_code: String,
    out_path: PathBuf,
    crate_name: String,
}

impl ModuleMain {
    pub fn new() -> Self {
        let crate_name = std::env::var("CARGO_PKG_NAME").unwrap().replace('-', "_");
        let crate_version = std::env::var("CARGO_PKG_VERSION").unwrap();
        let rustc_version = rustc_version::version().unwrap();
        let mod_const_name = format!("BP3D_OS_MODULE_{}", crate_name.to_uppercase());
        let package = Manifest::from_path(
            std::env::var_os("CARGO_MANIFEST_PATH").expect("Failed to get CARGO_MANIFEST_PATH"),
        )
        .expect("Failed to read CARGO_MANIFEST_PATH");
        let deps_list = package
            .dependencies
            .map(|v| {
                v.iter()
                    .map(|(k, v)| format!("{}={}", k, v.req()))
                    .join(",")
            })
            .unwrap_or("".into());
        let data = format!(
            "\"\0BP3D_OS_MODULE|TYPE=RUST|NAME={}|VERSION={}|RUSTC={}|DEPS={}\0\"",
            crate_name, crate_version, rustc_version, deps_list
        );
        let rust_code = format!(
            r"
    #[unsafe(no_mangle)]
    static mut {mod_const_name}: *const std::ffi::c_char = {data}.as_ptr() as _;
"
        );
        let out_path =
            PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("bp3d_os_module.rs");
        Self {
            rust_code,
            out_path,
            crate_name,
        }
    }

    pub fn add_open(mut self) -> Self {
        let motherfuckingrust = "extern \"C\"";
        let crate_name = &self.crate_name;
        let rust_code = format!(
            r"
    #[unsafe(no_mangle)]
    pub {motherfuckingrust} fn bp3d_os_module_{crate_name}_open() {{
        module_open();
    }}
"
        );
        self.rust_code += &rust_code;
        self
    }

    pub fn add_close(mut self) -> Self {
        let motherfuckingrust = "extern \"C\"";
        let crate_name = &self.crate_name;
        let rust_code = format!(
            r"
    #[unsafe(no_mangle)]
    pub {motherfuckingrust} fn bp3d_os_module_{crate_name}_close() {{
        module_close();
    }}
"
        );
        self.rust_code += &rust_code;
        self
    }

    pub fn build(self) {
        let crate_name = self.crate_name;
        std::fs::write(&self.out_path, self.rust_code).unwrap();
        #[cfg(target_vendor = "apple")]
        println!("cargo::rustc-link-arg-cdylib=-Wl,-install_name,@rpath/lib{crate_name}.dylib");
        #[cfg(all(unix, not(target_vendor = "apple")))]
        println!("cargo::rustc-link-arg-cdylib=-Wl,-soname,lib{crate_name}.so");
        println!(
            "cargo:rustc-env=BP3D_OS_MODULE_MAIN={}",
            self.out_path.display()
        );
    }
}

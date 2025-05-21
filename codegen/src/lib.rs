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

use proc_macro::TokenStream;
use cargo_manifest::Manifest;
use itertools::Itertools;
use proc_macro2::{Ident, Span};
use quote::quote;

#[proc_macro]
pub fn module_main(_: TokenStream) -> TokenStream {
    let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();
    let crate_version = std::env::var("CARGO_PKG_VERSION").unwrap();
    let rustc_version = rustc_version::version().unwrap();
    let mod_const_name = format!("BP3D_OS_MODULE_{}", crate_name.to_uppercase());
    let mod_const = Ident::new(&mod_const_name, Span::call_site());
    let package = Manifest::from_path(std::env::var_os("CARGO_MANIFEST_PATH")
        .expect("Failed to get CARGO_MANIFEST_PATH"))
        .expect("Failed to read CARGO_MANIFEST_PATH");
    let deps_list = package.dependencies.map(|v| v.iter()
        .map(|(k, v)| format!("{}={}", k, v.req())).join(",")).unwrap_or("".into());
    let data = format!("BP3D_OS_MODULE|NAME={}|VERSION={}|RUSTC={}|DEPS={}\0", crate_name, crate_version, rustc_version, deps_list);
    let q = quote! {
        #[no_mangle]
        extern "C" static #mod_const: *const std::ffi::c_char = c #data.as_ptr();
    };
    q.into()
}

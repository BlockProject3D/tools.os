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

//! Operating System tools and extensions designed for BlockProject3D.

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(missing_docs)]
// This is unfortunate but objc appears to be generating a lot of these warnings and new versions
// exists. This is a temporary workaround. The solution would most likely be to re-write objc inline
// as crates.io forbids patches.
#![cfg_attr(target_vendor = "apple", allow(unexpected_cfgs))]

#[cfg(feature = "dirs")]
pub mod dirs;

#[cfg(feature = "assets")]
pub mod assets;

#[cfg(feature = "open")]
pub mod open;

#[cfg(feature = "fs")]
pub mod fs;

#[cfg(feature = "cpu-info")]
pub mod cpu_info;

#[cfg(feature = "time")]
pub mod time;

#[cfg(feature = "module")]
pub mod module;

#[cfg(feature = "shell")]
pub mod shell;

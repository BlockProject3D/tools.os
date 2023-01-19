// Copyright (c) 2023, BlockProject 3D
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

//! This module provides cross-platform functions to get application resources.

use std::path::PathBuf;

#[cfg(target_vendor = "apple")]
mod apple;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
mod bsd;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_vendor = "apple")]
use apple::{get_exe_path, get_resources_dir};

#[cfg(target_os = "linux")]
use linux::{get_exe_path, get_resources_dir};

#[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
use bsd::{get_exe_path, get_resources_dir};

#[cfg(target_os = "windows")]
use windows::{get_exe_path, get_resources_dir};

/// Returns the path to an asset of the application.
///
/// # Platform specific behavior
///
/// On supported platforms this returns an asset bundled in the application. Supported platforms are:
/// - Any Linux/Unix when app is packaged as an AppImage,
/// - macOS (when app is packaged as a .app),
/// - iOS
///
/// In the case a platform/packaging method isn't supported this function still returns a path based
/// on executable location.
///
/// Returns None if there is a system issue, ex: the system didn't return a proper path to the current
/// executing application. This should rarely occur.
pub fn get_app_bundled_asset(file_name: &str) -> Option<PathBuf> {
    let res = get_resources_dir().map(|v| v.join(file_name))
        .or_else(|| get_exe_path().map(|v| v.join("Assets").join(file_name)));
    if res.as_ref().map(|v| !v.exists()).unwrap_or(false) {
        return None;
    }
    res
}

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

//! Low-level access to standard system directories.
//!
//! Unsupported directories are returned as None.

use std::path::PathBuf;

#[cfg(target_vendor = "apple")]
mod apple_shared;
#[cfg(target_os = "ios")]
mod ios;
#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "ios", target_os = "android"))
))]
mod unix;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

#[cfg(target_os = "ios")]
use ios as _impl;
#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "ios", target_os = "android"))
))]
use unix as _impl;
#[cfg(target_os = "macos")]
use macos as _impl;
#[cfg(windows)]
use windows as _impl;

pub fn get_app_cache() -> Option<PathBuf> {
    _impl::get_app_cache()
}

pub fn get_app_config() -> Option<PathBuf> {
    _impl::get_app_config()
}

pub fn get_app_data() -> Option<PathBuf> {
    _impl::get_app_data()
}

pub fn get_app_logs() -> Option<PathBuf> {
    _impl::get_app_logs()
}

pub fn get_app_documents() -> Option<PathBuf> {
    _impl::get_app_documents()
}

pub fn get_user_home() -> Option<PathBuf> {
    _impl::get_user_home()
}

pub fn get_user_documents() -> Option<PathBuf> {
    _impl::get_user_documents()
}

pub fn get_user_downloads() -> Option<PathBuf> {
    _impl::get_user_downloads()
}

/*
/// Returns the path to an asset of the application.
///
/// On supported platforms this returns an asset bundled in the application. Supported platforms are:
/// - Any Linux/Unix when app is packaged as an AppImage,
/// - macOS (when app is packaged as a .app),
/// - iOS
///
/// In the case a platform/packaging method isn't supported this function still returns a path based
/// on executable location.
///
/// For macOS and iOS, localization is still supported by the app however assets could also be localized
/// in the app bundle as this function uses Apple APIs (CFBundleCopyResourceURL) to obtain the location
/// of resources.
///
/// Returns None if there is a system issue, ex: the system didn't return a proper path to the current
/// executing application. This should rarely occur.
pub fn get_app_bundled_asset(file_name: &str) -> Option<PathBuf> {
    _impl::get_app_bundled_asset(file_name)
}
*/
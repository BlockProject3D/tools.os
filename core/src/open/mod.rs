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

//! This module provides cross-platform functions to open files, urls and select files in the file
//! explorer.

mod url;
mod error;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "ios")]
mod ios;

#[cfg(all(unix, not(any(target_vendor = "apple", target_os = "android"))))]
mod unix;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
use macos as _impl;

#[cfg(target_os = "ios")]
use ios as _impl;

#[cfg(all(unix, not(any(target_vendor = "apple", target_os = "android"))))]
use unix as _impl;

#[cfg(target_os = "windows")]
use windows as _impl;

pub use url::{InvalidUrl, Url};
pub use error::{Result, Error};

/// Open a file explorer selecting the different files given as iterator.
///
/// Returns true if the operation has succeeded.
///
/// # Platform specific behavior
///
/// - On macOS, this function calls *activateFileViewerSelectingURLs* in *NSWorkspace*.
///   Unfortunately, the ObjectiveC function relies on the presence of NSRunLoop, as such,
///   *show_in_files* will also call *runUntilDate* in *NSRunLoop* which also means that
///   **this function will return false if called from a different thread than the main thread**.
///
/// - On iOS, this function always returns false because there is no matching functionality in UIKit.
///
/// - On Windows, this function always returns false because WinAPI doesn't have a matching
///   equivalent.
///
/// - On Linux and most other unix systems, this function attempts to call the dbus function
///   *ShowItems* in *org.freedesktop.FileManager1*. If no dbus connection could be made this
///   function returns false.
///
///   **Note: Not all file explorers are created equal under Linux, so the behavior of this
///   function depends on the file explorer.**
pub fn show_in_files<'a, I: Iterator<Item = &'a std::path::Path>>(iter: I) -> Result<()> {
    _impl::show_in_files(iter)
}

/// Opens an URL using the default associated app for the URL scheme.
///
/// Returns true if the operation has succeeded.
///
/// # Platform specific behavior
///
/// - On macOS, this function calls *openURL* in *NSWorkspace*.
///
/// - On iOS, this function currently returns false.
///
/// - On Windows, this function calls *ShellExecuteW* with the "open" operation.
///
/// - On Linux and most other unix systems, this function calls the dbus function *ShowFolders* in
///   *org.freedesktop.FileManager1* when the URL is a path to a directory, otherwise the function
///   attempts to execute the *xdg-open* command line tool with the URL string as argument.
///
/// # Arguments
///
/// * `url`: the URL to open.
///
/// returns: bool
pub fn open<'a, T: Into<Url<'a>>>(url: T) -> Result<()> {
    _impl::open(&url.into())
}

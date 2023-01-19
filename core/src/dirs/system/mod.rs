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

/// Returns the user's cache directory where all applications should store cached data.
///
/// # Platform specific behavior
///
/// | System               | Directory Name        | Usual path                                    |
/// |----------------------|-----------------------|-----------------------------------------------|
/// | macOS                | NS_CACHES_DIRECTORY   | ~/Library/Caches                              |
/// | macOS (with sandbox) | NS_CACHES_DIRECTORY   | ~/Library/Containers/{ID}/Data/Library/Caches |
/// | iOS                  | NS_CACHES_DIRECTORY   | Not applicable                                |
/// | Linux                | XDG_CACHE_HOME        | ~/.cache                                      |
/// | Windows              | FOLDERID_LocalAppData | %APPDATA%/Local                               |
pub fn get_app_cache() -> Option<PathBuf> {
    _impl::get_app_cache()
}

/// Returns the user's config directory where all applications should store configurations.
///
/// # Platform specific behavior
///
/// | System               | Directory Name                   | Usual path                                         |
/// |----------------------|----------------------------------|----------------------------------------------------|
/// | macOS                | NS_LIBRARY_DIRECTORY/Preferences | ~/Library/Preferences                              |
/// | macOS (with sandbox) | NS_LIBRARY_DIRECTORY/Preferences | ~/Library/Containers/{ID}/Data/Library/Preferences |
/// | iOS                  | NS_LIBRARY_DIRECTORY/Preferences | Not applicable                                     |
/// | Linux                | XDG_CONFIG_HOME                  | ~/.config                                          |
/// | Windows              | None                             | None                                               |
pub fn get_app_config() -> Option<PathBuf> {
    _impl::get_app_config()
}

/// Returns the user's config directory where all applications should store configurations.
///
/// # Platform specific behavior
///
/// | System               | Directory Name                   | Usual path                                                 |
/// |----------------------|----------------------------------|------------------------------------------------------------|
/// | macOS                | NS_APPLICATION_SUPPORT_DIRECTORY | ~/Library/Application Support                              |
/// | macOS (with sandbox) | NS_APPLICATION_SUPPORT_DIRECTORY | ~/Library/Containers/{ID}/Data/Library/Application Support |
/// | iOS                  | NS_APPLICATION_SUPPORT_DIRECTORY | Not applicable                                             |
/// | Linux                | XDG_DATA_HOME                    | ~/.local/share                                             |
/// | Windows              | FOLDERID_RoamingAppData          | %APPDATA%/Roaming                                          |
pub fn get_app_data() -> Option<PathBuf> {
    _impl::get_app_data()
}

/// Returns the user's log directory where all applications should store logs.
///
/// # Platform specific behavior
///
/// | System               | Directory Name            | Usual path                                  |
/// |----------------------|---------------------------|---------------------------------------------|
/// | macOS                | NS_LIBRARY_DIRECTORY/Logs | ~/Library/Logs                              |
/// | macOS (with sandbox) | NS_LIBRARY_DIRECTORY/Logs | ~/Library/Containers/{ID}/Data/Library/Logs |
/// | iOS                  | None                      | None                                        |
/// | Linux                | None                      | None                                        |
/// | Windows              | FOLDERID_RoamingAppData   | None                                        |
pub fn get_app_logs() -> Option<PathBuf> {
    _impl::get_app_logs()
}

/// Returns the public documents directory for this application.
///
/// **NOTE: This directory is already unique to this application unlike other directories.**
///
/// # Platform specific behavior
///
/// | System               | Directory Name        | Usual path                               |
/// |----------------------|-----------------------|------------------------------------------|
/// | macOS                | None                  | None                                     |
/// | macOS (with sandbox) | NS_DOCUMENT_DIRECTORY | ~/Library/Containers/{ID}/Data/Documents |
/// | iOS                  | NS_DOCUMENT_DIRECTORY | Not applicable                           |
/// | Linux                | None                  | None                                     |
/// | Windows              | None                  | None                                     |
pub fn get_app_documents() -> Option<PathBuf> {
    _impl::get_app_documents()
}

/// Returns the user's home directory.
///
/// # Platform specific behavior
///
/// | System               | Directory Name    | Usual path          |
/// |----------------------|-------------------|---------------------|
/// | macOS                | NS_USER_DIRECTORY | /Users/{username}   |
/// | macOS (with sandbox) | NS_USER_DIRECTORY | /Users/{username}   |
/// | iOS                  | None              | None                |
/// | Linux                | HOME              | /home/{username}    |
/// | Windows              | FOLDERID_Profile  | C:\Users\{username} |
pub fn get_user_home() -> Option<PathBuf> {
    _impl::get_user_home()
}

/// Returns the user's documents directory.
///
/// # Platform specific behavior
///
/// | System               | Directory Name        | Usual path                    |
/// |----------------------|-----------------------|-------------------------------|
/// | macOS                | NS_DOCUMENT_DIRECTORY | /Users/{username}/Documents   |
/// | macOS (with sandbox) | None                  | None                          |
/// | iOS                  | None                  | None                          |
/// | Linux                | XDG_DOCUMENTS_DIR     | /home/{username}/Documents    |
/// | Windows              | FOLDERID_Documents    | C:\Users\{username}\Documents |
pub fn get_user_documents() -> Option<PathBuf> {
    _impl::get_user_documents()
}

/// Returns the user's downloads directory.
///
/// # Platform specific behavior
///
/// | System               | Directory Name         | Usual path                  |
/// |----------------------|------------------------|-----------------------------|
/// | macOS                | NS_DOWNLOADS_DIRECTORY | /Users/{username}/Downloads |
/// | macOS (with sandbox) | None                   | None                        |
/// | iOS                  | None                   | None                        |
/// | Linux                | XDG_DOWNLOAD_DIR       | /home/{username}            |
/// | Windows              | FOLDERID_Downloads     | C:\Users\{username}         |
pub fn get_user_downloads() -> Option<PathBuf> {
    _impl::get_user_downloads()
}

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

use crate::dirs::system::apple_shared::{
    get_macos_dir, get_macos_dir_fail_if_sandbox, NS_APPLICATION_SUPPORT_DIRECTORY,
    NS_CACHES_DIRECTORY, NS_DOCUMENT_DIRECTORY, NS_DOWNLOADS_DIRECTORY, NS_LIBRARY_DIRECTORY,
    NS_USER_DIRECTORY,
};
use std::path::PathBuf;

pub fn get_app_cache() -> Option<PathBuf> {
    get_macos_dir(NS_CACHES_DIRECTORY).map(PathBuf::from)
}

pub fn get_app_config() -> Option<PathBuf> {
    get_macos_dir(NS_LIBRARY_DIRECTORY).map(|path| PathBuf::from(path).join("Preferences"))
}

pub fn get_app_data() -> Option<PathBuf> {
    get_macos_dir(NS_APPLICATION_SUPPORT_DIRECTORY).map(PathBuf::from)
}

pub fn get_app_logs() -> Option<PathBuf> {
    get_macos_dir(NS_LIBRARY_DIRECTORY).map(|path| PathBuf::from(path).join("Logs"))
}

pub fn get_app_documents() -> Option<PathBuf> {
    if let Some(dir) = get_macos_dir(NS_DOCUMENT_DIRECTORY) {
        if dir.contains("Library/Containers/") {
            //Running in a sandbox
            Some(PathBuf::from(dir))
        } else {
            None
        }
    } else {
        None
    }
}

pub fn get_user_home() -> Option<PathBuf> {
    get_macos_dir(NS_USER_DIRECTORY).map(PathBuf::from)
}

pub fn get_user_documents() -> Option<PathBuf> {
    get_macos_dir_fail_if_sandbox(NS_DOCUMENT_DIRECTORY)
}

pub fn get_user_downloads() -> Option<PathBuf> {
    get_macos_dir_fail_if_sandbox(NS_DOWNLOADS_DIRECTORY)
}

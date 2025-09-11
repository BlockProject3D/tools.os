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

#![allow(dead_code)] //Allow unused functions and constants to stop rust complaining on iOS.

use objc2::class;
use std::os::raw::c_ulong;
use std::path::PathBuf;

use crate::apple_helpers::{msg_send, ns_string_to_string};
use crate::apple_helpers::__msg_send_parse;
use crate::apple_helpers::{Object};

pub const NS_LIBRARY_DIRECTORY: c_ulong = 5;
pub const NS_USER_DIRECTORY: c_ulong = 7;
pub const NS_DOCUMENT_DIRECTORY: c_ulong = 9;
pub const NS_CACHES_DIRECTORY: c_ulong = 13;
pub const NS_APPLICATION_SUPPORT_DIRECTORY: c_ulong = 14;
pub const NS_DOWNLOADS_DIRECTORY: c_ulong = 15;

const NS_USER_DOMAIN_MASK: c_ulong = 1;

pub fn get_macos_dir(directory: c_ulong) -> Option<String> {
    unsafe {
        let nsfilemanager = class!(NSFileManager);
        let instance: &Object = msg_send![nsfilemanager, defaultManager];
        let directories: &Object = msg_send![instance, URLsForDirectory:directory inDomains:NS_USER_DOMAIN_MASK];
        let obj: Option<&Object> = msg_send![directories, firstObject];
        if let Some(obj) = obj {
            let str: Option<&Object> = msg_send![obj, path];
            match str {
                Some(v) => Some(String::from(ns_string_to_string(v))),
                None => None
            }
        } else {
            None
        }
    }
}

pub fn get_macos_dir_fail_if_sandbox(directory: c_ulong) -> Option<PathBuf> {
    if let Some(dir) = get_macos_dir(directory) {
        if dir.contains("Library/Containers/") {
            //Running in a sandbox
            None
        } else {
            Some(PathBuf::from(dir))
        }
    } else {
        None
    }
}

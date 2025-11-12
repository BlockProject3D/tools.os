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

use crate::apple_helpers::__msg_send_parse;
use crate::apple_helpers::msg_send;
use crate::apple_helpers::{ns_string_to_string, Object};
use libc::{strlen, PATH_MAX};
use objc2::class;
use std::ffi::{c_char, c_int, OsStr};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

extern "C" {
    pub fn _NSGetExecutablePath(buf: *mut c_char, bufsize: *mut u32) -> c_int;
}

pub fn get_exe_path() -> Option<PathBuf> {
    let mut buf: [c_char; PATH_MAX as usize] = [0; PATH_MAX as usize];
    let mut size: u32 = PATH_MAX as u32;
    unsafe {
        let res = _NSGetExecutablePath(&mut buf as _, &mut size as _);
        if res == -1 {
            //path is too large
            let mut v = Vec::with_capacity(size as usize);
            let res = _NSGetExecutablePath(v.as_mut_ptr(), &mut size as _);
            if res != 0 {
                //Something really bad happened.
                return None;
            }
            let str = OsStr::from_bytes(std::mem::transmute(&v[..size as usize]));
            return PathBuf::from(str).parent().map(|v| v.into());
        }
        if res != 0 {
            return None;
        }
        let len = strlen(buf.as_ptr());
        let str = OsStr::from_bytes(std::mem::transmute(&buf[..len]));
        PathBuf::from(str).parent().map(|v| v.into())
    }
}

pub fn get_resources_dir() -> Option<PathBuf> {
    unsafe {
        let nsbundle = class!(NSBundle);
        let bundle: Option<&Object> = msg_send![nsbundle, mainBundle];
        let bundle = bundle?;
        let str: Option<&Object> = msg_send![bundle, resourcePath];
        let str = str?;
        Some(PathBuf::from(ns_string_to_string(str)))
    }
}

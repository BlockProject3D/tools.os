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

use libc::{strlen, PATH_MAX};
use std::ffi::{c_char, c_int, OsStr};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use objc::{class, msg_send, sel, sel_impl};
use objc::runtime::Object;
use objc_foundation::{INSString, NSString};

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
            return Some(PathBuf::from(str));
        }
        if res != 0 {
            return None;
        }
        let len = strlen(buf.as_ptr());
        let str = OsStr::from_bytes(std::mem::transmute(&buf[..len as usize]));
        Some(PathBuf::from(str))
    }
}

pub fn get_resources_dir() -> Option<PathBuf> {
    unsafe {
        let nsbundle = class!(NSBundle);
        let bundle: *mut Object = msg_send![nsbundle, mainBundle];
        if bundle.is_null() {
            return None;
        }
        let str: *const NSString = msg_send![bundle, resourcePath];
        if str.is_null() {
            return None;
        }
        Some(PathBuf::from((*str).as_str()))
    }
}

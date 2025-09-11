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

use crate::open::{Error, Result, Url};
use objc2::class;
use std::ffi::{c_char, c_double};
use std::os::raw::c_ulong;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use objc2::rc::Retained;

use crate::apple_helpers::msg_send;
use crate::apple_helpers::__msg_send_parse;
use crate::apple_helpers::obj_alloc;
use crate::apple_helpers::obj_from;
use crate::apple_helpers::{Object, NO, BOOL};

const NS_UTF8_STRING_ENCODING: c_ulong = 4;

// Force link against AppKit on mac
#[link(name = "AppKit", kind = "framework")]
extern "C" {}

pub fn open(url: &Url) -> Result {
    let url_str = url.to_os_str().map_err(Error::Io)?;
    unsafe {
        let nsworkspace = class!(NSWorkspace);
        let bytes = url_str.as_bytes().as_ptr() as *const c_char;
        let str = obj_alloc![NSString, initWithBytes: bytes length: url_str.len() as c_ulong encoding: NS_UTF8_STRING_ENCODING];
        let url = obj_from![NSURL, URLWithString: &*str];
        let workspace: &Object = msg_send![nsworkspace, sharedWorkspace];
        let res: BOOL = msg_send![workspace, openURL: &*url];
        match res == NO {
            true => Err(Error::Other("failed to open url".into())),
            false => Ok(()),
        }
    }
}

pub fn show_in_files<'a, I: Iterator<Item = &'a Path>>(iter: I) -> Result {
    let nsthread = class!(NSThread);
    let nsrunloop = class!(NSRunLoop);
    let nsworkspace = class!(NSWorkspace);
    unsafe {
        let flag: BOOL = msg_send![nsthread, isMainThread];
        if flag == NO {
            return Err(Error::Other("current thread is not the main thread".into()));
        }
    }
    let v: std::io::Result<Vec<Retained<Object>>> = iter.map(|v| crate::fs::get_absolute_path(v).map(|v| {
        unsafe {
            let str = obj_alloc![NSString, initWithBytes: v.as_os_str().as_bytes().as_ptr() length: v.as_os_str().len() as c_ulong encoding: NS_UTF8_STRING_ENCODING];
            obj_from![NSURL, fileURLWithPath: &*str]
        }
    })).collect();
    let urls = v.map_err(Error::Io)?;
    unsafe {
        let arr = obj_from![NSArray, arrayWithObjects: urls.as_ptr() as *const Object count: urls.len() as c_ulong];
        let workspace: &Object = msg_send![nsworkspace, sharedWorkspace];
        let _: () = msg_send![workspace, activateFileViewerSelectingURLs: &*arr];
        //Create a date of 100ms in the future
        let runloop: &Object = msg_send![nsrunloop, mainRunLoop];
        let date = obj_from![NSDate, dateWithTimeIntervalSinceNow: 0.1 as c_double];
        let _: () = msg_send![runloop, runUntilDate: &*date];
    }
    Ok(())
}

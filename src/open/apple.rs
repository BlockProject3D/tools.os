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

use std::os::unix::ffi::OsStrExt;
use std::os::raw::c_ulong;
use objc::class;
use objc::msg_send;
use objc::sel;
use objc::sel_impl;
use objc::runtime::Object;
use std::path::Path;
use crate::fs::PathExt;
use crate::open::Url;

const NS_UTF8_STRING_ENCODING: c_ulong = 4;

pub fn open(url: &Url) -> bool {
    let url_str = match url.to_os_str() {
        Ok(v) => v,
        Err(_) => return false
    };
    unsafe {
        let nsstring = class!(NSString);
        let nsurl = class!(NSURL);
        let nsworkspace = class!(NSWorkspace);
        let mut str: *mut Object = msg_send![nsstring, alloc];
        str = msg_send![str, initWithBytes: url_str.as_bytes().as_ptr() length: url_str.len() as c_ulong encoding: NS_UTF8_STRING_ENCODING];
        let url: *mut Object = msg_send![nsurl, URLWithString: str];
        let workspace: *mut Object = msg_send![nsworkspace, sharedWorkspace];
        let _: () = msg_send![workspace, openURL: url];
        // release objects
        // do not release url as it's still owned by Foundation
        let _: () = msg_send![str, release]; // release string (we used alloc)
        true
    }
}

pub fn show_in_files<'a, I: Iterator<Item = &'a Path>>(iter: I) -> bool {
    let nsarray = class!(NSArray);
    let nsworkspace = class!(NSWorkspace);
    let nsstring = class!(NSString);
    let nsurl = class!(NSURL);
    let v: std::io::Result<Vec<*mut Object>> = iter.map(|v| v.get_absolute().map(|v| {
        unsafe {
            let mut str: *mut Object = msg_send![nsstring, alloc];
            str = msg_send![str, initWithBytes: v.as_os_str().as_bytes().as_ptr() length: v.as_os_str().len() as c_ulong encoding: NS_UTF8_STRING_ENCODING];
            let url = msg_send![nsurl, URLWithString: str];
            //release string so that we don't have to do it later
            //TODO: Check if URLWithString keeps a strong reference to the underlying NSString or
            // even copies it
            let _: () = msg_send![str, release];
            url
        }
    })).collect();
    let urls = match v {
        Ok(v) => v,
        Err(_) => return false
    };
    unsafe {
        let arr: *mut Object = msg_send![nsarray, arrayWithObjects: urls.as_ptr() count: urls.len() as c_ulong];
        let workspace: *mut Object = msg_send![nsworkspace, sharedWorkspace];
        let _: () = msg_send![workspace, activateFileViewerSelectingURLs: arr];
    }
    // do not release the urls or the array because they're still owned by Foundation (no alloc called)
    true
}

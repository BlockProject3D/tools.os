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

use crate::fs::PathExt;
use crate::open::{Error, Result, Url};
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows_sys::core::PCWSTR;
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOW;

pub fn open(url: &Url) -> Result {
    unsafe {
        let operation = ['o' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 0x0000];
        let mut urlw: Vec<u16> = match url.is_path() {
            true => Path::new(url.path())
                .get_absolute()
                .map_err(Error::Io)?
                .as_os_str()
                .encode_wide()
                .collect(),
            false => url.to_os_str().map_err(Error::Io)?.encode_wide().collect(),
        };
        urlw.push(0x0000);
        let operation: PCWSTR = operation.as_ptr();
        let res = ShellExecuteW(
            0,
            operation,
            urlw.as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            SW_SHOW as _,
        );
        match res > 32 {
            true => Ok(()),
            false => Err(Error::Io(std::io::Error::last_os_error())),
        }
    }
}

pub fn show_in_files<'a, I: Iterator<Item = &'a Path>>(_: I) -> Result {
    Err(Error::Unsupported)
}

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

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use windows_sys::Win32::Foundation::{GetLastError, ERROR_INSUFFICIENT_BUFFER, MAX_PATH};
use windows_sys::Win32::System::LibraryLoader::GetModuleFileNameW;

pub fn get_exe_path() -> Option<PathBuf> {
    unsafe {
        //Try fast path with MAX_PATH which should work for most windows versions.
        let mut buf: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let res = GetModuleFileNameW(std::ptr::null_mut(), &mut buf as _, MAX_PATH);
        if res == 0 {
            return None; //System error.
        } else if res == MAX_PATH {
            //We might have a problem where the buffer is not large enough...
            let err = GetLastError();
            if err == ERROR_INSUFFICIENT_BUFFER {
                //We definitely have a buffer length problem!
                let mut len = MAX_PATH as usize * 2;
                loop {
                    //Start allocating twice buffer size.
                    let mut v = Vec::with_capacity(len);
                    //Attempt reading module file name again.
                    let res = GetModuleFileNameW(std::ptr::null_mut(), v.as_mut_ptr(), len as u32);
                    if res == 0 {
                        return None; //System error.
                    } else if res == len as u32 {
                        let err = GetLastError();
                        if err != ERROR_INSUFFICIENT_BUFFER {
                            break;
                        }
                    } else {
                        break;
                    }
                    //If this reaches, well it's still not looking good, and we need more re-allocations.
                    len *= 2;
                }
            }
        }
        //We finally found the executable file name!
        let str1 = OsString::from_wide(&buf[..res as usize]);
        Some(PathBuf::from(str1).parent()?.into())
    }
}

pub fn get_resources_dir() -> Option<PathBuf> {
    None
}

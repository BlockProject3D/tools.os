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

use libc::size_t;
use libc::strlen;
use libc::sysctl;
use libc::CTL_KERN;
use libc::KERN_PROC;
use libc::KERN_PROC_PATHNAME;
use libc::PATH_MAX;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

pub fn get_exe_path() -> Option<PathBuf> {
    let mut mib = [CTL_KERN, KERN_PROC, KERN_PROC_PATHNAME, -1];
    let mut buf: Vec<u8> = Vec::with_capacity(PATH_MAX);
    let mut cb: size_t = PATH_MAX;
    unsafe {
        let res = sysctl(
            mib.as_mut_ptr(),
            4,
            buf.as_mut_ptr() as *mut _,
            &mut cb as _,
            std::ptr::null_mut(),
            0,
        );
        if res == 0 {
            //FreeBSD without procfs.
            let len = strlen(buf.as_ptr() as _);
            //This is where we defer from process_path: we use std::os::unix::ffi::OsStrExt.
            let str = OsStr::from_bytes(&buf[..len]);
            let path = PathBuf::from(str);
            path.parent().map(|v| v.into())
        } else {
            //FreeBSD with procfs.
            std::fs::read_link("/proc/curproc/file")
                .ok()
                .map(|v| v.parent().map(PathBuf::from))
                .flatten()
        }
    }
}

pub fn get_resources_dir() -> Option<PathBuf> {
    None
}

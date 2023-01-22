// Copyright (c) 2022, BlockProject 3D
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

use crate::fs::PathUpdate;
use std::io::{Error, ErrorKind, Result};
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use windows_sys::Win32::Storage::FileSystem::GetFileAttributesW;
use windows_sys::Win32::Storage::FileSystem::SetFileAttributesW;
use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_HIDDEN;
use windows_sys::Win32::Storage::FileSystem::INVALID_FILE_ATTRIBUTES;

pub fn hide<T: AsRef<Path>>(r: T) -> Result<PathUpdate<T>> {
    let path = r.as_ref();
    if !path.exists() {
        return Err(Error::new(ErrorKind::NotFound, "file or directory found"));
    }
    let mut file: Vec<u16> = path.as_os_str().encode_wide().collect();
    file.push(0x0000);
    unsafe {
        let attrs = GetFileAttributesW(file.as_ptr());
        if attrs == INVALID_FILE_ATTRIBUTES {
            return Err(Error::last_os_error());
        }
        if SetFileAttributesW(file.as_ptr(), attrs | FILE_ATTRIBUTE_HIDDEN) == 0 {
            Err(Error::last_os_error())
        } else {
            Ok(PathUpdate::Unchanged(r))
        }
    }
}

pub fn show<T: AsRef<Path>>(r: T) -> Result<PathUpdate<T>> {
    let path = r.as_ref();
    if !path.exists() {
        return Err(Error::new(ErrorKind::NotFound, "file or directory found"));
    }
    let mut file: Vec<u16> = path.as_os_str().encode_wide().collect();
    file.push(0x0000);
    unsafe {
        let attrs = GetFileAttributesW(file.as_ptr());
        if attrs == INVALID_FILE_ATTRIBUTES {
            return Err(Error::last_os_error());
        }
        if SetFileAttributesW(file.as_ptr(), attrs & !FILE_ATTRIBUTE_HIDDEN) == 0 {
            Err(Error::last_os_error())
        } else {
            Ok(PathUpdate::Unchanged(r))
        }
    }
}

pub fn get_absolute_path<T: AsRef<Path>>(path: T) -> Result<PathBuf> {
    dunce::canonicalize(path)
}

pub fn is_hidden<T: AsRef<Path>>(path: T) -> bool {
    let path = path.as_ref();
    if !path.exists() {
        return false;
    }
    let mut file: Vec<u16> = path.as_os_str().encode_wide().collect();
    file.push(0x0000);
    unsafe {
        let attrs = GetFileAttributesW(file.as_ptr());
        if attrs == INVALID_FILE_ATTRIBUTES {
            return false;
        }
        if attrs & FILE_ATTRIBUTE_HIDDEN == 0 {
            return true;
        }
    }
    false
}

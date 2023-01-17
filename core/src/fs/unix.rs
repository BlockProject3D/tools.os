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

use std::io::{Error, ErrorKind, Result};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// Hides the given path in the current platform's file explorer.
///
/// # Arguments
///
/// * `path`: the path to convert.
///
/// returns: Result<(), Error>
///
/// # Errors
///
/// Returns an [Error](Error) if the path couldn't be hidden.
pub fn hide<T: AsRef<Path>>(path: T) -> Result<()> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(Error::new(ErrorKind::NotFound, "file or directory found"));
    }
    if let Some(str) = path.file_name() {
        let bytes = str.as_bytes();
        if bytes[0] == b'.' {
            return Ok(()); //path is already hidden.
        }
        let mut vec = bytes.to_vec();
        vec.insert(0, b'.');
        let mut copy: PathBuf = path.into();
        copy.set_file_name(OsStr::from_bytes(&vec));
        return std::fs::rename(path, copy);
    }
    Err(Error::new(ErrorKind::InvalidInput, "the path does not have a file name"))
}

/// Un-hides the given path in the current platform's file explorer.
///
/// # Arguments
///
/// * `path`: the path to convert.
///
/// returns: Result<(), Error>
///
/// # Errors
///
/// Returns an [Error](Error) if the path couldn't be un-hidden.
pub fn unhide<T: AsRef<Path>>(path: T) -> Result<()> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(Error::new(ErrorKind::NotFound, "file or directory found"));
    }
    if let Some(str) = path.file_name() {
        let bytes = str.as_bytes();
        if bytes[0] != b'.' {
            return Ok(()); //path is already visible.
        }
        let mut vec = bytes.to_vec();
        vec.remove(0); //remove the '.' character from the file name.
        let mut copy: PathBuf = path.into();
        copy.set_file_name(OsStr::from_bytes(&vec));
        return std::fs::rename(path, copy);
    }
    Err(Error::new(ErrorKind::InvalidInput, "the path does not have a file name"))
}

/// Converts a path to an absolute path.
///
/// # Arguments
///
/// * `path`: the path to convert.
///
/// returns: Result<PathBuf, Error>
///
/// # Errors
///
/// Returns an [Error](Error) if the path couldn't be converted to an absolute path.
pub fn get_absolute_path<T: AsRef<Path>>(path: T) -> Result<PathBuf> {
    std::fs::canonicalize(path)
}

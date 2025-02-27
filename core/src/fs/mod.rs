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

//! This module provides cross-platform functions to hide, unhide files, manage file extensions and
//! get the most compatible absolute path of a file.

#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod windows;

#[cfg(unix)]
use unix as _impl;

#[cfg(windows)]
use windows as _impl;

/// The result of hide and show.
pub enum PathUpdate<T: AsRef<std::path::Path>> {
    /// Indicates that the source path was changed.
    Changed(std::path::PathBuf),

    /// Indicates that the source path was returned as-is with no changes.
    Unchanged(T),
}

impl<T: AsRef<std::path::Path>> AsRef<std::path::Path> for PathUpdate<T> {
    fn as_ref(&self) -> &std::path::Path {
        match self {
            PathUpdate::Changed(v) => v.as_ref(),
            PathUpdate::Unchanged(v) => v.as_ref(),
        }
    }
}

impl<T: AsRef<std::path::Path>> std::ops::Deref for PathUpdate<T> {
    type Target = std::path::Path;

    fn deref(&self) -> &std::path::Path {
        self.as_ref()
    }
}

/// Converts a path to an absolute path.
///
/// NOTE: Unlike [canonicalize](std::fs::canonicalize) the paths returned by this function may not
/// always be normalized.
///
/// # Platform specific behavior
///
/// - On Unix, this function redirects to [canonicalize](std::fs::canonicalize).
///
/// - On Windows, contrary to [canonicalize](std::fs::canonicalize) which always normalizes the
///   input path to UNC, this function will try it's best to avoid using UNC paths which aren't
///   supported by all applications, including some built-in applications. Currently, the function
///   calls the *GetFullPathNameW* API.
///
/// # Arguments
///
/// * `path`: the path to convert.
///
/// returns: Result<PathBuf, Error>
///
/// # Errors
///
/// Returns an [Error](std::io::Error) if the path couldn't be converted to an absolute path.
pub fn get_absolute_path<T: AsRef<std::path::Path>>(
    path: T,
) -> std::io::Result<std::path::PathBuf> {
    _impl::get_absolute_path(path)
}

/// Checks if a given path is hidden.
///
/// # Platform specific behavior
///
/// - On Unix, this function returns true when the given path has a '.' prefix.
///
/// - On Windows, this function return true when GetFileAttributesW succeeds and that the file
///   attributes contains the attribute *FILE_ATTRIBUTE_HIDDEN*.
///
/// # Arguments
///
/// * `path`: the path to check.
///
/// returns: bool
pub fn is_hidden<T: AsRef<std::path::Path>>(path: T) -> bool {
    _impl::is_hidden(path)
}

/// Hides the given path in the current platform's file explorer.
///
/// # Platform specific behavior
///
/// - On Unix, this function prefixes the path with a '.' and returns [Changed](PathUpdate::Changed)
///   if it does not already have one. If the path already has the prefix, the function returns
///   [Unchanged](PathUpdate::Unchanged).
///
/// - On Windows, this function calls *GetFileAttributesW* and *SetFileAttributesW* with the
///   *FILE_ATTRIBUTE_HIDDEN* attribute. Because windows uses file attributes to define if a
///   file should be visible, the function always returns [Unchanged](PathUpdate::Unchanged).
///
/// # Arguments
///
/// * `path`: the path to convert.
///
/// returns: Result<(), Error>
///
/// # Errors
///
/// Returns an [Error](std::io::Error) if the path couldn't be hidden.
pub fn hide<T: AsRef<std::path::Path>>(path: T) -> std::io::Result<PathUpdate<T>> {
    _impl::hide(path)
}

/// Shows the given path in the current platform's file explorer.
///
/// # Platform specific behavior
///
/// - On Unix, this function removes the '.' prefix from the given path and returns
///   [Changed](PathUpdate::Changed) if it does have it. If the path does not already has the
///   prefix, the function returns [Unchanged](PathUpdate::Unchanged).
///
/// - On Windows, this function calls *GetFileAttributesW* and *SetFileAttributesW* and removes the
///   *FILE_ATTRIBUTE_HIDDEN* attribute. Because windows uses file attributes to define if a file
///   should be visible, the function always returns [Unchanged](PathUpdate::Unchanged).
///
/// # Arguments
///
/// * `path`: the path to convert.
///
/// returns: Result<(), Error>
///
/// # Errors
///
/// Returns an [Error](std::io::Error) if the path couldn't be un-hidden.
pub fn show<T: AsRef<std::path::Path>>(path: T) -> std::io::Result<PathUpdate<T>> {
    _impl::show(path)
}

/// Copy options.
#[derive(Default)]
pub struct CopyOptions<'a> {
    overwrite: bool,
    excludes: Vec<&'a std::ffi::OsStr>
}

impl<'a> CopyOptions<'a> {
    /// Creates a new default filled instance of [CopyOptions].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether overwriting existing files is accepted.
    /// The default is to not allow overwriting files.
    ///
    /// # Arguments
    ///
    /// * `overwrite`: true to allow overwriting files, false otherwise.
    ///
    /// returns: &mut CopyOptions
    pub fn overwrite(&mut self, overwrite: bool) -> &mut Self {
        self.overwrite = overwrite;
        self
    }

    /// Adds a file name or folder name to be excluded from the copy operation.
    ///
    /// # Arguments
    ///
    /// * `name`: the file or folder name to exclude.
    ///
    /// returns: &mut CopyOptions
    pub fn exclude(&mut self, name: &'a std::ffi::OsStr) -> &mut Self {
        self.excludes.push(name);
        self
    }
}

/// Copy a file or a folder.
///
/// # Usage
///
/// | src  |  dst | result                                         |
/// | ---- | ---- | ---------------------------------------------- |
/// | file | file | copy src into dst using [copy](std::fs::copy). |
/// | file | dir  | copy src into dst/file_name.                   |
/// | dir  | file | error.                                         |
/// | dir  | dir  | deep copy of the content of src into dst.      |
///
/// # Arguments
///
/// * `src`:
/// * `dst`:
///
/// returns: Result<(), Error>
pub fn copy<'a>(src: &std::path::Path, dst: &std::path::Path, options: impl std::borrow::Borrow<CopyOptions<'a>>) -> std::io::Result<()> {
    let options = options.borrow();
    if src.file_name().map(|v| options.excludes.contains(&v)).unwrap_or(false) {
        // No error but file is to be excluded so don't copy.
        return Ok(());
    }
    if src.is_file() {
        if dst.is_dir() {
            let name = src.file_name().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid source file"))?;
            return copy(src, &dst.join(name), options);
        } else {
            if !options.overwrite {
                return Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "overwriting files is not allowed"))
            }
            return std::fs::copy(src, dst).map(|_| ());
        }
    }
    if dst.is_file() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotADirectory, "a directory is needed to copy a directory"));
    }
    if !dst.exists() {
        std::fs::create_dir(dst)?;
    }
    for v in std::fs::read_dir(src)? {
        let entry = v?;
        copy(&entry.path(), &dst.join(entry.file_name()), options)?;
    }
    Ok(())
}

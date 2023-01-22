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
    Unchanged(T)
}

impl<T: AsRef<std::path::Path>> AsRef<std::path::Path> for PathUpdate<T> {
    fn as_ref(&self) -> &std::path::Path {
        match self {
            PathUpdate::Changed(v) => v.as_ref(),
            PathUpdate::Unchanged(v) => v.as_ref()
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
/// # Platform specific behavior
///
/// - On Unix, this function redirects to [canonicalize](std::fs::canonicalize).
///
/// - On Windows, contrary to [canonicalize](std::fs::canonicalize) which always uses UNC paths
///   to ensure the highest possible breakage with other Windows applications, this function will
///   try it's best to avoid using UNC paths which aren't supported by all applications. Currently,
///   the function redirects to the *dunce* library.
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
pub fn get_absolute_path<T: AsRef<std::path::Path>>(path: T) -> std::io::Result<std::path::PathBuf> {
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
/// Returns an [Error](Error) if the path couldn't be hidden.
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
/// Returns an [Error](Error) if the path couldn't be un-hidden.
pub fn show<T: AsRef<std::path::Path>>(path: T) -> std::io::Result<PathUpdate<T>> {
    _impl::show(path)
}

mod sealing {
    pub trait Sealed {}
    impl Sealed for std::path::Path {}
}
use sealing::Sealed;

/// Extension trait for [Path](std::path::Path) for common functionality in BP3D software.
pub trait PathExt: Sealed {
    /// Ensures the given extension is present on a [Path](std::path::Path). Reallocates a new
    /// [PathBuf](std::path::PathBuf) if no extension is present or that the extension is incorrect.
    fn ensure_extension<S: AsRef<std::ffi::OsStr>>(
        &self,
        extension: S,
    ) -> std::borrow::Cow<std::path::Path>;

    /// Converts this path to an absolute path.
    ///
    /// On Windows, this function will try it's best to avoid using UNC paths which aren't
    /// supported by all applications.
    ///
    /// returns: Result<PathBuf, Error>
    ///
    /// # Errors
    ///
    /// Returns an [Error](std::io::Error) if the path couldn't be converted to an absolute path.
    fn get_absolute(&self) -> std::io::Result<std::path::PathBuf>;

    /// Checks if this path is hidden in the current platform's file explorer.
    fn is_hidden(&self) -> bool;
}

impl PathExt for std::path::Path {
    fn ensure_extension<S: AsRef<std::ffi::OsStr>>(
        &self,
        extension: S,
    ) -> std::borrow::Cow<std::path::Path> {
        if let Some(ext) = self.extension() {
            if ext == extension.as_ref() {
                self.into()
            } else {
                let mut buf = self.to_path_buf();
                buf.set_extension(extension);
                buf.into()
            }
        } else {
            self.with_extension(extension).into()
        }
    }

    fn get_absolute(&self) -> std::io::Result<std::path::PathBuf> {
        get_absolute_path(self)
    }

    fn is_hidden(&self) -> bool {
        is_hidden(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::fs::PathExt;
    use std::borrow::Cow;
    use std::path::Path;

    #[test]
    fn basic() {
        let wrong_ext = Path::new("myfile.txt");
        let no_ext = Path::new("myfile");
        let correct_ext = Path::new("myfile.bpx");
        let wrong_ext_corrected = wrong_ext.ensure_extension("bpx");
        let no_ext_corrected = no_ext.ensure_extension("bpx");
        let correct_ext_corrected = correct_ext.ensure_extension("bpx");
        if let Cow::Owned(_) = correct_ext_corrected {
            panic!("If the extension is already correct no allocation should be performed")
        }
        assert_eq!(&wrong_ext_corrected, Path::new("myfile.bpx"));
        assert_eq!(&no_ext_corrected, Path::new("myfile.bpx"));
        assert_eq!(&correct_ext_corrected, Path::new("myfile.bpx"));
    }
}

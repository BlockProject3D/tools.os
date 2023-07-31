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

use std::io::Result;
use std::path::{Path, PathBuf};

/// An application path.
pub struct AppPath<'a> {
    path: &'a Path,
}

impl<'a> AsRef<Path> for AppPath<'a> {
    fn as_ref(&self) -> &Path {
        self.path
    }
}

impl<'a> AppPath<'a> {
    pub(crate) fn new(path: &'a Path) -> AppPath {
        AppPath { path }
    }

    /// Create the underlying path if it does not exist and return it.
    ///
    /// # Errors
    ///
    /// This function returns [Error](std::io::Error) if the path couldn't be created.
    pub fn create(&self) -> Result<&Path> {
        if !self.path.is_dir() {
            std::fs::create_dir_all(self.path)?;
        }
        Ok(self.path)
    }

    /// Create the underlying path if it does not exist and join a new path to it.
    ///
    /// # Errors
    ///
    /// This function returns [Error](std::io::Error) if the path couldn't be created.
    pub fn create_join<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        Ok(self.create()?.join(path))
    }

    /// Joins the underlying path with another component and returns the joint path.
    pub fn join<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.path.join(path)
    }
}

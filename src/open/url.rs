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

use std::convert::TryFrom;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;
use crate::fs::PathExt;

#[derive(Debug)]
pub struct InvalidUrl<'a>(&'a str);

impl<'a> Display for InvalidUrl<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid url \"{}\"", self.0)
    }
}

impl<'a> Error for InvalidUrl<'a> {}

pub struct Url<'a> {
    scheme: &'a str,
    path: &'a OsStr
}

impl<'a> Url<'a> {
    pub fn new(scheme: &'a str, path: &'a OsStr) -> Url<'a> {
        Url { scheme, path }
    }

    pub fn scheme(&self) -> &'a str {
        self.scheme
    }

    pub fn path(&self) -> &'a OsStr {
        self.path
    }

    pub fn is_path(&self) -> bool {
        self.scheme == "file"
    }

    pub fn to_os_str(&self) -> std::io::Result<OsString> {
        let mut s = OsString::from(self.scheme);
        s.push("://");
        if self.is_path() {
            let path = Path::new(self.path);
            if !path.is_absolute() {
                let path = path.get_absolute()?;
                s.push(path);
            } else {
                s.push(path);
            }
        } else {
            s.push(self.path);
        }
        Ok(s)
    }
}

impl<'a> Display for Url<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}://{}", self.scheme, self.path.to_string_lossy())
    }
}

impl<'a> From<&'a Path> for Url<'a> {
    fn from(value: &'a Path) -> Self {
        Url::new("file", value.as_os_str())
    }
}

impl<'a> TryFrom<&'a str> for Url<'a> {
    type Error = InvalidUrl<'a>;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value.find("://") {
            Some(id) => {
                let scheme = &value[..id];
                let path = &value[id + 3..];
                Ok(Url { scheme, path: path.as_ref() })
            },
            None => Err(InvalidUrl(value))
        }
    }
}

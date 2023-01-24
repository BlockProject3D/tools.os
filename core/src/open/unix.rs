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
use crate::open::{Url, Result, Error};
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::Command;
use zbus::{blocking::Connection, dbus_proxy, Result};

#[dbus_proxy(
    default_service = "org.freedesktop.FileManager1",
    interface = "org.freedesktop.FileManager1",
    default_path = "/org/freedesktop/FileManager1"
)]
trait FileManager {
    //This is what we want when the url is a path (file://) and a folder
    fn show_folders(&self, uris: &[&str], startup_id: &str) -> Result<()>;

    //This is what we want when we want to show items selected in the file explorer
    fn show_items(&self, uris: &[&str], startup_id: &str) -> Result<()>;
}

fn attempt_dbus_call(urls: &[&str], show_items: bool) -> Result<()> {
    let con = Connection::session()
        .map_err(|e| Error::Other(format!("DBus connection error: {}", e)))?;
    let proxy = FileManagerProxyBlocking::new(&con)
        .map_err(|e| Error::Other(format!("DBus error: {}", e)))?;
    let res = match show_items {
        true => proxy.show_items(urls, "test"),
        false => proxy.show_folders(urls, "test"),
    };
    match res {
        Err(e) => Err(Error::Other(format!("DBus error: {}", e)))?,
        Ok(_) => Ok(())
    }
}

fn attempt_xdg_open(url: &OsStr) -> Result<()> {
    let res = Command::new("xdg-open").args([url]).spawn();
    match res {
        Ok(_) => Ok(()),
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => Err(Error::Unsupported),
            _ => Err(Error::Io(e))
        }
    }
}

pub fn open(url: &Url) -> Result<()> {
    let path = Path::new(url.path());
    let uri = url.to_os_str().map_err(Error::Io)?;
    if !url.is_path() || !path.is_dir() {
        return attempt_xdg_open(&uri);
    }
    match uri.to_str() {
        Some(v) => attempt_dbus_call(&[v], false),
        None => attempt_xdg_open(&uri)
    }
}

pub fn show_in_files<'a, I: Iterator<Item = &'a Path>>(iter: I) -> Result<()> {
    let v: std::io::Result<Vec<OsString>> = iter
        .map(|v| {
            v.get_absolute().map(|v| {
                let mut s = OsString::with_capacity(v.as_os_str().len() + 7);
                s.push("file://");
                s.push(v.as_os_str());
                s
            })
        })
        .collect();
    let paths = v.map_err(Error::Io)?;
    let paths: Option<Vec<&str>> = paths.iter().map(|v| v.as_os_str().to_str()).collect();
    match paths {
        Some(v) => attempt_dbus_call(&v, true),
        None => Err(Error::Other("one ore more paths contains invalid UTF-8 characters".into()))
    }
}

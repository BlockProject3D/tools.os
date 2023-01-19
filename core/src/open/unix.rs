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

use crate::open::Url;
use std::ffi::OsStr;
use zbus::{blocking::Connection, dbus_proxy, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::fs::PathExt;

#[dbus_proxy(default_service = "org.freedesktop.FileManager1", interface = "org.freedesktop.FileManager1", default_path = "/org/freedesktop/FileManager1")]
trait FileManager {
    //This is what we want when the url is a path (file://) and a folder
    fn show_folders(&self, uris: &[&str], startup_id: &str) -> Result<()>;

    //This is what we want when we want to show items selected in the file explorer
    fn show_items(&self, uris: &[&str], startup_id: &str) -> Result<()>;
}

fn attempt_dbus_call(urls: &[&str], show_items: bool) -> bool {
    let con = match Connection::session() {
        Ok(v) => v,
        Err(_) => return false,
    };
    let proxy = match FileManagerProxyBlocking::new(&con) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let res = match show_items {
        true => proxy.show_items(urls, "test"),
        false => proxy.show_folders(urls, "test")
    };
    res.is_ok()
}

fn attempt_xdg_open(url: &OsStr) -> bool {
    let res = Command::new("xdg-open")
        .args([&*url])
        .output();
    res.is_ok()
}

pub fn open(url: &Url) -> bool {
    let path = Path::new(url.path());
    let uri = match url.to_os_str().ok() {
        Some(v) => v,
        None => return false
    };
    if !url.is_path() || !path.is_dir() {
        return attempt_xdg_open(&uri);
    }
    let mut flag = match uri.to_str() {
        Some(v) => attempt_dbus_call(&[v], false),
        None => false
    };
    if !flag {
        flag = attempt_xdg_open(&uri);
    }
    flag
}

pub fn show_in_files<'a, I: Iterator<Item = &'a Path>>(iter: I) -> bool {
    let v: std::io::Result<Vec<PathBuf>> = iter.map(|v| v.get_absolute()).collect();
    let paths: Option<Vec<&str>> = match v.as_ref() {
        Ok(v) => v.iter().map(|v| v.as_os_str().to_str()).collect(),
        Err(_) => return false
    };
    match paths {
        Some(v) => attempt_dbus_call(&v, true),
        None => false
    }
}

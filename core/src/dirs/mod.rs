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

//! This module provides cross-platform functions to get various system paths.

use std::path::PathBuf;
use std::sync::OnceLock;

pub use self::path::AppPath;

mod path;
pub mod system;

//TODO: Remove once once_cell_try feature is stabilized.
mod sealing {
    use std::sync::OnceLock;

    pub trait CellExt<T> {
        fn get_or_try_set<E>(&self, f: impl Fn() -> Result<T, E>) -> Result<&T, E>;
    }

    impl<T> CellExt<T> for OnceLock<T> {
        fn get_or_try_set<E>(&self, f: impl Fn() -> Result<T, E>) -> Result<&T, E> {
            if let Some(value) = self.get() {
                Ok(value)
            } else {
                let value = f()?;
                Ok(self.get_or_init(|| value))
            }
        }
    }
}

use sealing::CellExt;

/// Represents the application's directories.
///
/// Main entry point to obtain any directory for your application.
///
/// These APIs will fail as last resort. If they fail it usually means the system has a problem.
/// The system may also include specific configuration to break applications on purpose,
/// in which case these APIs will also fail.
///
/// These APIs do not automatically create the directories, instead they return a matching instance of [AppPath](AppPath).
pub struct App<'a> {
    name: &'a str,
    data: OnceLock<PathBuf>,
    cache: OnceLock<PathBuf>,
    docs: OnceLock<PathBuf>,
    logs: OnceLock<PathBuf>,
    config: OnceLock<PathBuf>,
}

impl<'a> App<'a> {
    /// Creates a new application.
    ///
    /// # Arguments
    ///
    /// * `name`: the name of the application.
    ///
    /// returns: App
    pub fn new(name: &'a str) -> App<'a> {
        App {
            name,
            data: OnceLock::new(),
            cache: OnceLock::new(),
            docs: OnceLock::new(),
            logs: OnceLock::new(),
            config: OnceLock::new(),
        }
    }

    /// Returns the path to this application's files.
    ///
    /// Use this directory to store any information not intended to be user accessible.
    /// Returns None if this system doesn't have any application writable location; this should
    /// never occur on any supported system except if such system is broken.
    pub fn get_data(&self) -> Option<AppPath> {
        self.data
            .get_or_try_set(|| system::get_app_data().ok_or(()).map(|v| v.join(self.name)))
            .ok()
            .map(|v| v.as_ref())
            .map(AppPath::new)
    }

    /// Returns the path to this application's cache.
    ///
    /// Use this directory to store cached files such as downloads, intermediate files, etc.
    ///
    /// This function first tries to use [get_app_cache](system::get_app_cache)/{APP} and
    /// falls back to [get_data](App::get_data)/Cache.
    pub fn get_cache(&self) -> Option<AppPath> {
        self.cache
            .get_or_try_set(|| {
                system::get_app_cache()
                    .map(|v| v.join(self.name))
                    .or_else(|| self.get_data().map(|v| v.join("Cache")))
                    .ok_or(())
            })
            .ok()
            .map(|v| v.as_ref())
            .map(AppPath::new)
    }

    /// Returns the path to this application's public documents.
    ///
    /// Use this directory to store any content the user should see and alter.
    ///
    /// This function first tries to use [get_app_documents](system::get_app_documents) and
    /// falls back to [get_data](App::get_data)/Documents.
    pub fn get_documents(&self) -> Option<AppPath> {
        // If this is OK then we must be running from a sandboxed system
        // where the app has it's own public documents folder, otherwise
        // create a "public" Documents directory inside the application's data directory.
        self.docs
            .get_or_try_set(|| {
                system::get_app_documents()
                    .or_else(|| self.get_data().map(|v| v.join("Documents")))
                    .ok_or(())
            })
            .ok()
            .map(|v| v.as_ref())
            .map(AppPath::new)
    }

    /// Returns the path to this application's logs.
    ///
    /// Use this directory to store all logs. The user can view and alter this directory.
    ///
    /// This function first tries to use [get_app_logs](system::get_app_logs)/{APP} and
    /// falls back to [get_documents](App::get_documents)/Logs.
    pub fn get_logs(&self) -> Option<AppPath> {
        // Logs should be public and not contain any sensitive information, so store that in
        // the app's public documents.
        self.logs
            .get_or_try_set(|| {
                system::get_app_logs()
                    .map(|v| v.join(self.name))
                    .or_else(|| self.get_documents().map(|v| v.join("Logs")))
                    .ok_or(())
            })
            .ok()
            .map(|v| v.as_ref())
            .map(AppPath::new)
    }

    /// Returns the path to this application's config.
    ///
    /// Use this directory to store all configs for the current user.
    /// This directory is not intended for direct user access.
    ///
    /// This function first tries to use [get_app_config](system::get_app_config)/{APP} and
    /// falls back to [get_data](App::get_data)/Config.
    pub fn get_config(&self) -> Option<AppPath> {
        self.config
            .get_or_try_set(|| {
                system::get_app_config()
                    .map(|v| v.join(self.name))
                    .or_else(|| self.get_data().map(|v| v.join("Config")))
                    .ok_or(())
            })
            .ok()
            .map(|v| v.as_ref())
            .map(AppPath::new)
    }
}

impl<'a> Clone for App<'a> {
    fn clone(&self) -> Self {
        App {
            name: self.name,
            data: self.data.clone(),
            cache: self.cache.clone(),
            docs: self.docs.clone(),
            logs: self.logs.clone(),
            config: self.config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::dirs::App;

    fn assert_sync_send<T: Sync + Send>(x: T) -> T {
        x
    }

    #[test]
    fn test_sync_send() {
        let obj = App::new("test");
        let _ = assert_sync_send(obj);
    }

    #[test]
    fn api_breakage() {
        let app = App::new("test");
        let _: Option<PathBuf> = app
            .get_logs()
            .map(|v| v.create())
            .unwrap()
            .ok()
            .map(|v| v.into());
    }
}

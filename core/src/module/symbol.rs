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

//! This module contains helpers for C module symbols.

use std::ffi::c_void;

/// This represents a symbol from a module.
#[derive(Debug)]
pub struct Symbol<T>(*const T);

impl<T> Symbol<T> {
    /// Creates a new [Symbol] from a raw pointer.
    ///
    /// # Arguments
    ///
    /// * `val`: the raw pointer.
    ///
    /// returns: Symbol<T>
    ///
    /// # Safety
    ///
    /// This is UB if val does not match the signature of T.
    pub unsafe fn from_raw(val: *const c_void) -> Self {
        Self(val as *const T)
    }

    /// Returns the raw pointer of this symbol.
    pub fn as_ptr(&self) -> *const T {
        self.0
    }
}

impl<T, R> Symbol<extern "C" fn (T) -> R> {
    /// Calls this symbol if this symbol is a function.
    ///
    /// # Arguments
    ///
    /// * `val`: argument #1.
    ///
    /// returns: R
    pub fn call(&self, val: T) -> R {
        let f: fn(T) -> R = unsafe { std::mem::transmute(self.0) };
        f(val)
    }
}

impl<T, T1, R> Symbol<extern "C" fn (T, T1) -> R> {
    /// Calls this symbol if this symbol is a function.
    ///
    /// # Arguments
    ///
    /// * `val`: argument #1.
    ///
    /// returns: R
    pub fn call(&self, val: T, val1: T1) -> R {
        let f: fn(T, T1) -> R = unsafe { std::mem::transmute(self.0) };
        f(val, val1)
    }
}

impl<T, T1, T2, R> Symbol<extern "C" fn (T, T1, T2) -> R> {
    /// Calls this symbol if this symbol is a function.
    ///
    /// # Arguments
    ///
    /// * `val`: argument #1.
    ///
    /// returns: R
    pub fn call(&self, val: T, val1: T1, val2: T2) -> R {
        let f: fn(T, T1, T2) -> R = unsafe { std::mem::transmute(self.0) };
        f(val, val1, val2)
    }
}

impl<R> Symbol<extern "C" fn () -> R> {
    /// Calls this symbol if this symbol is a function.
    ///
    /// returns: R
    pub fn call(&self) -> R {
        let f: fn() -> R = unsafe { std::mem::transmute(self.0) };
        f()
    }
}

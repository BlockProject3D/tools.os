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

use std::cell::Cell;
use std::mem::MaybeUninit;
use windows_sys::Win32::System::Console::{
    GetConsoleMode, GetConsoleScreenBufferInfo, GetStdHandle, SetConsoleMode, WriteConsoleW,
    CONSOLE_MODE, CONSOLE_SCREEN_BUFFER_INFO, ENABLE_VIRTUAL_TERMINAL_PROCESSING,
    STD_OUTPUT_HANDLE,
};

/// Represents an interactive terminal.
pub struct Terminal {
    attrs: CONSOLE_MODE,
}

impl Terminal {
    /// Creates a new instance of an interactive terminal.
    ///
    /// This function automatically sets-up the current OS terminal for interactive input and resets
    /// it back on drop automatically.
    pub fn new() -> Self {
        let mut attrs = MaybeUninit::<CONSOLE_MODE>::uninit();
        unsafe {
            let handle = GetStdHandle(STD_OUTPUT_HANDLE);
            if GetConsoleMode(handle, attrs.as_mut_ptr()) != 1 {
                panic!("Failed to initialize a windows console");
            }
            let mut attrs2 = attrs.assume_init();
            attrs2 |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
            SetConsoleMode(handle, attrs2);
            Terminal {
                attrs: attrs.assume_init(),
            }
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        unsafe {
            let handle = GetStdHandle(STD_OUTPUT_HANDLE);
            SetConsoleMode(handle, self.attrs);
        }
    }
}

/// Writes the given string immediately (unbuffered).
pub fn write(str: &str) {
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        let mut encoded = str.encode_utf16().collect::<Vec<_>>();
        encoded.push(0);
        WriteConsoleW(
            handle,
            encoded.as_ptr(),
            (encoded.len() - 1) as _,
            std::ptr::null_mut(),
            std::ptr::null(),
        );
    }
}

/// Returns a tuple with respectively the maximum number of columns and rows available in the
/// terminal.
///
/// This function issues a syscall each time it is invoked.
pub fn get_window_size() -> (i32, i32) {
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        let mut info = MaybeUninit::<CONSOLE_SCREEN_BUFFER_INFO>::uninit();
        GetConsoleScreenBufferInfo(handle, info.as_mut_ptr());
        let info = info.assume_init();
        let columns = info.srWindow.Right - info.srWindow.Left + 1;
        let rows = info.srWindow.Bottom - info.srWindow.Top + 1;
        (columns as _, rows as _)
    }
}

thread_local! {
    static HEIGHT: Cell<i32> = Cell::new(-1);
}

/// Returns the maximum number of rows available in the terminal.
///
/// This function amortizes the cost of the syscall by only issuing it once for the current thread.
pub fn get_window_height_amortized() -> i32 {
    if HEIGHT.get() == -1 {
        let (_, rows) = get_window_size();
        HEIGHT.set(rows);
    }
    HEIGHT.get()
}

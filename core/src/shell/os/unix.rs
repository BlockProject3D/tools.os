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

/// Represents an interactive terminal.
pub struct Terminal {
    attrs: libc::termios,
}

impl crate::shell::os::Terminal {
    /// Creates a new instance of an interactive terminal.
    ///
    /// This function automatically sets-up the current OS terminal for interactive input and resets
    /// it back on drop automatically.
    pub fn new() -> Self {
        let mut attrs = MaybeUninit::<libc::termios>::uninit();
        unsafe {
            libc::tcgetattr(0, attrs.as_mut_ptr());
            let mut newattrs = attrs.assume_init();
            newattrs.c_lflag &= !(libc::ICANON | libc::ECHO);
            libc::tcsetattr(0, libc::TCSANOW, &newattrs);
            crate::shell::os::Terminal { attrs: attrs.assume_init() }
        }
    }
}

impl Drop for crate::shell::os::Terminal {
    fn drop(&mut self) {
        unsafe { libc::tcsetattr(0, libc::TCSANOW, &self.attrs); }
    }
}

/// Writes the given string immediately (unbuffered).
pub fn write(str: &str) {
    unsafe {
        libc::write(1, str.as_bytes().as_ptr() as _, str.len());
    }
}

/// Returns a tuple with respectively the maximum number of columns and rows available in the
/// terminal.
///
/// This function issues a syscall each time it is invoked.
pub fn get_window_size() -> (i32, i32) {
    let mut sz = std::mem::MaybeUninit::<libc::winsize>::uninit();
    unsafe {
        libc::ioctl(1, libc::TIOCGWINSZ, sz.as_mut_ptr());
        (sz.assume_init().ws_col as _, sz.assume_init().ws_row as _)
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
        let mut sz = std::mem::MaybeUninit::<libc::winsize>::uninit();
        unsafe {
            libc::ioctl(1, libc::TIOCGWINSZ, sz.as_mut_ptr());
            HEIGHT.set(sz.assume_init().ws_row as _);
        }
    }
    HEIGHT.get()
}

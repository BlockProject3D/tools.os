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

//! Low-level platform-specific tools to control the OS console/terminal.

#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod windows;

#[cfg(unix)]
pub use unix::Terminal;

#[cfg(unix)]
pub use unix::write;

#[cfg(unix)]
pub use unix::get_window_size;

#[cfg(unix)]
pub use unix::get_window_height_amortized;

#[cfg(windows)]
pub use windows::Terminal;

#[cfg(windows)]
pub use windows::write;

#[cfg(windows)]
pub use windows::get_window_size;

#[cfg(windows)]
pub use windows::get_window_height_amortized;

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}

/// Move the terminal cursor to the given x, y position in columns and rows respectively.
pub fn move_cursor(x: i32, y: i32) {
    write(&format!("\x1b[{};{}H", y, x + 1)) // yeah rust is broken: impossible to use octal set
}

/// Clear the rest of the current line starting at the current cursor position.
pub fn clear_remaining() {
    write("\x1b[K");
}

/// Simplified macro which does exactly the same as [println](std::println) but overwrites the current prompt
/// rather than appending text after the prompt.
#[macro_export]
macro_rules! shell_println {
    ($($data: tt)*) => {
        $crate::shell::os::move_cursor(0, $crate::shell::os::get_window_height_amortized());
        $crate::shell::os::clear_remaining();
        println!($($data)*);
    };
}

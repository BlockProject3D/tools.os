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

use std::cell::Cell;

/// Represents an interactive terminal.
pub struct Terminal {
    #[cfg(unix)]
    attrs: libc::termios,
}

impl Terminal {
    /// Creates a new instance of an interactive terminal.
    ///
    /// This function automatically sets-up the current OS terminal for interactive input and resets
    /// it back on drop automatically.
    pub fn new() -> Self {
        #[cfg(unix)]
        {
            use std::mem::MaybeUninit;
            let mut attrs = MaybeUninit::<libc::termios>::uninit();
            unsafe {
                libc::tcgetattr(0, attrs.as_mut_ptr());
                let mut newattrs = attrs.assume_init();
                newattrs.c_lflag &= !(libc::ICANON | libc::ECHO);
                libc::tcsetattr(0, libc::TCSANOW, &newattrs);
                Terminal { attrs: attrs.assume_init() }
            }
        }
        #[cfg(windows)]
        {
            Terminal {}
        }
    }

    /*pub fn cancel(&self) {
        //FIXME: Not working.
        #[cfg(unix)]
        unsafe { libc::close(0); }
        //TODO: CancelIoEx
    }*/
}

impl Drop for Terminal {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            unsafe { libc::tcsetattr(0, libc::TCSANOW, &self.attrs); }
        }
    }
}

/// Writes the given string immediately (unbuffered).
pub fn write(str: &str) {
    #[cfg(unix)]
    {
        unsafe {
            libc::write(1, str.as_bytes().as_ptr() as _, str.len());
        }
    }
    #[cfg(windows)]
    write!("{}", str)
}

/// Move the terminal cursor to the given x, y position in columns and rows respectively.
pub fn move_cursor(x: i32, y: i32) {
    #[cfg(unix)]
    write(&format!("\x1b[{};{}H", y, x + 1)) // yeah rust is broken: impossible to use octal set
}

/// Clear the rest of the current line starting at the current cursor position.
pub fn clear_remaining() {
    #[cfg(unix)]
    write("\x1b[K");
}

/// Returns a tuple with respectively the maximum number of columns and rows available in the
/// terminal.
///
/// This function issues a syscall each time it is invoked.
pub fn get_window_size() -> (i32, i32) {
    #[cfg(unix)]
    {
        let mut sz = std::mem::MaybeUninit::<libc::winsize>::uninit();
        unsafe {
            libc::ioctl(1, libc::TIOCGWINSZ, sz.as_mut_ptr());
            (sz.assume_init().ws_col as _, sz.assume_init().ws_row as _)
        }
    }
    #[cfg(windows)]
    (0, 0)
}

thread_local! {
    static HEIGHT: Cell<i32> = Cell::new(-1);
}

/// Returns the maximum number of rows available in the terminal.
///
/// This function amortizes the cost of the syscall by only issuing it once for the current thread.
pub fn get_window_height_amortized() -> i32 {
    #[cfg(unix)]
    {
        if HEIGHT.get() == -1 {
            let mut sz = std::mem::MaybeUninit::<libc::winsize>::uninit();
            unsafe {
                libc::ioctl(1, libc::TIOCGWINSZ, sz.as_mut_ptr());
                HEIGHT.set(sz.assume_init().ws_row as _);
            }
        }
        HEIGHT.get()
    }
    #[cfg(windows)]
    0
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

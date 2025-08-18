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

use crate::shell::input_thread::{input_thread, InputEvent};
use crate::shell::os::{clear_remaining, get_window_size, move_cursor, write, Terminal};
use crate::shell_println;
use std::sync::mpsc;
use std::thread::JoinHandle;

/// Represents an event emitted from the input abstraction.
pub enum Event {
    /// A command string was submitted to the application.
    CommandReceived(String),

    /// Application exit was requested.
    ExitRequested,
}

/// Represents any object which can be used to send [Event] structures.
pub trait SendChannel: Send + 'static {
    /// Sends an [Event] to the underlying application.
    ///
    /// # Arguments
    ///
    /// * `event`: the event to send.
    ///
    /// returns: ()
    fn send(&self, event: Event);
}

/// Represents an interactive shell
pub struct Shell {
    _os: Terminal,
    input_thread: JoinHandle<()>,
    app_thread: JoinHandle<()>,
    _send_ch: mpsc::Sender<InputEvent>,
}

fn print_prompt(row: i32, prompt: &'static str) {
    move_cursor(0, row);
    write(prompt);
    move_cursor(prompt.len() as _, row);
}

enum Window {
    StartEnd(usize, usize),
    Start(usize),
    Full,
}

fn string_window(pos: usize, col: i32, prompt: &'static str, str: &str) -> Window {
    let maxsize = col as usize - prompt.len() - 1;
    if str.len() > maxsize {
        if pos >= str.len() {
            Window::Start(str.len() - maxsize)
        } else {
            let mut start = pos;
            let mut end = pos + maxsize;
            while end >= str.len() {
                end -= 1;
                if start > 0 {
                    start -= 1;
                }
            }
            Window::StartEnd(start, end)
        }
    } else {
        Window::Full
    }
}

fn reset_string(pos: usize, col: i32, row: i32, prompt: &'static str, str: &str) {
    print_prompt(row, prompt);
    let window = string_window(pos, col, prompt, str);
    match window {
        Window::Start(start) => write(&str[start..]),
        Window::StartEnd(start, end) => write(&str[start..end]),
        Window::Full => write(str),
    }
    clear_remaining();
}

fn move_to_pos(pos: usize, col: i32, row: i32, prompt: &'static str, str: &str) {
    let window = string_window(pos, col, prompt, str);
    match window {
        Window::StartEnd(start, _) => move_cursor((prompt.len() + (pos - start)) as _, row),
        Window::Start(start) => move_cursor((prompt.len() + (pos - start)) as _, row),
        Window::Full => move_cursor((prompt.len() + pos) as _, row),
    }
}

fn application_thread<T: SendChannel>(
    prompt: &'static str,
    recv_ch: mpsc::Receiver<InputEvent>,
    master_send_ch: T,
) {
    let mut history = Vec::new();
    let mut hindex = 0;
    let mut cur_line = String::new();
    let (col, row) = get_window_size();
    let mut pos = 0;
    print_prompt(row, prompt);
    loop {
        let msg = recv_ch.recv().unwrap();
        match msg {
            InputEvent::End => {
                master_send_ch.send(Event::ExitRequested);
                break;
            }
            InputEvent::NewLine => {
                write("\n");
                print_prompt(row, prompt);
                history.push(cur_line.clone());
                hindex = history.len();
                master_send_ch.send(Event::CommandReceived(cur_line.clone()));
                cur_line.clear();
                pos = 0;
            }
            InputEvent::Complete => {
                shell_println!("Not yet implemented");
            }
            InputEvent::HistoryPrev => {
                if history.len() == 0 {
                    continue;
                }
                if hindex != 0 {
                    hindex -= 1;
                }
                let msg = &history[hindex];
                cur_line = msg.clone();
                pos = cur_line.len();
                reset_string(pos, col, row, prompt, &cur_line);
            }
            InputEvent::HistoryNext => {
                if history.len() == 0 {
                    continue;
                }
                if hindex != history.len() {
                    hindex += 1;
                }
                if hindex == history.len() {
                    reset_string(0, col, row, prompt, "");
                    cur_line.clear();
                    pos = 0;
                    continue;
                }
                let msg = &history[hindex];
                cur_line = msg.clone();
                pos = cur_line.len();
                reset_string(pos, col, row, prompt, &cur_line);
            }
            InputEvent::LineStart => {
                pos = 0;
                reset_string(pos, col, row, prompt, &cur_line);
                move_to_pos(pos, col, row, prompt, &cur_line);
            }
            InputEvent::LineEnd => {
                pos = cur_line.len();
                reset_string(pos, col, row, prompt, &cur_line);
                move_to_pos(pos, col, row, prompt, &cur_line);
            }
            InputEvent::Input(s) => {
                cur_line.insert_str(pos, &s);
                pos += s.len();
                reset_string(pos, col, row, prompt, &cur_line);
                move_to_pos(pos, col, row, prompt, &cur_line);
            }
            InputEvent::Left => {
                if pos == 0 {
                    continue;
                }
                pos -= 1;
                reset_string(pos, col, row, prompt, &cur_line);
                move_to_pos(pos, col, row, prompt, &cur_line);
            }
            InputEvent::Right => {
                if pos >= cur_line.len() {
                    continue;
                }
                pos += 1;
                reset_string(pos, col, row, prompt, &cur_line);
                move_to_pos(pos, col, row, prompt, &cur_line);
            }
            InputEvent::Delete => {
                if pos == 0 {
                    continue;
                }
                cur_line.remove(pos - 1);
                pos -= 1;
                reset_string(pos, col, row, prompt, &cur_line);
                move_to_pos(pos, col, row, prompt, &cur_line);
            }
        }
    }
}

impl Shell {
    /// Creates a new interactive shell type application.
    ///
    /// This internally creates the [Terminal] instance to set up the OS terminal properly.
    ///
    /// # Arguments
    ///
    /// * `prompt`: a static prompt string to display as input prefix.
    /// * `master_send_ch`: the master channel where application events should be submitted.
    ///
    /// returns: Shell
    pub fn new<T: SendChannel>(prompt: &'static str, master_send_ch: T) -> Self {
        let (send_ch, recv_ch) = mpsc::channel();
        let motherfuckingrust = send_ch.clone();
        let input_thread = std::thread::spawn(|| {
            input_thread(motherfuckingrust);
        });
        let app_thread = std::thread::spawn(move || {
            application_thread(prompt, recv_ch, master_send_ch);
        });
        Self {
            _os: Terminal::new(),
            input_thread,
            app_thread,
            _send_ch: send_ch,
        }
    }

    /// Gracefully exits this interactive shell.
    pub fn exit(self) {
        // Should interrupt the syscall and make the syscall return -1.
        #[cfg(unix)]
        {
            // Use SIGUSR2 because SIGUSR1 is reserved for application use.
            use std::os::unix::thread::JoinHandleExt;

            // Attach to SIGUSR2 an empty function to use the EINTR syscall error.
            extern "C" fn useless() {}
            let mut sig2: std::mem::MaybeUninit<libc::sigaction> = std::mem::MaybeUninit::uninit();
            let mut sig: libc::sigaction = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
            sig.sa_sigaction = useless as _;
            unsafe { libc::sigaction(libc::SIGUSR2, &sig as _, sig2.as_mut_ptr()) };

            // Send a signal to the input thread which should raise EINTR on the getchar function.
            let pthread = self.input_thread.as_pthread_t();
            unsafe { libc::pthread_kill(pthread, libc::SIGUSR2) };

            // Join the threads.
            self.input_thread.join().unwrap();
            self.app_thread.join().unwrap();

            // Reset the previous action attached to SIGUSR2 in case the application would be using
            // that particular signal.
            unsafe { libc::sigaction(libc::SIGUSR2, sig2.as_ptr(), std::ptr::null_mut()) };
        }
        #[cfg(windows)]
        {
            // Cancel all pending IO operations on standard input.
            let handle = unsafe {
                windows_sys::Win32::System::Console::GetStdHandle(
                    windows_sys::Win32::System::Console::STD_INPUT_HANDLE,
                )
            };
            unsafe { windows_sys::Win32::System::IO::CancelIoEx(handle, std::ptr::null()) };

            // Join the threads.
            self.input_thread.join().unwrap();
            self.app_thread.join().unwrap();
        }
    }
}

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

use super::InputEvent;
use libc::getchar;
use libc::EOF;
use std::sync::mpsc;

const BUF_SIZE: usize = 8;

fn handle_input(buf: &[u8], log_ch: &mpsc::Sender<InputEvent>) -> bool {
    // Codes found by reverse engineering on macOS Terminal. Apparently these codes are NEVER EVER
    // documented in the entire internet. All docs I found expose wrong information.
    const CODE_LEFT: &[u8] = &[27, 91, 68];
    const CODE_RIGHT: &[u8] = &[27, 91, 67];
    const CODE_UP: &[u8] = &[27, 91, 65];
    const CODE_DOWN: &[u8] = &[27, 91, 66];
    const CODE_TAB: &[u8] = b"\t";
    const CODE_HOME: &[u8] = &[27, 91, 72];
    const CODE_END: &[u8] = &[27, 91, 70];

    if buf == b"\x04" {
        unsafe { libc::close(0) };
    } else if buf == CODE_LEFT {
        log_ch.send(InputEvent::Left).unwrap();
        return true;
    } else if buf == CODE_RIGHT {
        log_ch.send(InputEvent::Right).unwrap();
        return true;
    } else if buf == CODE_TAB {
        log_ch.send(InputEvent::Complete).unwrap();
        return true;
    } else if buf == CODE_UP {
        log_ch.send(InputEvent::HistoryPrev).unwrap();
        return true;
    } else if buf == CODE_DOWN {
        log_ch.send(InputEvent::HistoryNext).unwrap();
        return true;
    } else if buf == CODE_HOME {
        log_ch.send(InputEvent::LineStart).unwrap();
        return true;
    } else if buf == CODE_END {
        log_ch.send(InputEvent::LineEnd).unwrap();
        return true;
    } else if buf == b"\n" {
        log_ch.send(InputEvent::NewLine).unwrap();
        return true;
    } else if buf == b"\x7f" {
        log_ch.send(InputEvent::Delete).unwrap();
        return true;
    } else if buf[0] != 27 {
        log_ch
            .send(InputEvent::Input(String::from_utf8_lossy(buf).into()))
            .unwrap();
        return true;
    }
    false
}

pub fn input_thread(log_ch: mpsc::Sender<InputEvent>) {
    let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
    let mut idx = 0;
    loop {
        let ch = unsafe { getchar() };
        if ch == EOF {
            break;
        }
        buf[idx] = ch as u8;
        if idx < BUF_SIZE {
            idx += 1;
        }
        if handle_input(&buf[..idx], &log_ch) {
            idx = 0;
        }
    }
    log_ch.send(InputEvent::End).unwrap();
}

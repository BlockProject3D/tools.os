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
use std::mem::MaybeUninit;
use std::sync::mpsc;
use windows_sys::Win32::System::Console::{
    GetStdHandle, ReadConsoleInputW, INPUT_RECORD, STD_INPUT_HANDLE,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    VK_BACK, VK_C, VK_CONTROL, VK_D, VK_DOWN, VK_END, VK_HOME, VK_LEFT, VK_RETURN, VK_RIGHT,
    VK_SHIFT, VK_TAB, VK_UP,
};

const BUF_SIZE: usize = 1;

pub fn input_thread(log_ch: mpsc::Sender<InputEvent>) {
    let handle = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
    let mut buf: [INPUT_RECORD; BUF_SIZE] = unsafe { MaybeUninit::zeroed().assume_init() };
    let mut eventnum = 0;
    let mut is_ctrl = false;
    let mut surrogate: u32 = 0;
    loop {
        let flag =
            unsafe { ReadConsoleInputW(handle, buf.as_mut_ptr(), BUF_SIZE as _, &mut eventnum) };
        if flag != 1 {
            break;
        }
        let mut end = false;
        for i in 0..eventnum {
            let event = buf[i as usize];
            match event.EventType {
                0x0001 => {
                    let record = unsafe { event.Event.KeyEvent };
                    if record.wVirtualKeyCode == VK_CONTROL {
                        is_ctrl = record.bKeyDown == 1;
                    }
                    if record.bKeyDown == 1 {
                        if is_ctrl
                            && (record.wVirtualKeyCode == VK_C || record.wVirtualKeyCode == VK_D)
                        {
                            end = true;
                            break;
                        }
                        match record.wVirtualKeyCode {
                            VK_BACK => log_ch.send(InputEvent::Delete).unwrap(),
                            VK_LEFT => log_ch.send(InputEvent::Left).unwrap(),
                            VK_RIGHT => log_ch.send(InputEvent::Right).unwrap(),
                            VK_UP => log_ch.send(InputEvent::HistoryPrev).unwrap(),
                            VK_DOWN => log_ch.send(InputEvent::HistoryNext).unwrap(),
                            VK_HOME => log_ch.send(InputEvent::LineStart).unwrap(),
                            VK_END => log_ch.send(InputEvent::LineEnd).unwrap(),
                            VK_RETURN => log_ch.send(InputEvent::NewLine).unwrap(),
                            VK_TAB => log_ch.send(InputEvent::Complete).unwrap(),
                            VK_SHIFT => (),
                            _ => {
                                let val =
                                    std::char::from_u32(unsafe { record.uChar.UnicodeChar } as _);
                                match val {
                                    Some(c) => {
                                        log_ch.send(InputEvent::Input(String::from(c))).unwrap()
                                    }
                                    None => {
                                        if surrogate != 0 {
                                            let val = std::char::from_u32(
                                                surrogate
                                                    | unsafe { record.uChar.UnicodeChar } as u32,
                                            );
                                            if let Some(c) = val {
                                                log_ch
                                                    .send(InputEvent::Input(String::from(c)))
                                                    .unwrap();
                                            }
                                            surrogate = 0;
                                        } else {
                                            surrogate = unsafe { record.uChar.UnicodeChar } as _;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => continue,
            }
        }
        if end {
            break;
        }
    }
    log_ch.send(InputEvent::End).unwrap();
}

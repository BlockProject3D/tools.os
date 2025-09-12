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

macro_rules! __msg_send_parse {
    // No arguments
    {
        // Intentionally empty
        ()
        ()
        ($selector:ident $(,)?)

        ($($error_data:tt)*)
        ($($data:tt)*)

        ($out_macro:path)
        $($macro_args:tt)*
    } => {
        __msg_send_parse! {
            ($selector)
            ()
            ()

            ($($error_data)*)
            ($($data)*)

            ($out_macro)
            $($macro_args)*
        }
    };

    // tt-munch remaining `selector: argument` pairs, looking for a pattern
    // that ends with `sel: _`.
    {
        ($($selector_output:tt)*)
        ($($argument_output:tt)*)
        ()

        ($($error_data:tt)*)
        ($($data:tt)*)

        ($out_macro:path)
        $($macro_args:tt)*
    } => ({
        $out_macro! {
            $($macro_args)*

            ($($data)*)
            ($($selector_output)*)
            ($($argument_output)*)
        }
    });
    {
        ($($selector_output:tt)*)
        ($($argument_output:tt)*)
        ($selector:ident: _ $(,)?)

        ($($error_data:tt)*)
        ($($data:tt)*)

        ($out_macro:path)
        $($macro_args:tt)*
    } => {
        __msg_send_parse! {
            ($($selector_output)* $selector:)
            // Don't pass an argument
            ($($argument_output)*)
            ()

            // Instead, we change the data to the error data.
            ($($error_data)*)
            ($($error_data)*)

            ($out_macro)
            $($macro_args)*
        }
    };
    {
        ($($selector_output:tt)*)
        ($($argument_output:tt)*)
        ($selector:ident : $argument:expr $(, $($rest:tt)*)?)

        ($($error_data:tt)*)
        ($($data:tt)*)

        ($out_macro:path)
        $($macro_args:tt)*
    } => {
        __msg_send_parse! {
            ($($selector_output)* $selector:)
            ($($argument_output)* $argument,)
            ($($($rest)*)?)

            ($($error_data)*)
            ($($data)*)

            ($out_macro)
            $($macro_args)*
        }
    };

    // Handle calls without comma between `selector: argument` pair.
    {
        // Intentionally empty
        ()
        ()
        ($($selector:ident : $argument:expr)*)

        ($($error_data:tt)*)
        ($($data:tt)*)

        ($out_macro:path)
        $($macro_args:tt)*
    } => {{
        __msg_send_parse! {
            ()
            ()
            ($($selector : $argument),*)

            ($($error_data)*)
            ($($data)*)

            ($out_macro)
            $($macro_args)*
        }
    }};
}

macro_rules! msg_send {
    [$obj:expr, $($selector_and_arguments:tt)+] => {
        __msg_send_parse! {
            ()
            ()
            ($($selector_and_arguments)+)

            (MsgSendError::send_message_error)
            (MsgSend::send_message)

            (objc2::__msg_send_helper)
            ($obj)
            () // No method family
        }
    };
}

macro_rules! obj_alloc {
    ($class_name: ident, $($init: tt)*) => {
        {
            let class = class!($class_name);
            let mut obj: *mut Object = msg_send![class, alloc];
            obj = msg_send![obj, $($init)*];
            Retained::from_raw(obj).unwrap_unchecked()
        }
    };
}

macro_rules! obj_from {
    ($class_name: ident, $($init: tt)*) => {
        {
            let class = class!($class_name);
            let obj: *mut Object = msg_send![class, $($init)*];
            Retained::from_raw(obj).unwrap_unchecked()
        }
    };
}

pub(crate) use __msg_send_parse;
pub(crate) use msg_send;
pub(crate) use obj_alloc;
pub(crate) use obj_from;
use objc2::__framework_prelude::NSUInteger;
use std::ffi::c_char;

use objc2::runtime::{AnyObject, Bool};

pub type BOOL = Bool;
pub const NO: Bool = Bool::NO;
pub type Object = AnyObject;

pub fn ns_string_to_string(obj: &Object) -> &str {
    let data: *const c_char = unsafe { msg_send![obj, UTF8String] };
    let len: NSUInteger = unsafe { msg_send![obj, length] };
    let slice = unsafe { std::slice::from_raw_parts(data as *const u8, len as usize) };
    unsafe { std::str::from_utf8_unchecked(slice) }
}

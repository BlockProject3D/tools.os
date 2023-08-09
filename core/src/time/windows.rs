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

use std::mem::MaybeUninit;

use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};
use windows_sys::Win32::System::Time::{GetTimeZoneInformation, TIME_ZONE_ID_INVALID};

use crate::time::MonthExt;

pub fn local_offset_at(tm: &OffsetDateTime) -> Option<UtcOffset> {
    let mut info = MaybeUninit::uninit();
    unsafe {
        let res = GetTimeZoneInformation(info.as_mut_ptr());
        println!("{}", res);
        None
        /*if res == TIME_ZONE_ID_INVALID {
            None
        } else {
            let info = info.assume_init();
            //Windows works at inverse instead of storing propely the bias based on UTC time it stores the bias based on local time.
            let mut offset = info.Bias * 60;
            let tempoffset = UtcOffset::from_whole_seconds(offset).ok()?;
            let standard_date = PrimitiveDateTime::new(
                Date::from_calendar_date(
                    info.StandardDate.wYear as _,
                    Month::from_index(info.StandardDate.wMonth as _).unwrap_unchecked(),
                    info.StandardDate.wDay as _,
                )
                .unwrap_unchecked(),
                Time::from_hms_milli(
                    info.StandardDate.wHour as _,
                    info.StandardDate.wMinute as _,
                    info.StandardDate.wSecond as _,
                    info.StandardDate.wMilliseconds,
                )
                .unwrap_unchecked(),
            )
            .assume_offset(tempoffset);
            let daylight_date = PrimitiveDateTime::new(
                Date::from_calendar_date(
                    info.DaylightDate.wYear as _,
                    Month::from_index(info.DaylightDate.wMonth as _).unwrap_unchecked(),
                    info.DaylightDate.wDay as _,
                )
                .unwrap_unchecked(),
                Time::from_hms_milli(
                    info.DaylightDate.wHour as _,
                    info.DaylightDate.wMinute as _,
                    info.DaylightDate.wSecond as _,
                    info.DaylightDate.wMilliseconds,
                )
                .unwrap_unchecked(),
            )
            .assume_offset(tempoffset);
            if tm > &standard_date {
                offset += info.StandardBias * 60;
            }
            if tm > &daylight_date {
                offset += info.DaylightBias * 60;
            }
            UtcOffset::from_whole_seconds(-offset).ok()
        }*/
    }
}

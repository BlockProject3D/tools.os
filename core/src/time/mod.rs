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

//! OS local date/time extensions for time-rs.

pub mod tzif;

#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod windows;

#[cfg(unix)]
use unix as _impl;

#[cfg(windows)]
use windows as _impl;

use time::{OffsetDateTime, UtcOffset, Month};

mod sealed {
    use time::{OffsetDateTime, UtcOffset, Month};

    pub trait SealUO {}
    pub trait SealODT {}
    pub trait SealM {}

    impl SealUO for UtcOffset {}
    impl SealODT for OffsetDateTime {}
    impl SealM for Month {}
}

/// Extension trait for constructing a [Month](time::Month) from an index.
pub trait MonthExt: sealed::SealM {
    /// Constructs a month from its index. Returns None if the index is unknown.
    /// 
    /// # Arguments
    ///
    /// * `index`: the month index between 1 and 12.
    ///
    fn from_index(index: u8) -> Option<Month>;
}

impl MonthExt for Month {
    fn from_index(index: u8) -> Option<Month> {
        match index {
            1 => Some(Month::January),
            2 => Some(Month::February),
            3 => Some(Month::March),
            4 => Some(Month::April),
            5 => Some(Month::May),
            6 => Some(Month::June),
            7 => Some(Month::July),
            8 => Some(Month::August),
            9 => Some(Month::September),
            10 => Some(Month::October),
            11 => Some(Month::November),
            12 => Some(Month::December),
            _ => None
        }
    }
}

/// Extension trait for a proper current_local_offset over [UtcOffset](time::UtcOffset).
pub trait LocalUtcOffset: sealed::SealUO {
    /// Attempts to obtain the system’s current UTC offset. If the offset cannot be determined, None is returned.
    ///
    /// # Platform specific behavior
    ///
    /// - On unix, this reads and decodes the /etc/localtime file.
    /// - On windows, this calls [GetTimeZoneInformation](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/nf-timezoneapi-gettimezoneinformation) and reads the **Bias** field of the structure.
    fn current_local_offset() -> Option<UtcOffset>;
}

/// Extension trait for a proper now_local over [OffsetDateTime](time::OffsetDateTime).
pub trait LocalOffsetDateTime: sealed::SealODT {
    /// Attempts to create a new OffsetDateTime with the current date and time in the local offset. If the offset cannot be determined, None is returned.
    ///
    /// # Platform specific behavior
    ///
    /// - On unix, this reads and decodes the /etc/localtime file.
    /// - On windows, this calls [GetTimeZoneInformation](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/nf-timezoneapi-gettimezoneinformation) and reads the **Bias** field of the structure.
    fn now_local() -> Option<OffsetDateTime>;
}

impl LocalUtcOffset for UtcOffset {
    fn current_local_offset() -> Option<UtcOffset> {
        _impl::local_offset_at(&OffsetDateTime::now_utc())
    }
}

impl LocalOffsetDateTime for OffsetDateTime {
    fn now_local() -> Option<OffsetDateTime> {
        let tm = OffsetDateTime::now_utc();
        let offset = _impl::local_offset_at(&tm)?;
        Some(tm.to_offset(offset))
    }
}

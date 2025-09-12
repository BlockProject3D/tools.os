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

//! OS local date/time extensions for time-rs.

#[cfg(unix)]
mod unix;

mod instant;
#[cfg(windows)]
mod windows;

use std::time::Duration;
#[cfg(unix)]
use unix as _impl;

#[cfg(windows)]
use windows as _impl;

use time::{Month, OffsetDateTime, UtcOffset};

mod sealed {
    use std::time::Duration;
    use time::{Month, OffsetDateTime, UtcOffset};

    pub trait SealUO {}
    pub trait SealODT {}
    pub trait SealM {}

    pub trait SealD {}

    impl SealUO for UtcOffset {}
    impl SealODT for OffsetDateTime {}
    impl SealM for Month {}

    impl SealD for Duration {}
}

/// Extension trait for constructing a [Month](Month) from an index.
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
            _ => None,
        }
    }
}

/// Extension trait for a proper current_local_offset over [UtcOffset](UtcOffset).
pub trait LocalUtcOffset: sealed::SealUO {
    /// Attempts to obtain the system’s current UTC offset. If the offset cannot be determined, None is returned.
    ///
    /// # Platform specific behavior
    ///
    /// - On unix, this reads and decodes the /etc/localtime file.
    /// - On windows, this calls [GetTimeZoneInformation](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/nf-timezoneapi-gettimezoneinformation) and reads the **Bias** field of the structure.
    fn current_local_offset() -> Option<UtcOffset>;

    /// Attempts to obtain the system’s UTC offset for the given UTC [OffsetDateTime](OffsetDateTime). If the offset cannot be determined, None is returned.
    ///
    /// This searches for a matching offset in UTC time for the given input datetime.
    ///
    /// # Platform specific behavior
    ///
    /// - On unix, this reads and decodes the /etc/localtime file.
    /// - On windows, this calls [GetTimeZoneInformation](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/nf-timezoneapi-gettimezoneinformation) and reads the **Bias** field of the structure.
    fn local_offset_at(datetime: OffsetDateTime) -> Option<UtcOffset>;
}

/// Extension trait for a proper now_local over [OffsetDateTime](OffsetDateTime).
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
    #[inline]
    fn current_local_offset() -> Option<UtcOffset> {
        _impl::local_offset_at(&OffsetDateTime::now_utc())
    }

    #[inline]
    fn local_offset_at(datetime: OffsetDateTime) -> Option<UtcOffset> {
        _impl::local_offset_at(&datetime)
    }
}

impl LocalOffsetDateTime for OffsetDateTime {
    fn now_local() -> Option<OffsetDateTime> {
        let tm = OffsetDateTime::now_utc();
        let offset = _impl::local_offset_at(&tm)?;
        Some(tm.to_offset(offset))
    }
}

/// This trait is a hack because Rust decided to reject new_unchecked on Duration.
pub trait DurationNewUnchecked: sealed::SealD {
    /// Unsafely construct a [Duration] object.
    ///
    /// # Arguments
    ///
    /// * `secs`: the seconds part.
    /// * `subsec_nanos`: the sub-nanoseconds part
    ///
    /// returns: Duration
    ///
    /// # Safety
    ///
    /// This is insta-UB if subsec_nanos is >= 1000000000.
    unsafe fn new_unchecked(secs: u64, subsec_nanos: u32) -> Duration;
}

impl DurationNewUnchecked for Duration {
    unsafe fn new_unchecked(secs: u64, subsec_nanos: u32) -> Duration {
        const NANOS_PER_SEC: u32 = 1000000000;
        if subsec_nanos >= NANOS_PER_SEC {
            unsafe { std::hint::unreachable_unchecked() }
        }
        Duration::new(secs, subsec_nanos)
    }
}

/// This is a replacement of [Instant](std::time::Instant) for real-time systems
///
/// # Platform specific behavior
///
/// - On all unixes (including macOS), this uses `clock_gettime` with CLOCK_MONOTONIC_RAW
///   instead of `CLOCK_MONOTONIC` which Rust decided not to do and leave broken (see
///   https://github.com/rust-lang/rust/issues/77807).
/// - On windows, this falls back to [Instant](std::time::Instant) which uses the WinAPI `QueryPerformanceCounter`.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Instant(instant::Instant);

impl Instant {
    /// Creates a new [Instant] to measure performance or time in real-time systems. This instant is
    /// monotonic and guaranteed to not be skewed by NTP adjustments.
    #[inline(always)]
    pub fn now() -> Self {
        Self(instant::Instant::now())
    }

    /// Measure the time elapsed since this [Instant] was created.
    #[inline(always)]
    pub fn elapsed(&self) -> Duration {
        self.0.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use time::{OffsetDateTime, UtcOffset};

    use crate::time::{Instant, LocalUtcOffset};

    use super::LocalOffsetDateTime;

    #[test]
    fn current_offset() {
        let offset = UtcOffset::current_local_offset();
        println!("Offset: {:?}", offset)
    }

    #[test]
    fn now_local() {
        let date = OffsetDateTime::now_local();
        println!("Date: {:?}", date)
    }

    #[test]
    fn instant() {
        let time = Instant::now();
        //nanosleep is awfully broken...
        std::thread::sleep(std::time::Duration::from_millis(8));
        let elapsed = time.elapsed();
        println!("{:?}", elapsed);
        assert!(elapsed >= std::time::Duration::from_millis(8));
    }
}

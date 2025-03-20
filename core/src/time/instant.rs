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

use crate::time::DurationNewUnchecked;
#[cfg(unix)]
use libc::{clock_gettime, timespec, CLOCK_MONOTONIC_RAW};
#[cfg(unix)]
use std::cmp::Ordering;
#[cfg(unix)]
use std::hash::{Hash, Hasher};
use std::time::Duration;

#[cfg(unix)]
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Instant(timespec);

#[cfg(unix)]
impl PartialEq for Instant {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0.tv_sec == other.0.tv_sec && self.0.tv_nsec == other.0.tv_nsec
    }
}

#[cfg(unix)]
impl PartialOrd for Instant {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(unix)]
impl Eq for Instant {}

#[cfg(unix)]
impl Ord for Instant {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .tv_sec
            .cmp(&other.0.tv_sec)
            .cmp(&self.0.tv_nsec.cmp(&other.0.tv_nsec))
    }
}

#[cfg(unix)]
impl Hash for Instant {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.tv_sec.hash(state);
        self.0.tv_nsec.hash(state);
    }
}

#[cfg(windows)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Instant(std::time::Instant);

#[cfg(windows)]
impl Instant {
    #[inline(always)]
    pub fn now() -> Self {
        Self(std::time::Instant::now())
    }

    #[inline(always)]
    pub fn elapsed(&self) -> Duration {
        self.0.elapsed()
    }
}

#[cfg(unix)]
impl Instant {
    pub fn now() -> Self {
        let mut t = timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        unsafe { clock_gettime(CLOCK_MONOTONIC_RAW, &mut t) };
        Self(t)
    }

    pub fn elapsed(&self) -> Duration {
        let mut other = timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        unsafe { clock_gettime(CLOCK_MONOTONIC_RAW, &mut other) };
        unsafe {
            Duration::new_unchecked(
                (other.tv_sec - self.0.tv_sec) as _,
                (other.tv_nsec - self.0.tv_nsec) as _,
            )
        }
    }
}

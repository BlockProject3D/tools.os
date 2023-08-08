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

use std::{cmp::Ordering, fs::File};

use time::{OffsetDateTime, UtcOffset};

use super::tzif::{LeapSecondRecord, TZIF};

//Imported from https://github.com/Yuri6037/time-tz/blob/master/src/binary_search.rs
pub fn binary_search<F: Fn(usize) -> Ordering>(start: usize, end: usize, cmp: F) -> Option<usize> {
    if start >= end {
        return None;
    }
    let half = (end - start) / 2;
    let mid = start + half;
    match cmp(mid) {
        Ordering::Greater => binary_search(start, mid, cmp),
        Ordering::Equal => Some(mid),
        Ordering::Less => binary_search(mid + 1, end, cmp),
    }
}

struct Span {
    start: Option<i64>,
    end: Option<i64>
}

impl Span {
    pub fn new(start: Option<i64>, end: Option<i64>) -> Span {
        Span {
            start,
            end
        }
    }

    pub fn cmp(&self, x: i64) -> Ordering {
        match (self.start, self.end) {
            (Some(a), Some(b)) if a <= x && x < b => Ordering::Equal,
            (Some(a), Some(b)) if a <= x && b <= x => Ordering::Less,
            (Some(_), Some(_)) => Ordering::Greater,
            (Some(a), None) if a <= x => Ordering::Equal,
            (Some(_), None) => Ordering::Greater,
            (None, Some(b)) if b <= x => Ordering::Less,
            (None, Some(_)) => Ordering::Equal,
            (None, None) => Ordering::Equal,
        }    
    }
}

impl From<(&[LeapSecondRecord], usize)> for Span {
    fn from((records, i): (&[LeapSecondRecord], usize)) -> Self {
        let start = records[i].occurrence;
        let end = if i >= records.len() {
            None
        } else {
            Some(records[i + 1].occurrence)
        };
        Self::new(Some(start), end)
    }
}

impl From<(&[i64], usize)> for Span {
    fn from((records, i): (&[i64], usize)) -> Self {
        let start = records[i];
        let end = if i >= records.len() {
            None
        } else {
            Some(records[i + 1])
        };
        Self::new(Some(start), end)
    }
}

pub fn local_offset_at(tm: &OffsetDateTime) -> Option<UtcOffset> {
    let mut utc = tm.unix_timestamp();
    let file = File::open("/etc/localtime").ok()?;
    let data = TZIF::read(file).ok()?;
    let block = data.block_v2p.map(|v| v.data).unwrap_or_else(|| data.block_v1.data);
    //Apply leap second correction if any for the given timestamp
    if let Some(i) = binary_search(0, block.leap_second_records.len(), |i| Span::from((&*block.leap_second_records, i)).cmp(utc)) {
        utc += block.leap_second_records[i].correction as i64;
    }
    let i = binary_search(0, block.transition_times.len(), |i| Span::from((&*block.transition_times, i)).cmp(utc))
        .unwrap_or(0);
    let offset = UtcOffset::from_whole_seconds(block.local_time_type_records.get(*block.transition_types.get(i)? as usize)?.utoff).ok()?;
    Some(offset)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_binary_search() {
        assert_eq!(super::binary_search(0, 8, |x| x.cmp(&6)), Some(6));
        assert_eq!(super::binary_search(0, 5000, |x| x.cmp(&1337)), Some(1337));
        assert_eq!(super::binary_search(0, 5000, |x| x.cmp(&9000)), None);
        assert_eq!(super::binary_search(30, 50, |x| x.cmp(&42)), Some(42));
        assert_eq!(super::binary_search(300, 500, |x| x.cmp(&42)), None);
        assert_eq!(
            super::binary_search(0, 500, |x| if x < 42 {
                super::Ordering::Less
            } else {
                super::Ordering::Greater
            }),
            None
        );
    }    
}

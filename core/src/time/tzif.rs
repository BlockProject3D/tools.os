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

//! A simple TZIF reader module used to get UTC to local offset.
//!
//! **See [RFC](https://www.rfc-editor.org/rfc/rfc8536.html)**

use std::{fmt::Display, io::Read};

/// A series of six-octet records specifying a local time type.
/// The number of records is specified by the "typecnt" field in the header.
/// Each record has the following format (the lengths of multi-octet fields are shown in parentheses).
pub struct LocalTimeTypeRecord {
    /// A four-octet signed integer specifying the number of seconds to be added to UT in order to determine local time.
    pub utoff: i32,

    /// A one-octet value indicating whether local time should be considered Daylight Saving Time (DST).
    pub dst: bool,

    /// A one-octet unsigned integer specifying a zero-based index into the series of time zone designation octets,
    /// thereby selecting a particular designation string.
    pub idx: u8,
}

/// A series of eight- or twelve-octet records specifying the corrections that need to be applied to UTC in order to determine TAI.
pub struct LeapSecondRecord {
    /// A four- or eight-octet UNIX leap time value specifying the time at which a leap-second correction occurs.
    pub occurrence: i64,

    /// A four-octet signed integer specifying the value of LEAPCORR on or after the occurrence.
    pub correction: i32,
}

/// Possible errors when parsing TZIF.
#[derive(Debug)]
pub enum Error {
    /// An Io error.
    Io(std::io::Error),

    /// The signature of the file cannot be recognized.
    InvalidSignature,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "io error: {}", e),
            Error::InvalidSignature => f.write_str("invalid TZIF signature"),
        }
    }
}

impl std::error::Error for Error {}

/// A data block.
pub struct Data {
    /// A series of four- or eight-octet UNIX leap-time values sorted in strictly ascending order.
    pub transition_times: Vec<i64>,

    /// A series of one-octet unsigned integers specifying the type of local time of the corresponding transition time.
    pub transition_types: Vec<u8>,

    /// A series of six-octet records specifying a local time type.
    pub local_time_type_records: Vec<LocalTimeTypeRecord>,

    /// A series of eight- or twelve-octet records specifying the corrections that need to be applied to UTC in order to determine TAI.
    pub leap_second_records: Vec<LeapSecondRecord>,
}

/// TZIF header.
pub struct Header {
    /// Block version.
    pub version: u8, //4

    /// A four-octet unsigned integer specifying the number of UT/local indicators contained in the data block -- MUST either be zero or equal to "typecnt".
    pub isutcnt: u32, //20..24

    /// A four-octet unsigned integer specifying the number of standard/wall indicators contained in the data block -- MUST either be zero or equal to "typecnt".
    pub isstdcnt: u32, //24..28

    /// A four-octet unsigned integer specifying the number of leap-second records contained in the data block.
    pub leapcnt: u32, //28..32

    /// A four-octet unsigned integer specifying the number of transition times contained in the data block.
    pub timecnt: u32, //32..36

    /// A four-octet unsigned integer specifying the number of local time type records contained in the data block -- MUST NOT be zero.
    pub typecnt: u32, //36..40

    /// A four-octet unsigned integer specifying the total number of octets used by the set of time zone designations contained in the data block - MUST NOT be zero.
    pub charcnt: u32, //40..44
}

/// Combined header and data block.
///
/// According to RFC, there can be currently 2 blocks: 1 for V1 header/data and 1 for V2+ header/data.
pub struct Block {
    /// The header block.
    pub header: Header,

    /// The associated data block.
    pub data: Data,
}

/// Simplified TZIF decoded structure.
pub struct TZIF {
    /// The V1 combined header/data block.
    pub block_v1: Block,

    /// The V2+ combined header/data block.
    pub block_v2p: Option<Block>,
}

impl Header {
    fn time_size(&self) -> usize {
        match self.version {
            0x00 => 4,
            _ => 8,
        }
    }

    fn read<R: Read>(mut reader: R) -> Result<Header, Error> {
        let mut header: [u8; 44] = [0; 44];
        reader.read_exact(&mut header).map_err(Error::Io)?;
        if &header[0..4] != b"TZif" {
            return Err(Error::InvalidSignature);
        }
        unsafe {
            Ok(Header {
                version: header[4],
                isutcnt: u32::from_be_bytes(header[20..24].try_into().unwrap_unchecked()),
                isstdcnt: u32::from_be_bytes(header[24..28].try_into().unwrap_unchecked()),
                leapcnt: u32::from_be_bytes(header[28..32].try_into().unwrap_unchecked()),
                timecnt: u32::from_be_bytes(header[32..36].try_into().unwrap_unchecked()),
                typecnt: u32::from_be_bytes(header[36..40].try_into().unwrap_unchecked()),
                charcnt: u32::from_be_bytes(header[40..44].try_into().unwrap_unchecked()),
            })
        }
    }
}

impl Data {
    fn read<R: Read>(mut reader: R, header: &Header) -> Result<Data, Error> {
        let size = header.time_size();
        let mut transition_times = vec![0; size * header.timecnt as usize];
        reader
            .read_exact(&mut transition_times)
            .map_err(Error::Io)?;
        let mut transition_types = vec![0; header.timecnt as usize];
        reader
            .read_exact(&mut transition_types)
            .map_err(Error::Io)?;
        let mut local_time_type_records = vec![0; 6 * header.typecnt as usize];
        reader
            .read_exact(&mut local_time_type_records)
            .map_err(Error::Io)?;
        let mut time_zone_designations = vec![0; header.charcnt as usize];
        reader
            .read_exact(&mut time_zone_designations)
            .map_err(Error::Io)?;
        let mut leap_second_records = vec![0; (size + 4) * header.leapcnt as usize];
        reader
            .read_exact(&mut leap_second_records)
            .map_err(Error::Io)?;
        let mut std_wall_indicators = vec![0; header.isstdcnt as usize];
        reader
            .read_exact(&mut std_wall_indicators)
            .map_err(Error::Io)?;
        let mut ut_indicators = vec![0; header.isutcnt as usize];
        reader.read_exact(&mut ut_indicators).map_err(Error::Io)?;
        unsafe {
            let local_time_type_records = local_time_type_records
                .as_slice()
                .chunks(6)
                .map(|v| LocalTimeTypeRecord {
                    utoff: i32::from_be_bytes(v[0..4].try_into().unwrap_unchecked()),
                    dst: v[4] == 1,
                    idx: v[5],
                })
                .collect();
            if header.version == 0x00 {
                Ok(Data {
                    transition_types,
                    transition_times: transition_times
                        .as_slice()
                        .chunks(4)
                        .map(|v| i32::from_be_bytes(v.try_into().unwrap_unchecked()) as i64)
                        .collect(),
                    leap_second_records: leap_second_records
                        .as_slice()
                        .chunks(8)
                        .map(|v| LeapSecondRecord {
                            occurrence: i32::from_be_bytes(v[0..4].try_into().unwrap_unchecked())
                                as i64,
                            correction: i32::from_be_bytes(v[4..8].try_into().unwrap_unchecked()),
                        })
                        .collect(),
                    local_time_type_records,
                })
            } else {
                Ok(Data {
                    transition_types,
                    transition_times: transition_times
                        .as_slice()
                        .chunks(8)
                        .map(|v| i64::from_be_bytes(v.try_into().unwrap_unchecked()))
                        .collect(),
                    leap_second_records: leap_second_records
                        .as_slice()
                        .chunks(12)
                        .map(|v| LeapSecondRecord {
                            occurrence: i64::from_be_bytes(v[0..8].try_into().unwrap_unchecked()),
                            correction: i32::from_be_bytes(v[8..12].try_into().unwrap_unchecked()),
                        })
                        .collect(),
                    local_time_type_records,
                })
            }
        }
    }
}

impl TZIF {
    /// Reads and decodes a TZIF stream.
    ///
    /// # Arguments
    ///
    /// * `reader`: the [Read](std::io::Read) to read and decode from.
    ///
    /// # Errors
    ///
    /// This function returns an [Error](self::Error) if the simplified TZIF structure could not be decoded.
    pub fn read<R: Read>(mut reader: R) -> Result<TZIF, Error> {
        let mut header_v1 = Header::read(&mut reader)?;
        header_v1.version = 0x00; //RFC is badly broken it says bullshit.
        let block_v1 = Block {
            data: Data::read(&mut reader, &header_v1)?,
            header: header_v1,
        };
        let block_v2p = match Header::read(&mut reader) {
            Ok(header_v2) => Some(Block {
                data: Data::read(&mut reader, &header_v2)?,
                header: header_v2,
            }),
            _ => None,
        };
        Ok(TZIF {
            block_v1,
            block_v2p,
        })
    }
}

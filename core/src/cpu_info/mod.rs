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

//! This module contains tools to obtain information about the current system.

/// The CPU information structure.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CpuInfo {
    /// The name of the CPU (aka brand string).
    pub name: String,

    /// The number of cores on the physical package.
    pub core_count: u32,
}

//Linux and Windows
//x86 & x86-64 -> cpuid instruction (rust-cpuid library / get_processor_brand_string() and max_cores_for_package().or(max_cores_for_cache()))
//other -> None

#[cfg(any(
    target_vendor = "apple",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
mod bsd;

//if vendor != apple && os != bsd* && (arch == x86 || arch == x86_64)
#[cfg(all(
    not(any(
        target_vendor = "apple",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )),
    any(target_arch = "x86", target_arch = "x86_64")
))]
mod x86_64;

#[cfg(any(
    target_vendor = "apple",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
use bsd as _impl;

//if vendor != apple && os != bsd* && (arch == x86 || arch == x86_64)
#[cfg(all(
    not(any(
        target_vendor = "apple",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )),
    any(target_arch = "x86", target_arch = "x86_64")
))]
use x86_64 as _impl;

//if vendor != apple && os != bsd* && arch != x86 && arch != x86_64
#[cfg(all(
    not(any(
        target_vendor = "apple",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )),
    not(any(target_arch = "x86", target_arch = "x86_64"))
))]
mod unknown;

//if vendor != apple && os != bsd* && arch != x86 && arch != x86_64
#[cfg(all(
    not(any(
        target_vendor = "apple",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )),
    not(any(target_arch = "x86", target_arch = "x86_64"))
))]
use unknown as _impl;

/// Reads CPU information from the current system, returns None if no CPU information could be found on the current system.
///
/// # Platform specific behavior
///
/// - On macOS and iOS, this function calls *sysctlbyname* with key names `machdep.cpu.core_count` and `machdep.cpu.brand_string`.
///
/// - On other BSD systems, this function calls *sysctlbyname* with key names `hw.ncpu` and `hw.model`.
///
/// - On any x86_64 system except BSD and macos, this function uses the cpuid instruction through the raw-cpuid library.
///
/// - This function returns None for any other system not present in this list.
pub fn read_cpu_info() -> Option<CpuInfo> {
    _impl::read_cpu_info()
}

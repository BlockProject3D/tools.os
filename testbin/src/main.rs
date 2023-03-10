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

use std::io::{BufRead, BufReader};
use std::path::Path;
use bp3d_os::assets;
use bp3d_os::open;

fn ensure_yes(str: &str, func: &str) {
    println!("{}", str);
    let mut buffer = BufReader::new(std::io::stdin()).lines();
    let line = buffer.next().unwrap().unwrap();
    if line == "yes" {
        println!("{} => ok", func);
    } else {
        panic!("{} => fail", func);
    }
}

fn main() {
    //There is no Assets folder so this should just return None
    assert!(assets::get_app_bundled_asset("file.txt").is_none());

    let url = open::Url::try_from("https://rust-lang.org").expect("Failed to parse valid address!");
    assert!(open::open(url));
    ensure_yes("Did your browser has opened the rust-lang.org website?", "open::open(Url)");
    assert!(open::open(Path::new(".")));
    ensure_yes("Did your file explorer open to the current working directory?", "open::open(Path)");
    assert!(open::show_in_files([Path::new("/Users/yuri/Projects/tools.os/Cargo.toml"), Path::new("/Users/yuri/Projects/tools.os/Cargo.lock")].into_iter()));
    ensure_yes("Did your file explorer open to the current working directory selecting both Cargo files?", "open::show_in_files(Path)");
}

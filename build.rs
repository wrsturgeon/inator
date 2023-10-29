/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Basic CI checks that would be a pain in the ass to write with a shell.

use std::{
    fs::{self, File},
    io,
    path::Path,
};

const MPL_HEADER: &[u8] = b"/*\n * This Source Code Form is subject to the terms of the Mozilla Public\n * License, v. 2.0. If a copy of the MPL was not distributed with this\n * file, You can obtain one at https://mozilla.org/MPL/2.0/.\n */\n\n";

fn main() -> io::Result<()> {
    check(Path::new(r"build.rs"))?;
    check(Path::new(r"src"))?;
    Ok(())
}

fn check(file: &Path) -> io::Result<()> {
    if file.is_dir() {
        for f in fs::read_dir(file)? {
            check(&f?.path())?
        }
        Ok(())
    } else {
        let mut read = io::BufReader::with_capacity(MPL_HEADER.len(), File::open(file)?);
        if io::BufRead::fill_buf(&mut read)? == MPL_HEADER {
            Ok(())
        } else {
            panic!("{file:?} is missing the verbatim MPL comment (must start at the very first character, and must be followed by a newline). Please copy and paste it from any other file.")
        }
    }
}

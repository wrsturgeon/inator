/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Basic CI checks that would be a pain in the ass to write with a shell.

const MPL_HEADER: &[u8] = b"/*\n * This Source Code Form is subject to the terms of the Mozilla Public\n * License, v. 2.0. If a copy of the MPL was not distributed with this\n * file, You can obtain one at https://mozilla.org/MPL/2.0/.\n */\n\n";

fn main() -> std::io::Result<()> {
    check(std::path::Path::new(r"build.rs"))?;
    check(std::path::Path::new(r"src"))?;
    Ok(())
}

fn check(file: &std::path::Path) -> std::io::Result<()> {
    if file.is_dir() {
        for f in std::fs::read_dir(file)? {
            check(&f?.path())?
        }
        Ok(())
    } else {
        let mut read =
            std::io::BufReader::with_capacity(MPL_HEADER.len(), std::fs::File::open(file)?);
        if std::io::BufRead::fill_buf(&mut read)? == MPL_HEADER {
            Ok(())
        } else {
            panic!("{file:?} is missing the verbatim MPL comment (must start at the very first character, and must be followed by a newline). Please copy and paste it from any other file.")
        }
    }
}

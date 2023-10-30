use crate::*;
use std::{fs, path::PathBuf};

#[inline]
fn all_test_cases() -> impl Iterator<Item = (PathBuf, Vec<u8>, bool)> {
    fs::read_dir("JSONTestSuite/test_parsing")
        .unwrap()
        .into_iter()
        .filter_map(|r| {
            let file = r.unwrap();
            let os_name = file.file_name();
            let name = os_name.to_str().unwrap();
            let succeed = match &name[..2] {
                "y_" => true,
                "n_" => false,
                "i_" => None?,
                _ => unreachable!(),
            };
            Some((file.path(), fs::read(file.path()).unwrap(), succeed))
        })
}

#[test]
fn entire_test_suite() {
    for (filename, input, should_pass) in all_test_cases() {
        assert_eq!(
            crate::parser::parse(input).is_ok(),
            should_pass,
            "

    FILE:
    {}

    CONTENTS:
    ```
    {}
    ```
    ",
            filename.to_str().unwrap(),
            fs::read_to_string(filename.clone()).unwrap(),
        );
    }
}

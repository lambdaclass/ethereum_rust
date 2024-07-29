use std::path::Path;

use ef_tests::test_runner::{execute_test, parse_test_file, validate_test};

fn parse_and_execute(path: &Path) -> datatest_stable::Result<()> {
    let tests = parse_test_file(path);

    for (test_key, test) in tests {
        validate_test(&test);
        execute_test(&test_key, &test)
    }
    Ok(())
}

#[allow(unused)]
fn parse_and_validate(path: &Path) -> datatest_stable::Result<()> {
    let tests = parse_test_file(path);

    for (_k, test) in tests {
        validate_test(&test);
    }
    Ok(())
}

//TODO: eip6780_selfdestruct tests are not passing, probably because they
//      test using several transactions one after the other.
//TODO: eip4844_blobs tests are not passing because they expect exceptions.
datatest_stable::harness!(
    parse_and_execute,
    "vectors/cancun/",
    r"eip1153_tstore/.*/.*\.json",
    parse_and_execute,
    "vectors/cancun/",
    r"eip4788_beacon_root/.*/.*\.json",
    parse_and_execute,
    "vectors/cancun/",
    r"eip5656_mcopy/.*/.*\.json",
    parse_and_execute,
    "vectors/cancun/",
    r"eip7516_blobgasfee/.*/.*\.json"
);

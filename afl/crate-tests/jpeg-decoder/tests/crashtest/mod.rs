use jpeg;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use super::common;

#[test]
fn crashtest() {
    let files = common::test_files(&Path::new("tests").join("crashtest").join("images"));

    for path in &files {
        let file = File::open(path).unwrap();
        let mut decoder = jpeg::Decoder::new(BufReader::new(file));
        let _ = decoder.decode();
    }
}

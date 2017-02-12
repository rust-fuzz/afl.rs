extern crate jpeg_decoder as jpeg;
extern crate png;
extern crate walkdir;

use std::path::Path;
use std::fs::File;

mod common;
mod crashtest;
mod reftest;

#[test]
fn read_info() {
    let path = Path::new("tests").join("reftest").join("images").join("mozilla").join("jpg-progressive.jpg");

    let mut decoder = jpeg::Decoder::new(File::open(&path).unwrap());
    let ref_data = decoder.decode().unwrap();
    let ref_info = decoder.info().unwrap();

    decoder = jpeg::Decoder::new(File::open(&path).unwrap());
    decoder.read_info().unwrap();
    let info = decoder.info().unwrap();
    let data = decoder.decode().unwrap();

    assert_eq!(info, decoder.info().unwrap());
    assert_eq!(info, ref_info);
    assert_eq!(data, ref_data);
}

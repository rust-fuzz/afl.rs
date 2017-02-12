#![feature(plugin)]
#![plugin(afl_plugin)]

extern crate byteorder;
extern crate rayon;

pub use decoder::{Decoder, ImageInfo, PixelFormat};
pub use error::{Error, UnsupportedFeature};

mod decoder;
mod error;
mod huffman;
mod idct;
mod marker;
mod parser;
mod resampler;
mod worker_thread;

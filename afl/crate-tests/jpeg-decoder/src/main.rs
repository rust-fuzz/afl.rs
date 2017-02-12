extern crate afl;
extern crate jpeg_decoder;

fn main() {
    afl::handle_bytes(|bytes| {
        let mut decoder = jpeg_decoder::Decoder::new(&bytes[..]);
        let _ = decoder.read_info();
    });
}

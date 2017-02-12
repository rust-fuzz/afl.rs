extern crate jpeg_decoder as jpeg;
extern crate png;

use png::HasParameters;
use std::env;
use std::fs::File;
use std::io::BufReader;

fn print_usage(name: &str) {
    println!("Usage: {} [--output <pngfile>] <file>", name);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut arg_index = 1;
    let mut output = None;
    let mut input = None;

    while arg_index < args.len() {
        match &args[arg_index][..] {
            "-o" | "--output" => {
                output = Some(args[arg_index + 1].clone());
                arg_index += 2;
            },
            _ => {
                if args[arg_index].starts_with("-") {
                    println!("Unknown option: {}", args[arg_index]);
                    print_usage(&args[0]);
                    return;
                }
                if input.is_some() {
                    println!("Only one input is allowed");
                    print_usage(&args[0]);
                    return;
                }

                input = Some(args[arg_index].clone());
                arg_index += 1;
            },
        }
    }

    if input.is_none() {
        println!("No input specified");
        print_usage(&args[0]);
        return;
    }

    let file = File::open(input.unwrap());

    if file.is_err() {
        println!("The specified input does not exist");
        return;
    }

    let mut decoder = jpeg::Decoder::new(BufReader::new(file.unwrap()));
    let mut data = decoder.decode().expect("decode failed");

    if let Some(output) = output {
        let info = decoder.info().unwrap();
        let output_file = File::create(output).unwrap();
        let mut encoder = png::Encoder::new(output_file, info.width as u32, info.height as u32);
        encoder.set(png::BitDepth::Eight);

        match info.pixel_format {
            jpeg::PixelFormat::L8     => encoder.set(png::ColorType::Grayscale),
            jpeg::PixelFormat::RGB24  => encoder.set(png::ColorType::RGB),
            jpeg::PixelFormat::CMYK32 => {
                data = cmyk_to_rgb(&mut data);
                encoder.set(png::ColorType::RGB)
            },
        };

        encoder.write_header().expect("writing png header failed").write_image_data(&data).expect("png encoding failed");
    }
}

fn cmyk_to_rgb(input: &[u8]) -> Vec<u8> {
    let size = input.len() - input.len() / 4;
    let mut output = Vec::with_capacity(size);

    for pixel in input.chunks(4) {
        let c = pixel[0] as f32 / 255.0;
        let m = pixel[1] as f32 / 255.0;
        let y = pixel[2] as f32 / 255.0;
        let k = pixel[3] as f32 / 255.0;

        // CMYK -> CMY
        let c = c * (1.0 - k) + k;
        let m = m * (1.0 - k) + k;
        let y = y * (1.0 - k) + k;

        // CMY -> RGB
        let r = (1.0 - c) * 255.0;
        let g = (1.0 - m) * 255.0;
        let b = (1.0 - y) * 255.0;

        output.push(r as u8);
        output.push(g as u8);
        output.push(b as u8);
    }

    output
}

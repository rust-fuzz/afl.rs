use byteorder::ReadBytesExt;
use error::{Error, Result, UnsupportedFeature};
use huffman::{HuffmanDecoder, HuffmanTable};
use marker::Marker;
use parser::{AdobeColorTransform, AppData, CodingProcess, Component, Dimensions, EntropyCoding, FrameInfo,
             parse_app, parse_com, parse_dht, parse_dqt, parse_dri, parse_sof, parse_sos, ScanInfo};
use rayon::par_iter::*;
use resampler::Resampler;
use std::cmp;
use std::io::Read;
use std::mem;
use std::ops::Range;
use std::sync::Arc;
use std::sync::mpsc::{self, Sender};
use worker_thread::{RowData, spawn_worker_thread, WorkerMsg};

pub const MAX_COMPONENTS: usize = 4;

static UNZIGZAG: [u8; 64] = [
     0,  1,  8, 16,  9,  2,  3, 10,
    17, 24, 32, 25, 18, 11,  4,  5,
    12, 19, 26, 33, 40, 48, 41, 34,
    27, 20, 13,  6,  7, 14, 21, 28,
    35, 42, 49, 56, 57, 50, 43, 36,
    29, 22, 15, 23, 30, 37, 44, 51,
    58, 59, 52, 45, 38, 31, 39, 46,
    53, 60, 61, 54, 47, 55, 62, 63,
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PixelFormat {
    L8,     // Luminance, 8 bits per channel
    RGB24,  // RGB, 8 bits per channel
    CMYK32, // CMYK, 8 bits per channel
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageInfo {
    pub width: u16,
    pub height: u16,
    pub pixel_format: PixelFormat,
}

pub struct Decoder<R> {
    reader: R,

    frame: Option<FrameInfo>,
    dc_huffman_tables: Vec<Option<HuffmanTable>>,
    ac_huffman_tables: Vec<Option<HuffmanTable>>,
    quantization_tables: [Option<Arc<[u8; 64]>>; 4],

    restart_interval: u16,
    color_transform: Option<AdobeColorTransform>,
    is_jfif: bool,

    // Used for progressive JPEGs.
    coefficients: Vec<Vec<i16>>,
    // Bitmask of which coefficients has been completely decoded.
    coefficients_finished: [u64; MAX_COMPONENTS],
}

impl<R: Read> Decoder<R> {
    pub fn new(reader: R) -> Decoder<R> {
        Decoder {
            reader: reader,
            frame: None,
            dc_huffman_tables: vec![None, None, None, None],
            ac_huffman_tables: vec![None, None, None, None],
            quantization_tables: [None, None, None, None],
            restart_interval: 0,
            color_transform: None,
            is_jfif: false,
            coefficients: Vec::new(),
            coefficients_finished: [0; MAX_COMPONENTS],
        }
    }

    pub fn info(&self) -> Option<ImageInfo> {
        match self.frame {
            Some(ref frame) => {
                let pixel_format = match frame.components.len() {
                    1 => PixelFormat::L8,
                    3 => PixelFormat::RGB24,
                    4 => PixelFormat::CMYK32,
                    _ => panic!(),
                };

                Some(ImageInfo {
                    width: frame.image_size.width,
                    height: frame.image_size.height,
                    pixel_format: pixel_format,
                })
            },
            None => None,
        }
    }

    pub fn read_info(&mut self) -> Result<()> {
        self.decode_internal(true).map(|_| ())
    }

    pub fn decode(&mut self) -> Result<Vec<u8>> {
        self.decode_internal(false)
    }

    fn decode_internal(&mut self, stop_after_metadata: bool) -> Result<Vec<u8>> {
        if stop_after_metadata && self.frame.is_some() {
            // The metadata has already been read.
            return Ok(Vec::new());
        }
        else if self.frame.is_none() && (try!(self.reader.read_u8()) != 0xFF || Marker::from_u8(try!(self.reader.read_u8())) != Some(Marker::SOI)) {
            return Err(Error::Format("first two bytes is not a SOI marker".to_owned()));
        }

        let mut previous_marker = Marker::SOI;
        let mut pending_marker = None;
        let mut worker_chan = None;
        let mut scans_processed = 0;
        let mut planes = vec![Vec::new(); self.frame.as_ref().map_or(0, |frame| frame.components.len())];

        loop {
            let marker = match pending_marker.take() {
                Some(m) => m,
                None => try!(self.read_marker()),
            };

            match marker {
                // Frame header
                Marker::SOF(..) => {
                    // Section 4.10
                    // "An image contains only one frame in the cases of sequential and
                    //  progressive coding processes; an image contains multiple frames for the
                    //  hierarchical mode."
                    if self.frame.is_some() {
                        return Err(Error::Unsupported(UnsupportedFeature::Hierarchical));
                    }

                    let frame = try!(parse_sof(&mut self.reader, marker));
                    let component_count = frame.components.len();

                    if frame.is_differential {
                        return Err(Error::Unsupported(UnsupportedFeature::Hierarchical));
                    }
                    if frame.coding_process == CodingProcess::Lossless {
                        return Err(Error::Unsupported(UnsupportedFeature::Lossless));
                    }
                    if frame.entropy_coding == EntropyCoding::Arithmetic {
                        return Err(Error::Unsupported(UnsupportedFeature::ArithmeticEntropyCoding));
                    }
                    if frame.precision != 8 {
                        return Err(Error::Unsupported(UnsupportedFeature::SamplePrecision(frame.precision)));
                    }
                    if frame.image_size.height == 0 {
                        return Err(Error::Unsupported(UnsupportedFeature::DNL));
                    }
                    if component_count != 1 && component_count != 3 && component_count != 4 {
                        return Err(Error::Unsupported(UnsupportedFeature::ComponentCount(component_count as u8)));
                    }

                    // Make sure we support the subsampling ratios used.
                    let _ = try!(Resampler::new(&frame.components));

                    if frame.coding_process == CodingProcess::DctProgressive {
                        self.coefficients = frame.components.iter().map(|c| {
                            let block_count = c.block_size.width as usize * c.block_size.height as usize;
                            vec![0; block_count * 64]
                        }).collect();
                    }

                    self.frame = Some(frame);

                    if stop_after_metadata {
                        return Ok(Vec::new());
                    }

                    planes = vec![Vec::new(); component_count];
                },

                // Scan header
                Marker::SOS => {
                    if self.frame.is_none() {
                        return Err(Error::Format("scan encountered before frame".to_owned()));
                    }
                    if worker_chan.is_none() {
                        worker_chan = Some(try!(spawn_worker_thread()));
                    }

                    let frame = self.frame.clone().unwrap();
                    let scan = try!(parse_sos(&mut self.reader, &frame));

                    if scan.successive_approximation_low == 0 {
                        for &i in scan.component_indices.iter() {
                            for j in scan.spectral_selection.clone() {
                                self.coefficients_finished[i] |= 1 << j;
                            }
                        }
                    }

                    let is_final_scan = scan.component_indices.iter().all(|&i| self.coefficients_finished[i] == !0);
                    let (marker, data) = try!(self.decode_scan(&frame, &scan, worker_chan.as_ref().unwrap(), is_final_scan));

                    if let Some(data) = data {
                        for (i, plane) in data.into_iter().enumerate().filter(|&(_, ref plane)| !plane.is_empty()) {
                            planes[i] = plane;
                        }
                    }

                    pending_marker = marker;
                    scans_processed += 1;
                },

                // Table-specification and miscellaneous markers
                // Quantization table-specification
                Marker::DQT => {
                    let tables = try!(parse_dqt(&mut self.reader));

                    for (i, &table) in tables.into_iter().enumerate() {
                        if let Some(table) = table {
                            let mut unzigzagged_table = [0u8; 64];

                            for j in 0 .. 64 {
                                unzigzagged_table[UNZIGZAG[j] as usize] = table[j];
                            }

                            self.quantization_tables[i] = Some(Arc::new(unzigzagged_table));
                        }
                    }
                },
                // Huffman table-specification
                Marker::DHT => {
                    let is_baseline = self.frame.as_ref().map(|frame| frame.is_baseline);
                    let (dc_tables, ac_tables) = try!(parse_dht(&mut self.reader, is_baseline));

                    let current_dc_tables = mem::replace(&mut self.dc_huffman_tables, vec![]);
                    self.dc_huffman_tables = dc_tables.into_iter()
                                                      .zip(current_dc_tables.into_iter())
                                                      .map(|(a, b)| a.or(b))
                                                      .collect();

                    let current_ac_tables = mem::replace(&mut self.ac_huffman_tables, vec![]);
                    self.ac_huffman_tables = ac_tables.into_iter()
                                                      .zip(current_ac_tables.into_iter())
                                                      .map(|(a, b)| a.or(b))
                                                      .collect();
                },
                // Arithmetic conditioning table-specification
                Marker::DAC => return Err(Error::Unsupported(UnsupportedFeature::ArithmeticEntropyCoding)),
                // Restart interval definition
                Marker::DRI => self.restart_interval = try!(parse_dri(&mut self.reader)),
                // Comment
                Marker::COM => {
                    let _comment = try!(parse_com(&mut self.reader));
                },
                // Application data
                Marker::APP(..) => {
                    if let Some(data) = try!(parse_app(&mut self.reader, marker)) {
                        match data {
                            AppData::Adobe(color_transform) => self.color_transform = Some(color_transform),
                            AppData::Jfif => {
                                // From the JFIF spec:
                                // "The APP0 marker is used to identify a JPEG FIF file.
                                //     The JPEG FIF APP0 marker is mandatory right after the SOI marker."
                                // Some JPEGs in the wild does not follow this though, so we allow
                                // JFIF headers anywhere APP0 markers are allowed.
                                /*
                                if previous_marker != Marker::SOI {
                                    return Err(Error::Format("the JFIF APP0 marker must come right after the SOI marker".to_owned()));
                                }
                                */

                                self.is_jfif = true;
                            },
                        }
                    }
                },

                // Define number of lines
                Marker::DNL => {
                    // Section B.2.1
                    // "If a DNL segment (see B.2.5) is present, it shall immediately follow the first scan."
                    if previous_marker != Marker::SOS || scans_processed != 1 {
                        return Err(Error::Format("DNL is only allowed immediately after the first scan".to_owned()));
                    }

                    return Err(Error::Unsupported(UnsupportedFeature::DNL));
                },

                // Hierarchical mode markers
                Marker::DHP | Marker::EXP => return Err(Error::Unsupported(UnsupportedFeature::Hierarchical)),

                // End of image
                Marker::EOI => break,

                _ => return Err(Error::Format(format!("{:?} marker found where not allowed", marker))),
            }

            previous_marker = marker;
        }

        if planes.iter().all(|plane| !plane.is_empty()) {
            let frame = self.frame.as_ref().unwrap();
            compute_image(&frame.components, &planes, frame.image_size, self.is_jfif, self.color_transform)
        }
        else {
            Err(Error::Format("no data found".to_owned()))
        }
    }

    fn read_marker(&mut self) -> Result<Marker> {
        if try!(self.reader.read_u8()) != 0xFF {
            return Err(Error::Format("did not find marker where expected".to_owned()));
        }

        let mut byte = try!(self.reader.read_u8());

        // Section B.1.1.2
        // "Any marker may optionally be preceded by any number of fill bytes, which are bytes assigned code X’FF’."
        while byte == 0xFF {
            byte = try!(self.reader.read_u8());
        }

        match byte {
            0x00 => Err(Error::Format("FF 00 found where marker was expected".to_owned())),
            _    => Ok(Marker::from_u8(byte).unwrap()),
        }
    }

    fn decode_scan(&mut self,
                   frame: &FrameInfo,
                   scan: &ScanInfo,
                   worker_chan: &Sender<WorkerMsg>,
                   produce_data: bool)
                   -> Result<(Option<Marker>, Option<Vec<Vec<u8>>>)> {
        assert!(scan.component_indices.len() <= MAX_COMPONENTS);

        let components: Vec<Component> = scan.component_indices.iter()
                                                               .map(|&i| frame.components[i].clone())
                                                               .collect();

        // Verify that all required quantization tables has been set.
        if components.iter().any(|component| self.quantization_tables[component.quantization_table_index].is_none()) {
            return Err(Error::Format("use of unset quantization table".to_owned()));
        }

        // Verify that all required huffman tables has been set.
        if scan.spectral_selection.start == 0 &&
                scan.dc_table_indices.iter().any(|&i| self.dc_huffman_tables[i].is_none()) {
            return Err(Error::Format("scan makes use of unset dc huffman table".to_owned()));
        }
        if scan.spectral_selection.end > 1 &&
                scan.ac_table_indices.iter().any(|&i| self.ac_huffman_tables[i].is_none()) {
            return Err(Error::Format("scan makes use of unset ac huffman table".to_owned()));
        }

        if produce_data {
            // Prepare the worker thread for the work to come.
            for (i, component) in components.iter().enumerate() {
                let row_data = RowData {
                    index: i,
                    component: component.clone(),
                    quantization_table: self.quantization_tables[component.quantization_table_index].clone().unwrap(),
                };

                try!(worker_chan.send(WorkerMsg::Start(row_data)));
            }
        }

        let blocks_per_mcu: Vec<u16> = components.iter()
                                                 .map(|c| c.horizontal_sampling_factor as u16 * c.vertical_sampling_factor as u16)
                                                 .collect();
        let is_progressive = frame.coding_process == CodingProcess::DctProgressive;
        let is_interleaved = components.len() > 1;
        let mut dummy_block = [0i16; 64];
        let mut huffman = HuffmanDecoder::new();
        let mut dc_predictors = [0i16; MAX_COMPONENTS];
        let mut mcus_left_until_restart = self.restart_interval;
        let mut expected_rst_num = 0;
        let mut eob_run = 0;
        let mut mcu_row_coefficients = Vec::with_capacity(components.len());

        if produce_data && !is_progressive {
            for component in &components {
                let coefficients_per_mcu_row = component.block_size.width as usize * component.vertical_sampling_factor as usize * 64;
                mcu_row_coefficients.push(vec![0i16; coefficients_per_mcu_row]);
            }
        }

        for mcu_y in 0 .. frame.mcu_size.height {
            for mcu_x in 0 .. frame.mcu_size.width {
                for (i, component) in components.iter().enumerate() {
                    for j in 0 .. blocks_per_mcu[i] {
                        let (block_x, block_y) = if is_interleaved {
                            // Section A.2.3
                            (mcu_x * component.horizontal_sampling_factor as u16 + j % component.horizontal_sampling_factor as u16,
                             mcu_y * component.vertical_sampling_factor as u16 + j / component.horizontal_sampling_factor as u16)
                        }
                        else {
                            // Section A.2.2

                            let blocks_per_row = component.block_size.width as usize;
                            let block_num = (mcu_y as usize * frame.mcu_size.width as usize +
                                mcu_x as usize) * blocks_per_mcu[i] as usize + j as usize;

                            let x = (block_num % blocks_per_row) as u16;
                            let y = (block_num / blocks_per_row) as u16;

                            if x * 8 >= component.size.width || y * 8 >= component.size.height {
                                continue;
                            }

                            (x, y)
                        };

                        let block_offset = (block_y as usize * component.block_size.width as usize + block_x as usize) * 64;
                        let mcu_row_offset = mcu_y as usize * component.block_size.width as usize * component.vertical_sampling_factor as usize * 64;
                        let coefficients = if is_progressive {
                            &mut self.coefficients[scan.component_indices[i]][block_offset .. block_offset + 64]
                        } else if produce_data {
                            &mut mcu_row_coefficients[i][block_offset - mcu_row_offset .. block_offset - mcu_row_offset + 64]
                        } else {
                            &mut dummy_block[..]
                        };

                        if scan.successive_approximation_high == 0 {
                            let dc_diff = try!(decode_block(&mut self.reader,
                                                            coefficients,
                                                            &mut huffman,
                                                            self.dc_huffman_tables[scan.dc_table_indices[i]].as_ref(),
                                                            self.ac_huffman_tables[scan.ac_table_indices[i]].as_ref(),
                                                            scan.spectral_selection.clone(),
                                                            scan.successive_approximation_low,
                                                            &mut eob_run,
                                                            dc_predictors[i]));
                            dc_predictors[i] += dc_diff;
                        }
                        else {
                            try!(decode_block_successive_approximation(&mut self.reader,
                                                                       coefficients,
                                                                       &mut huffman,
                                                                       self.ac_huffman_tables[scan.ac_table_indices[i]].as_ref(),
                                                                       scan.spectral_selection.clone(),
                                                                       scan.successive_approximation_low,
                                                                       &mut eob_run));
                        }
                    }
                }

                if self.restart_interval > 0 {
                    let is_last_mcu = mcu_x == frame.mcu_size.width - 1 && mcu_y == frame.mcu_size.height - 1;
                    mcus_left_until_restart -= 1;

                    if mcus_left_until_restart == 0 && !is_last_mcu {
                        match huffman.take_marker() {
                            Some(marker) => {
                                match marker {
                                    Marker::RST(n) => {
                                        if n != expected_rst_num {
                                            return Err(Error::Format(format!("found RST {:?} where {:?} was expected", n, expected_rst_num)));
                                        }

                                        expected_rst_num = (expected_rst_num + 1) % 8;
                                    },
                                    _ => return Err(Error::Format(format!("found marker {:?} inside scan where RST was expected", marker))),
                                }
                            },
                            None => return Err(Error::Format("no marker found where RST was expected".to_owned())),
                        }

                        huffman.reset();
                        // Section F.2.1.3.1
                        dc_predictors = [0i16; MAX_COMPONENTS];
                        // Section G.1.2.2
                        eob_run = 0;

                        mcus_left_until_restart = self.restart_interval;
                    }
                }
            }

            if produce_data {
                // Send the coefficients from this MCU row to the worker thread for dequantization and idct.
                for (i, component) in components.iter().enumerate() {
                    let coefficients_per_mcu_row = component.block_size.width as usize * component.vertical_sampling_factor as usize * 64;

                    let row_coefficients = if is_progressive {
                        let offset = mcu_y as usize * coefficients_per_mcu_row;
                        self.coefficients[scan.component_indices[i]][offset .. offset + coefficients_per_mcu_row].to_vec()
                    } else {
                        mem::replace(&mut mcu_row_coefficients[i], vec![0i16; coefficients_per_mcu_row])
                    };

                    try!(worker_chan.send(WorkerMsg::AppendRow((i, row_coefficients))));
                }
            }
        }

        if produce_data {
            // Retrieve all the data from the worker thread.
            let mut data = vec![Vec::new(); frame.components.len()];

            for (i, &component_index) in scan.component_indices.iter().enumerate() {
                let (tx, rx) = mpsc::channel();
                try!(worker_chan.send(WorkerMsg::GetResult((i, tx))));

                data[component_index] = try!(rx.recv());
            }

            Ok((huffman.take_marker(), Some(data)))
        }
        else {
            Ok((huffman.take_marker(), None))
        }
    }
}

fn decode_block<R: Read>(reader: &mut R,
                         coefficients: &mut [i16],
                         huffman: &mut HuffmanDecoder,
                         dc_table: Option<&HuffmanTable>,
                         ac_table: Option<&HuffmanTable>,
                         spectral_selection: Range<u8>,
                         successive_approximation_low: u8,
                         eob_run: &mut u16,
                         dc_predictor: i16) -> Result<i16> {
    debug_assert_eq!(coefficients.len(), 64);

    let mut dc_diff = 0;

    if spectral_selection.start == 0 {
        // Section F.2.2.1
        // Figure F.12
        let value = try!(huffman.decode(reader, dc_table.unwrap()));
        let diff = match value {
            0 => 0,
            _ => {
                // Section F.1.2.1.1
                // Table F.1
                if value > 11 {
                    return Err(Error::Format("invalid DC difference magnitude category".to_owned()));
                }

                try!(huffman.receive_extend(reader, value))
            },
        };

        coefficients[0] = (dc_predictor + diff) << successive_approximation_low;
        dc_diff = diff;
    }

    let mut index = cmp::max(spectral_selection.start, 1);

    if index < spectral_selection.end && *eob_run > 0 {
        *eob_run -= 1;
        return Ok(dc_diff);
    }

    // Section F.1.2.2.1
    while index < spectral_selection.end {
        if let Some((value, run)) = try!(huffman.decode_fast_ac(reader, ac_table.unwrap())) {
            index += run;

            if index >= spectral_selection.end {
                break;
            }

            coefficients[UNZIGZAG[index as usize] as usize] = value << successive_approximation_low;
            index += 1;
        }
        else {
            let byte = try!(huffman.decode(reader, ac_table.unwrap()));
            let r = byte >> 4;
            let s = byte & 0x0f;

            if s == 0 {
                match r {
                    15 => index += 16, // Run length of 16 zero coefficients.
                    _  => {
                        *eob_run = (1 << r) - 1;

                        if r > 0 {
                            *eob_run += try!(huffman.get_bits(reader, r));
                        }

                        break;
                    },
                }
            }
            else {
                index += r;

                if index >= spectral_selection.end {
                    break;
                }

                coefficients[UNZIGZAG[index as usize] as usize] = try!(huffman.receive_extend(reader, s)) << successive_approximation_low;
                index += 1;
            }
        }
    }

    Ok(dc_diff)
}

fn decode_block_successive_approximation<R: Read>(reader: &mut R,
                                                  coefficients: &mut [i16],
                                                  huffman: &mut HuffmanDecoder,
                                                  ac_table: Option<&HuffmanTable>,
                                                  spectral_selection: Range<u8>,
                                                  successive_approximation_low: u8,
                                                  eob_run: &mut u16) -> Result<()> {
    debug_assert_eq!(coefficients.len(), 64);

    let bit = 1 << successive_approximation_low;

    if spectral_selection.start == 0 {
        // Section G.1.2.1

        if try!(huffman.get_bits(reader, 1)) == 1 {
            coefficients[0] |= bit;
        }
    }
    else {
        // Section G.1.2.3

        if *eob_run > 0 {
            *eob_run -= 1;
            try!(refine_non_zeroes(reader, coefficients, huffman, spectral_selection, 64, bit));
            return Ok(());
        }

        let mut index = spectral_selection.start;

        while index < spectral_selection.end {
            let byte = try!(huffman.decode(reader, ac_table.unwrap()));
            let r = byte >> 4;
            let s = byte & 0x0f;

            let mut zero_run_length = r;
            let mut value = 0;

            match s {
                0 => {
                    match r {
                        15 => {
                            // Run length of 16 zero coefficients.
                            // We don't need to do anything special here, zero_run_length is 15
                            // and then value (which is zero) gets written, resulting in 16
                            // zero coefficients.
                        },
                        _ => {
                            *eob_run = (1 << r) - 1;

                            if r > 0 {
                                *eob_run += try!(huffman.get_bits(reader, r));
                            }

                            // Force end of block.
                            zero_run_length = 64;
                        },
                    }
                },
                1 => {
                    if try!(huffman.get_bits(reader, 1)) == 1 {
                        value = bit;
                    }
                    else {
                        value = -bit;
                    }
                },
                _ => return Err(Error::Format("unexpected huffman code".to_owned())),
            }

            let range = Range {
                start: index,
                end: spectral_selection.end,
            };
            index = try!(refine_non_zeroes(reader, coefficients, huffman, range, zero_run_length, bit));

            if value != 0 {
                coefficients[UNZIGZAG[index as usize] as usize] = value;
            }

            index += 1;
        }
    }

    Ok(())
}

fn refine_non_zeroes<R: Read>(reader: &mut R,
                              coefficients: &mut [i16],
                              huffman: &mut HuffmanDecoder,
                              range: Range<u8>,
                              zrl: u8,
                              bit: i16) -> Result<u8> {
    debug_assert_eq!(coefficients.len(), 64);

    let last = range.end - 1;
    let mut zero_run_length = zrl;

    for i in range {
        let index = UNZIGZAG[i as usize] as usize;

        if coefficients[index] == 0 {
            if zero_run_length == 0 {
                return Ok(i);
            }

            zero_run_length -= 1;
        }
        else if try!(huffman.get_bits(reader, 1)) == 1 && coefficients[index] & bit == 0 {
            if coefficients[index] > 0 {
                coefficients[index] += bit;
            }
            else {
                coefficients[index] -= bit;
            }
        }
    }

    Ok(last)
}

fn compute_image(components: &[Component],
                 data: &[Vec<u8>],
                 output_size: Dimensions,
                 is_jfif: bool,
                 color_transform: Option<AdobeColorTransform>) -> Result<Vec<u8>> {
    if data.iter().any(|data| data.is_empty()) {
        return Err(Error::Format("not all components has data".to_owned()));
    }

    if components.len() == 1 {
        let component = &components[0];

        if component.size.width % 8 == 0 && component.size.height % 8 == 0 {
            return Ok(data[0].clone())
        }

        let mut buffer = vec![0u8; component.size.width as usize * component.size.height as usize];
        let line_stride = component.block_size.width as usize * 8;

        for y in 0 .. component.size.height as usize {
            for x in 0 .. component.size.width as usize {
                buffer[y * component.size.width as usize + x] = data[0][y * line_stride + x];
            }
        }

        Ok(buffer)
    }
    else {
        let color_convert_func = try!(choose_color_convert_func(components.len(), is_jfif, color_transform));
        let resampler = try!(Resampler::new(components));
        let line_size = output_size.width as usize * components.len();
        let mut image = vec![0u8; line_size * output_size.height as usize];

        image.chunks_mut(line_size)
             .collect::<Vec<&mut [u8]>>()
             .par_iter_mut()
             .weight_max()
             .enumerate()
             .for_each(|(row, line)| {
                 resampler.resample_and_interleave_row(data, row, output_size.width as usize, *line);
                 color_convert_func(*line, output_size.width as usize);
             });

        Ok(image)
    }
}

fn choose_color_convert_func(component_count: usize,
                             _is_jfif: bool,
                             color_transform: Option<AdobeColorTransform>)
                             -> Result<fn(&mut [u8], usize)> {
    match component_count {
        3 => {
            // http://www.sno.phy.queensu.ca/~phil/exiftool/TagNames/JPEG.html#Adobe
            // Unknown means the data is RGB, so we don't need to perform any color conversion on it.
            if color_transform == Some(AdobeColorTransform::Unknown) {
                Ok(color_convert_line_null)
            }
            else {
                Ok(color_convert_line_ycbcr)
            }
        },
        4 => {
            // http://www.sno.phy.queensu.ca/~phil/exiftool/TagNames/JPEG.html#Adobe
            match color_transform {
                Some(AdobeColorTransform::Unknown) => Ok(color_convert_line_cmyk),
                Some(_) => Ok(color_convert_line_ycck),
                None => Err(Error::Format("4 components without Adobe APP14 metadata to tell color space".to_owned())),
            }
        },
        _ => panic!(),
    }
}

fn color_convert_line_null(_data: &mut [u8], _width: usize) {
}

fn color_convert_line_ycbcr(data: &mut [u8], width: usize) {
    for i in 0 .. width {
        let (r, g, b) = ycbcr_to_rgb(data[i * 3], data[i * 3 + 1], data[i * 3 + 2]);

        data[i * 3]     = r;
        data[i * 3 + 1] = g;
        data[i * 3 + 2] = b;
    }
}

fn color_convert_line_ycck(data: &mut [u8], width: usize) {
    for i in 0 .. width {
        let (r, g, b) = ycbcr_to_rgb(data[i * 4], data[i * 4 + 1], data[i * 4 + 2]);
        let k = data[i * 4 + 3];

        data[i * 4]     = r;
        data[i * 4 + 1] = g;
        data[i * 4 + 2] = b;
        data[i * 4 + 3] = 255 - k;
    }
}

fn color_convert_line_cmyk(data: &mut [u8], width: usize) {
    for i in 0 .. width {
        data[i * 4]     = 255 - data[i * 4];
        data[i * 4 + 1] = 255 - data[i * 4 + 1];
        data[i * 4 + 2] = 255 - data[i * 4 + 2];
        data[i * 4 + 3] = 255 - data[i * 4 + 3];
    }
}

// ITU-R BT.601
fn ycbcr_to_rgb(y: u8, cb: u8, cr: u8) -> (u8, u8, u8) {
    let y = y as f32;
    let cb = cb as f32 - 128.0;
    let cr = cr as f32 - 128.0;

    let r = y                + 1.40200 * cr;
    let g = y - 0.34414 * cb - 0.71414 * cr;
    let b = y + 1.77200 * cb;

    (clamp((r + 0.5) as i32, 0, 255) as u8,
     clamp((g + 0.5) as i32, 0, 255) as u8,
     clamp((b + 0.5) as i32, 0, 255) as u8)
}

fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min { return min; }
    if value > max { return max; }
    value
}

use byteorder::ReadBytesExt;
use error::{Error, Result};
use marker::Marker;
use std::io::Read;
use std::iter::repeat;

const LUT_BITS: u8 = 8;

#[derive(Debug)]
pub struct HuffmanDecoder {
    bits: u64,
    num_bits: u8,
    marker: Option<Marker>,
}

impl HuffmanDecoder {
    pub fn new() -> HuffmanDecoder {
        HuffmanDecoder {
            bits: 0,
            num_bits: 0,
            marker: None,
        }
    }

    // Section F.2.2.3
    // Figure F.16
    pub fn decode<R: Read>(&mut self, reader: &mut R, table: &HuffmanTable) -> Result<u8> {
        if self.num_bits < 16 {
            try!(self.read_bits(reader));
        }

        let (value, size) = table.lut[self.peek_bits(LUT_BITS) as usize];

        if size > 0 {
            self.consume_bits(size);
            Ok(value)
        }
        else {
            let bits = self.peek_bits(16);

            for i in LUT_BITS .. 16 {
                let code = (bits >> (15 - i)) as i32;

                if code <= table.maxcode[i as usize] {
                    self.consume_bits(i + 1);

                    let index = (code + table.delta[i as usize]) as usize;
                    return Ok(table.values[index]);
                }
            }

            Err(Error::Format("failed to decode huffman code".to_owned()))
        }
    }

    pub fn decode_fast_ac<R: Read>(&mut self, reader: &mut R, table: &HuffmanTable) -> Result<Option<(i16, u8)>> {
        if let Some(ref ac_lut) = table.ac_lut {
            if self.num_bits < LUT_BITS {
                try!(self.read_bits(reader));
            }

            let (value, run_size) = ac_lut[self.peek_bits(LUT_BITS) as usize];

            if run_size != 0 {
                let run = run_size >> 4;
                let size = run_size & 0x0f;

                self.consume_bits(size);
                return Ok(Some((value, run)));
            }
        }

        Ok(None)
    }

    #[inline]
    pub fn get_bits<R: Read>(&mut self, reader: &mut R, count: u8) -> Result<u16> {
        if self.num_bits < count {
            try!(self.read_bits(reader));
        }

        let bits = self.peek_bits(count);
        self.consume_bits(count);

        Ok(bits)
    }

    #[inline]
    pub fn receive_extend<R: Read>(&mut self, reader: &mut R, count: u8) -> Result<i16> {
        let value = try!(self.get_bits(reader, count));
        Ok(extend(value, count))
    }

    pub fn reset(&mut self) {
        self.bits = 0;
        self.num_bits = 0;
    }

    pub fn take_marker(&mut self) -> Option<Marker> {
        self.marker.take()
    }

    #[inline]
    fn peek_bits(&mut self, count: u8) -> u16 {
        debug_assert!(count <= 16);
        debug_assert!(self.num_bits >= count);

        ((self.bits >> (64 - count)) & ((1 << count) - 1)) as u16
    }

    #[inline]
    fn consume_bits(&mut self, count: u8) {
        debug_assert!(self.num_bits >= count);

        self.bits <<= count as usize;
        self.num_bits -= count;
    }

    fn read_bits<R: Read>(&mut self, reader: &mut R) -> Result<()> {
        while self.num_bits <= 56 {
            // Fill with zero bits if we have reached the end.
            let byte = match self.marker {
                Some(_) => 0,
                None => try!(reader.read_u8()),
            };

            if byte == 0xFF {
                let mut next_byte = try!(reader.read_u8());

                // Check for byte stuffing.
                if next_byte != 0x00 {
                    // We seem to have reached the end of entropy-coded data and encountered a
                    // marker. Since we can't put data back into the reader, we have to continue
                    // reading to identify the marker so we can pass it on.

                    // Section B.1.1.2
                    // "Any marker may optionally be preceded by any number of fill bytes, which are bytes assigned code X’FF’."
                    while next_byte == 0xFF {
                        next_byte = try!(reader.read_u8());
                    }

                    match next_byte {
                        0x00 => return Err(Error::Format("FF 00 found where marker was expected".to_owned())),
                        _    => self.marker = Some(Marker::from_u8(next_byte).unwrap()),
                    }

                    continue;
                }
            }

            self.bits |= (byte as u64) << (56 - self.num_bits);
            self.num_bits += 8;
        }

        Ok(())
    }
}

// Section F.2.2.1
// Figure F.12
fn extend(value: u16, count: u8) -> i16 {
    let vt = 1 << (count as u16 - 1);

    if value < vt {
        value as i16 + (-1 << count as i16) + 1
    } else {
        value as i16
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HuffmanTableClass {
    DC,
    AC,
}

pub struct HuffmanTable {
    values: Vec<u8>,
    delta: [i32; 16],
    maxcode: [i32; 16],

    lut: [(u8, u8); 1 << LUT_BITS],
    ac_lut: Option<[(i16, u8); 1 << LUT_BITS]>,
}

impl HuffmanTable {
    pub fn new(bits: &[u8; 16], values: &[u8], class: HuffmanTableClass) -> Result<HuffmanTable> {
        let (huffcode, huffsize) = try!(derive_huffman_codes(bits));

        // Section F.2.2.3
        // Figure F.15
        // delta[i] is set to VALPTR(I) - MINCODE(I)
        let mut delta = [0i32; 16];
        let mut maxcode = [-1i32; 16];
        let mut j = 0;

        for i in 0 .. 16 {
            if bits[i] != 0 {
                delta[i] = j as i32 - huffcode[j] as i32;
                j += bits[i] as usize;
                maxcode[i] = huffcode[j - 1] as i32;
            }
        }

        // Build a lookup table for faster decoding.
        let mut lut = [(0u8, 0u8); 1 << LUT_BITS];

        for (i, &size) in huffsize.iter().enumerate().filter(|&(_, &size)| size <= LUT_BITS) {
            let bits_remaining = LUT_BITS - size;
            let start = (huffcode[i] << bits_remaining) as usize;

            for j in 0 .. 1 << bits_remaining {
                lut[start + j] = (values[i], size);
            }
        }

        // Build a lookup table for small AC coefficients which both decodes the value and does the
        // equivalent of receive_extend.
        let ac_lut = match class {
            HuffmanTableClass::DC => None,
            HuffmanTableClass::AC => {
                let mut table = [(0i16, 0u8); 1 << LUT_BITS];

                for (i, &(value, size)) in lut.iter().enumerate() {
                    let run_length = value >> 4;
                    let magnitude_category = value & 0x0f;

                    if magnitude_category > 0 && size + magnitude_category <= LUT_BITS {
                        let unextended_ac_value = (((i << size) & ((1 << LUT_BITS) - 1)) >> (LUT_BITS - magnitude_category)) as u16;
                        let ac_value = extend(unextended_ac_value, magnitude_category);

                        table[i] = (ac_value, (run_length << 4) | (size + magnitude_category));
                    }
                }

                Some(table)
            },
        };

        Ok(HuffmanTable {
            values: values.to_vec(),
            delta: delta,
            maxcode: maxcode,
            lut: lut,
            ac_lut: ac_lut,
        })
    }
}

// Section C.2
fn derive_huffman_codes(bits: &[u8; 16]) -> Result<(Vec<u16>, Vec<u8>)> {
    // Figure C.1
    let huffsize = bits.iter()
                       .enumerate()
                       .fold(Vec::new(), |mut acc, (i, &value)| {
                           let mut repeated_size: Vec<u8> = repeat((i + 1) as u8).take(value as usize).collect();
                           acc.append(&mut repeated_size);
                           acc
                       });

    // Figure C.2
    let mut huffcode = vec![0u16; huffsize.len()];
    let mut code_size = huffsize[0];
    let mut code = 0u16;

    for (i, &size) in huffsize.iter().enumerate() {
        while code_size < size {
            code <<= 1;
            code_size += 1;
        }

        if code as u32 >= (1u32 << size) {
            return Err(Error::Format("bad huffman code length".to_owned()));
        }

        huffcode[i] = code;
        code += 1;
    }

    Ok((huffcode, huffsize))
}

use error::{Error, Result, UnsupportedFeature};
use parser::Component;

type ResampleFunc = fn(&[u8], usize, usize, usize, usize, usize, &mut [u8]);

pub struct Resampler {
    resample_funcs: Vec<ResampleFunc>,
    sizes: Vec<(usize, usize)>,
    row_strides: Vec<usize>,
}

impl Resampler {
    pub fn new(components: &[Component]) -> Result<Resampler> {
        let h_max = components.iter().map(|c| c.horizontal_sampling_factor).max().unwrap();
        let v_max = components.iter().map(|c| c.vertical_sampling_factor).max().unwrap();
        let mut resample_funcs = vec![];

        for component in components {
            resample_funcs.push(try!(choose_resampling_func(component, h_max, v_max)));
        }

        Ok(Resampler {
            resample_funcs: resample_funcs,
            sizes: components.iter().map(|comp| (comp.size.width as usize, comp.size.height as usize)).collect(),
            row_strides: components.iter().map(|comp| comp.block_size.width as usize * 8).collect(),
        })
    }

    pub fn resample_and_interleave_row(&self, component_data: &[Vec<u8>], row: usize, output_width: usize, output: &mut [u8]) {
        let component_count = component_data.len();
        let mut line_buffer = vec![0u8; output_width + 1];

        for i in 0 .. component_count {
            self.resample_funcs[i](&component_data[i],
                                   self.sizes[i].0,
                                   self.sizes[i].1,
                                   self.row_strides[i],
                                   row,
                                   output_width,
                                   &mut line_buffer);

            for x in 0 .. output_width {
                output[x * component_count + i] = line_buffer[x];
            }
        }
    }
}

fn choose_resampling_func(component: &Component, h_max: u8, v_max: u8) -> Result<ResampleFunc> {
    let h1 = component.horizontal_sampling_factor == h_max;
    let v1 = component.vertical_sampling_factor == v_max;
    let h2 = component.horizontal_sampling_factor * 2 == h_max;
    let v2 = component.vertical_sampling_factor * 2 == v_max;

    if h1 && v1 {
        Ok(resample_row_1)
    }
    else if h2 && v1 {
        Ok(resample_row_h_2_bilinear)
    }
    else if h1 && v2 {
        Ok(resample_row_v_2_bilinear)
    }
    else if h2 && v2 {
        Ok(resample_row_hv_2_bilinear)
    }
    else {
        Err(Error::Unsupported(UnsupportedFeature::SubsamplingRatio))
    }
}

fn resample_row_1(input: &[u8],
                  _input_width: usize,
                  _input_height: usize,
                  row_stride: usize,
                  row: usize,
                  output_width: usize,
                  output: &mut [u8]) {
    let input = &input[row * row_stride ..];

    for i in 0 .. output_width {
        output[i] = input[i];
    }
}

fn resample_row_h_2_bilinear(input: &[u8],
                             input_width: usize,
                             _input_height: usize,
                             row_stride: usize,
                             row: usize,
                             _output_width: usize,
                             output: &mut [u8]) {
    let input = &input[row * row_stride ..];

    if input_width == 1 {
        output[0] = input[0];
        output[1] = input[0];
        return;
    }

    output[0] = input[0];
    output[1] = ((input[0] as u32 * 3 + input[1] as u32 + 2) >> 2) as u8;

    for i in 1 .. input_width - 1 {
        let sample = 3 * input[i] as u32 + 2;
        output[i * 2]     = ((sample + input[i - 1] as u32) >> 2) as u8;
        output[i * 2 + 1] = ((sample + input[i + 1] as u32) >> 2) as u8;
    }

    output[(input_width - 1) * 2] = ((input[input_width - 1] as u32 * 3 + input[input_width - 2] as u32 + 2) >> 2) as u8;
    output[(input_width - 1) * 2 + 1] = input[input_width - 1];
}

fn resample_row_v_2_bilinear(input: &[u8],
                             _input_width: usize,
                             input_height: usize,
                             row_stride: usize,
                             row: usize,
                             output_width: usize,
                             output: &mut [u8]) {
    let row_near = row as f32 / 2.0;
    // If row_near's fractional is 0.0 we want row_far to be the previous row and if it's 0.5 we
    // want it to be the next row.
    let row_far = (row_near + row_near.fract() * 3.0 - 0.25).min((input_height - 1) as f32);

    let input_near = &input[row_near as usize * row_stride ..];
    let input_far = &input[row_far as usize * row_stride ..];

    for i in 0 .. output_width {
        output[i] = ((3 * input_near[i] as u32 + input_far[i] as u32 + 2) >> 2) as u8;
    }
}

fn resample_row_hv_2_bilinear(input: &[u8],
                              input_width: usize,
                              input_height: usize,
                              row_stride: usize,
                              row: usize,
                              _output_width: usize,
                              output: &mut [u8]) {
    let row_near = row as f32 / 2.0;
    // If row_near's fractional is 0.0 we want row_far to be the previous row and if it's 0.5 we
    // want it to be the next row.
    let row_far = (row_near + row_near.fract() * 3.0 - 0.25).min((input_height - 1) as f32);

    let input_near = &input[row_near as usize * row_stride ..];
    let input_far = &input[row_far as usize * row_stride ..];

    if input_width == 1 {
        let value = ((3 * input_near[0] as u32 + input_far[0] as u32 + 2) >> 2) as u8;
        output[0] = value;
        output[1] = value;
        return;
    }

    let mut t1 = 3 * input_near[0] as u32 + input_far[0] as u32;
    output[0] = ((t1 + 2) >> 2) as u8;

    for i in 1 .. input_width {
        let t0 = t1;
        t1 = 3 * input_near[i] as u32 + input_far[i] as u32;

        output[i * 2 - 1] = ((3 * t0 + t1 + 8) >> 4) as u8;
        output[i * 2]     = ((3 * t1 + t0 + 8) >> 4) as u8;
    }

    output[input_width * 2 - 1] = ((t1 + 2) >> 2) as u8;
}

/// Wrapper around the afl-fuzz binary

use libc;
use std::env;
use std::ffi::CString;
use std::path::PathBuf;

extern "C" {
    // `main` function with afl-fuzz.c
    fn afl_fuzz_main(argc: libc::c_int,
                     argv: *const *const libc::c_char) -> libc::c_int;
}

// TODO: use builder pattern
pub struct AflFuzzConfig {
    pub in_dir: PathBuf,
    pub out_dir: PathBuf,
    pub target: PathBuf,
}

impl AflFuzzConfig {
    fn into_args(self) -> Vec<CString> {
        let mut args = vec![];

        // -i
        args.push("-i");
        args.push(self.in_dir.to_str().unwrap());

        // -o
        args.push("-o");
        args.push(self.out_dir.to_str().unwrap());

        // target
        args.push(self.target.to_str().unwrap());

        // Convert args from `str`s to to `CString`s
        args.into_iter()
            .map(|s| CString::new(s).unwrap())
            .collect()
    }
}

pub fn afl_fuzz_env() -> Result<(), libc::c_int> {
    let args = env::args();

    // don't include anything before "afl-fuzz"
    let args = args.skip_while(|a| a != "afl-fuzz");

    // convert the CStrings to raw pointers
    let c_args = args.map(|arg| CString::new(arg).unwrap())
                     .map(|arg| arg.as_ptr())
                     .collect::<Vec<_>>();

    let ret = unsafe {
        afl_fuzz_main(c_args.len() as libc::c_int, c_args.as_ptr())
    };

    match ret {
        0 => Ok(()),
        n => Err(n),
    }
}


pub fn afl_fuzz(config: AflFuzzConfig) -> Result<(), ()> {
    let mut args = vec![];

    // Fake the first argument
    let arg0 = CString::new("afl-fuzz").unwrap();
    args.push(arg0);

    args.extend(config.into_args());

    // convert the CStrings to raw pointers
    let c_args = args.iter()
                     .map(|arg| arg.as_ptr())
                     .collect::<Vec<_>>();

    let ret = unsafe {
        afl_fuzz_main(c_args.len() as i32, c_args.as_ptr())
    };

    match ret {
        0 => Ok(()),
        _ => Err(()),
    }
}

/// FIXME: figure out a way to make it not run indefinitely
#[test]
fn testsomething() {
    let config = AflFuzzConfig {
        in_dir: ".".into(),
        out_dir: "testout".into(),
        target: "../target/debug/examples/hello".into(),
    };
    afl_fuzz(config);
}

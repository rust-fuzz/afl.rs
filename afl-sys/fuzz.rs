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

fn call_afl_fuzz_main(args: Vec<CString>) -> Result<(), libc::c_int> {
    assert!(!args.is_empty());

    // `main` functions in C expect an array of pointers to the arguments
    let args_ptrs = args.iter()
                        .map(|arg| arg.as_ptr())
                        .collect::<Vec<_>>();

    let ret = unsafe {
        afl_fuzz_main(args_ptrs.len() as libc::c_int,
                      args_ptrs.as_ptr())
    };

    match ret {
        0 => Ok(()),
        n => Err(n),
    }
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

        // Convert args from `str`s to `CString`s
        args.into_iter()
            .map(|s| CString::new(s).unwrap())
            .collect()
    }
}

pub fn afl_fuzz_env() -> Result<(), libc::c_int> {
    let args = env::args();

    // don't include anything before 'afl-fuzz' or 'cargo-afl-fuzz'
    let mut args = args.skip_while(|a| !a.ends_with("afl-fuzz"))
                       .map(|arg| CString::new(arg).unwrap())
                       .collect::<Vec<_>>();
    assert!(!args.is_empty(), "Error generating afl-fuzz arguments");

    args[0] = CString::new("afl-fuzz").unwrap();

    call_afl_fuzz_main(args)
}


pub fn afl_fuzz(config: AflFuzzConfig) -> Result<(), libc::c_int> {
    let mut args = vec![];

    // Fake the first argument
    let arg0 = CString::new("afl-fuzz").unwrap();
    args.push(arg0);

    args.extend(config.into_args());

    call_afl_fuzz_main(args)
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

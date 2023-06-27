// Copyright 2015 Keegan McAllister.
// Copyright 2016 Corey Farwell.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// See `LICENSE` in this repository.

use std::io::{self, Read};
use std::panic;

mod common;

// those functions are provided by the afl-llvm-rt static library
extern "C" {
    fn __afl_persistent_loop(counter: usize) -> isize;
    fn __afl_manual_init();

    static __afl_fuzz_len: *const u32;
    static __afl_fuzz_ptr: *const u8;
}

#[doc(hidden)]
#[no_mangle]
pub static mut __afl_sharedmem_fuzzing: i32 = 1;

/// Fuzz a closure by passing it a `&[u8]`
///
/// This slice contains a "random" quantity of "random" data.
///
/// ```rust,no_run
/// # extern crate afl;
/// # use afl::fuzz;
/// # fn main() {
/// fuzz(true, |data|{
///     if data.len() != 6 {return}
///     if data[0] != b'q' {return}
///     if data[1] != b'w' {return}
///     if data[2] != b'e' {return}
///     if data[3] != b'r' {return}
///     if data[4] != b't' {return}
///     if data[5] != b'y' {return}
///     panic!("BOOM")
/// });
/// # }
/// ```
pub fn fuzz<F>(hook: bool, mut closure: F)
where
    F: FnMut(&[u8]) + std::panic::RefUnwindSafe,
{
    // this marker strings needs to be in the produced executable for
    // afl-fuzz to detect `persistent mode` and `defered mode`
    static PERSIST_MARKER: &str = "##SIG_AFL_PERSISTENT##\0";
    static DEFERED_MARKER: &str = "##SIG_AFL_DEFER_FORKSRV##\0";

    // we now need a fake instruction to prevent the compiler from optimizing out
    // those marker strings
    unsafe { std::ptr::read_volatile(&PERSIST_MARKER) }; // hack used in https://github.com/bluss/bencher for black_box()
    unsafe { std::ptr::read_volatile(&DEFERED_MARKER) };
    // unsafe { asm!("" : : "r"(&PERSIST_MARKER)) }; // hack used in nightly's back_box(), requires feature asm
    // unsafe { asm!("" : : "r"(&DEFERED_MARKER)) };

    if hook {
        // sets panic hook to abort
        std::panic::set_hook(Box::new(|_| {
            std::process::abort();
        }));
    }

    let mut input = vec![];

    // initialize forkserver there
    unsafe { __afl_manual_init() };

    while unsafe { __afl_persistent_loop(1000) } != 0 {
        // get the testcase from the fuzzer
        let input_ref = if unsafe { __afl_fuzz_ptr.is_null() } {
            // in-memory testcase delivery is not enabled
            // get buffer from AFL++ through stdin
            let result = io::stdin().read_to_end(&mut input);
            if result.is_err() {
                return;
            }
            &input
        } else {
            unsafe {
                // get the testcase from the shared memory
                let input_len = *__afl_fuzz_len as usize;
                std::slice::from_raw_parts(__afl_fuzz_ptr, input_len)
            }
        };

        // We still catch unwinding panics just in case the fuzzed code modifies
        // the panic hook.
        // If so, the fuzzer will be unable to tell different bugs apart and you will
        // only be able to find one bug at a time before fixing it to then find a new one.
        let did_panic = std::panic::catch_unwind(panic::AssertUnwindSafe(|| {
            closure(input_ref);
        }))
        .is_err();

        if did_panic {
            // hopefully the custom panic hook will be called before and abort the
            // process before the stack frames are unwinded.
            std::process::abort();
        }
        // The version of lazy_static.rs at https://github.com/rust-fuzz/resettable-lazy-static.rs
        // provides the `reset` function.
        #[cfg(feature = "reset_lazy_static")]
        unsafe {
            lazy_static::lazy::reset();
        }
        input.clear();
    }
}

/// Fuzz a closure-like block of code by passing it an object of arbitrary type.
///
/// You can choose the type of the argument using the syntax as in the example below.
/// Please check out the `arbitrary` crate to see which types are available.
///
/// For performance reasons, it is recommended that you use the native type `&[u8]` when possible.
///
/// ```rust,no_run
/// # #[macro_use] extern crate afl;
/// # fn main() {
/// fuzz!(|data: &[u8]| {
///     if data.len() != 6 {return}
///     if data[0] != b'q' {return}
///     if data[1] != b'w' {return}
///     if data[2] != b'e' {return}
///     if data[3] != b'r' {return}
///     if data[4] != b't' {return}
///     if data[5] != b'y' {return}
///     panic!("BOOM")
/// });
/// # }
/// ```
#[macro_export]
macro_rules! fuzz {
    ( $($x:tt)* ) => { $crate::__fuzz!(true, $($x)*) }
}

/// Like `fuzz!` above, but panics that are caught inside the fuzzed code are not turned into
/// crashes.
#[macro_export]
macro_rules! fuzz_nohook {
    ( $($x:tt)* ) => { $crate::__fuzz!(false, $($x)*) }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fuzz {
    ($hook:expr, |$buf:ident| $body:block) => {
        $crate::fuzz($hook, |$buf| $body);
    };
    ($hook:expr, |$buf:ident: &[u8]| $body:block) => {
        $crate::fuzz($hook, |$buf| $body);
    };
    ($hook:expr, |$buf:ident: $dty: ty| $body:block) => {
        $crate::fuzz($hook, |$buf| {
            let $buf: $dty = {
                let mut data = ::arbitrary::Unstructured::new($buf);
                if let Ok(d) = ::arbitrary::Arbitrary::arbitrary(&mut data).map_err(|_| "") {
                    d
                } else {
                    return;
                }
            };

            $body
        });
    };
}

#[cfg(test)]
mod test {
    use std::{path, process, thread, time};

    fn target_dir_path() -> &'static path::Path {
        if path::Path::new("../target/debug/cargo-afl").exists() {
            path::Path::new("../target/debug/")
        } else if path::Path::new("target/debug/cargo-afl").exists() {
            path::Path::new("target/debug/")
        } else {
            panic!("Could not find cargo-afl binary");
        }
    }

    fn cargo_afl_path() -> path::PathBuf {
        target_dir_path().join("cargo-afl")
    }

    fn examples_hello_path() -> path::PathBuf {
        target_dir_path().join("examples").join("hello")
    }

    fn input_path() -> path::PathBuf {
        path::Path::new(env!("CARGO_MANIFEST_DIR")).join("input")
    }

    #[test]
    fn integration() {
        let temp_dir = tempfile::TempDir::new().expect("Could not create temporary directory");
        let temp_dir_path = temp_dir.path();
        let mut child = process::Command::new(cargo_afl_path())
            .arg("afl")
            .arg("fuzz")
            .stdout(process::Stdio::inherit())
            .stderr(process::Stdio::inherit())
            .arg("-i")
            .arg(input_path())
            .arg("-o")
            .arg(temp_dir_path)
            .arg(examples_hello_path())
            .env("AFL_NO_UI", "1")
            .spawn()
            .expect("Could not run cargo afl fuzz");
        thread::sleep(time::Duration::from_secs(10));
        for _ in 0..5 {
            thread::sleep(time::Duration::from_secs(1));
            child.kill().unwrap_or_default();
        }
        assert!(temp_dir_path.join("default").join("fuzzer_stats").is_file());
    }
}

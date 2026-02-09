// Copyright 2015 Keegan McAllister.
// Copyright 2016 Corey Farwell.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// See `LICENSE` in this repository.

use std::env;
use std::io::{self, Read};
use std::os::raw::c_char;
use std::panic;

// those functions are provided by the afl-compiler-rt static library
unsafe extern "C" {
    fn __afl_persistent_loop(counter: usize) -> isize;
    fn __afl_manual_init();

    static __afl_fuzz_len: *const u32;
    static __afl_fuzz_ptr: *const u8;
}

// AFL++ IJON functions in afl-compiler-rt
unsafe extern "C" {
    pub fn ijon_max(addr: u32, val: u64);
    pub fn ijon_min(addr: u32, val: u64);
    pub fn ijon_set(addr: u32, val: u32);
    pub fn ijon_inc(addr: u32, val: u32);
    pub fn ijon_xor_state(val: u32);
    pub fn ijon_reset_state();
    pub fn ijon_simple_hash(x: u64) -> u64;
    pub fn ijon_hashint(old: u32, val: u32) -> u32;
    pub fn ijon_hashstr(old: u32, val: *const c_char) -> u32;
    pub fn ijon_hashmen(old: u32, val: *const u8, len: usize) -> u32;
    pub fn ijon_hashstack_backtrace() -> u32;
    pub fn ijon_hashstack() -> u32;
    pub fn ijon_strdist(a: *const u8, b: *const u8) -> u32;
    pub fn ijon_memdist(a: *const u8, b: *const u8, len: usize) -> u32;
    pub fn ijon_max_variadic(addr: u32, ...);
    pub fn ijon_min_variadic(addr: u32, ...);
}

#[macro_export]
macro_rules! ijon_inc {
    ($x:expr) => {{
        unsafe {
            static mut loc: u32 = 0;
            if loc == 0 {
                let cfile = std::ffi::CString::new(file!()).unwrap();
                loc = afl::ijon_hashstr(line!(), cfile.as_ptr());
            }
            afl::ijon_inc(loc, $x)
        };
    }};
}

#[macro_export]
macro_rules! ijon_max {
    ($($x:expr),+ $(,)?) => {{
        unsafe {
            static mut loc: u32 = 0;
            if loc == 0 {
                let cfile = std::ffi::CString::new(file!()).unwrap();
                loc = afl::ijon_hashstr(line!(), cfile.as_ptr());
            }
            afl::ijon_max_variadic(_IJON_LOC_CACHE, $($x),+, 0u64)
        };
    }};
}

#[macro_export]
macro_rules! ijon_min {
    ($($x:expr),+ $(,)?) => {{
        unsafe {
            static mut loc: u32 = 0;
            if loc == 0 {
                let cfile = std::ffi::CString::new(file!()).unwrap();
                loc = afl::ijon_hashstr(line!(), cfile.as_ptr());
            }
            afl::ijon_min_variadic(loc, $($x),+, 0u64)
        };
    }};
}

#[macro_export]
macro_rules! ijon_set {
    ($x:expr) => {{
        unsafe {
            static mut loc: u32 = 0;
            if loc == 0 {
                let cfile = std::ffi::CString::new(file!()).unwrap();
                loc = afl::ijon_hashstr(line!(), cfile.as_ptr());
            }
            afl::ijon_set(loc, $x)
        };
    }};
}

#[macro_export]
macro_rules! ijon_state {
    ($n:expr) => {
        unsafe { afl::ijon_xor_state($n) }
    };
}

#[macro_export]
macro_rules! ijon_ctx {
    ($x:expr) => {{
        let cfile = std::ffi::CString::new(file!()).unwrap();
        let hash = unsafe { afl::ijon_hashstr(line!(), cfile.as_ptr()) };
        unsafe { afl::ijon_xor_state(hash) };
        let temp = $x;
        unsafe { afl::ijon_xor_state(hash) };
        temp
    }};
}

#[macro_export]
macro_rules! ijon_max_at {
    ($addr:expr, $x:expr) => {
        unsafe { afl::ijon_max($addr, $x) }
    };
}

#[macro_export]
macro_rules! ijon_min_at {
    ($addr:expr, $x:expr) => {
        unsafe { afl::ijon_min($addr, $x) }
    };
}

#[macro_export]
macro_rules! _ijon_abs_dist {
    ($x:expr, $y:expr) => {
        if $x < $y { $y - $x } else { $x - $y }
    };
}

#[macro_export]
macro_rules! ijon_bits {
    ($x:expr) => {
        unsafe {
            afl::ijon_set(afl::ijon_hashint(
                afl::ijon_hashstack(),
                if $x == 0 {
                    0
                } else {
                    $x.leading_zeros() as u32
                },
            ))
        }
    };
}

#[macro_export]
macro_rules! ijon_strdist {
    ($x:expr, $y:expr) => {
        unsafe {
            afl::ijon_set(afl::ijon_hashint(
                afl::ijon_hashstack(),
                afl::ijon_strdist($x, $y),
            ))
        }
    };
}

#[macro_export]
macro_rules! ijon_dist {
    ($x:expr, $y:expr) => {
        unsafe {
            afl::ijon_set(afl::ijon_hashint(
                afl::ijon_hashstack(),
                $crate::_ijon_abs_dist!($x, $y),
            ))
        }
    };
}

#[macro_export]
macro_rules! ijon_cmp {
    ($x:expr, $y:expr) => {
        unsafe {
            afl::ijon_inc(afl::ijon_hashint(
                afl::ijon_hashstack(),
                ($x ^ $y).count_ones(),
            ))
        }
    };
}

#[macro_export]
macro_rules! ijon_stack_max {
    ($x:expr) => {{
        unsafe {
            static mut loc: u32 = 0;
            if loc == 0 {
                let cfile = std::ffi::CString::new(file!()).unwrap();
                loc = afl::ijon_hashstr(line!(), cfile.as_ptr());
            }
            afl::ijon_max(afl::ijon_hashint(loc, afl::ijon_hashstack()), $x)
        };
    }};
}

#[macro_export]
macro_rules! ijon_stack_min {
    ($x:expr) => {{
        unsafe {
            static mut loc: u32 = 0;
            if loc == 0 {
                let cfile = std::ffi::CString::new(file!()).unwrap();
                loc = afl::ijon_hashstr(line!(), cfile.as_ptr());
            }
            afl::ijon_min(afl::ijon_hashint(loc, afl::ijon_hashstack()), $x)
        };
    }};
}

// end if AFL++ IJON functions

#[allow(non_upper_case_globals)]
#[doc(hidden)]
#[unsafe(no_mangle)]
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
pub fn fuzz<F>(hook: bool, closure: F)
where
    F: FnMut(&[u8]) + std::panic::RefUnwindSafe,
{
    fuzz_with_reset(hook, closure, || {});
}

/// Like [`fuzz()`], but calls a `reset` closure after each successful iteration.
///
/// This is useful when the fuzz target uses static state (e.g., `OnceLock`, `lazy_static`)
/// that must be cleared between iterations in AFL++ persistent mode. Without resetting,
/// code paths that run only on the first iteration cause AFL's stability metric to drop.
///
/// ```rust,no_run
/// # extern crate afl;
/// # use afl::fuzz_with_reset;
/// # use std::sync::Mutex;
/// # static CACHE: Mutex<Option<Vec<u8>>> = Mutex::new(None);
/// # fn main() {
/// fuzz_with_reset(true, |data| {
///     let mut cache = CACHE.lock().unwrap();
///     if cache.is_none() {
///         *cache = Some(data.to_vec());
///     }
/// }, || {
///     *CACHE.lock().unwrap() = None;
/// });
/// # }
/// ```
pub fn fuzz_with_reset<F, R>(hook: bool, mut closure: F, mut reset: R)
where
    F: FnMut(&[u8]) + std::panic::RefUnwindSafe,
    R: FnMut(),
{
    // this marker strings needs to be in the produced executable for
    // afl-fuzz to detect `persistent mode` and `defered mode`
    static PERSIST_MARKER: &str = "##SIG_AFL_PERSISTENT##\0";
    static DEFERED_MARKER: &str = "##SIG_AFL_DEFER_FORKSRV##\0";

    // we now need a fake instruction to prevent the compiler from optimizing out
    // those marker strings
    unsafe { std::ptr::read_volatile(&raw const PERSIST_MARKER) }; // hack used in https://github.com/bluss/bencher for black_box()
    unsafe { std::ptr::read_volatile(&raw const DEFERED_MARKER) };
    // unsafe { asm!("" : : "r"(&PERSIST_MARKER)) }; // hack used in nightly's back_box(), requires feature asm
    // unsafe { asm!("" : : "r"(&DEFERED_MARKER)) };

    if hook {
        let prev_hook = std::panic::take_hook();
        // sets panic hook to abort
        std::panic::set_hook(Box::new(move |panic_info| {
            prev_hook(panic_info);
            std::process::abort();
        }));
    }

    let mut input = vec![];

    let loop_count = if let Ok(value) = env::var("AFL_FUZZER_LOOPCOUNT") {
        value
            .parse()
            .expect("Failed to parse environment variable to a number")
    } else {
        usize::MAX
    };

    // initialize forkserver there
    unsafe { __afl_manual_init() };

    if unsafe { __afl_fuzz_ptr.is_null() } {
        // in-memory testcase delivery is not enabled
        // get buffer from AFL++ through stdin
        let result = io::stdin().read_to_end(&mut input);
        if result.is_err() {
            return;
        }
        let input_ref = &input;

        let did_panic = std::panic::catch_unwind(panic::AssertUnwindSafe(|| {
            closure(input_ref);
        }))
        .is_err();

        if did_panic {
            // hopefully the custom panic hook will be called before and abort the
            // process before the stack frames are unwinded.
            std::process::abort();
        }

        reset();
    } else {
        while unsafe { __afl_persistent_loop(loop_count) } != 0 {
            // get the testcase from the fuzzer
            let input_ref = unsafe {
                // get the testcase from the shared memory
                let input_len = *__afl_fuzz_len as usize;
                std::slice::from_raw_parts(__afl_fuzz_ptr, input_len)
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

            reset();
            input.clear();
        }
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
    ($hook:expr, |$buf:ident| $body:expr) => {
        $crate::fuzz($hook, |$buf| $body);
    };
    ($hook:expr, |$buf:ident: &[u8]| $body:expr) => {
        $crate::fuzz($hook, |$buf| $body);
    };
    ($hook:expr, |$buf:ident: $dty: ty| $body:expr) => {
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
    ($hook:expr, |$buf:ident| $body:expr, $reset:expr) => {
        $crate::fuzz_with_reset($hook, |$buf| $body, $reset);
    };
    ($hook:expr, |$buf:ident: &[u8]| $body:expr, $reset:expr) => {
        $crate::fuzz_with_reset($hook, |$buf| $body, $reset);
    };
    ($hook:expr, |$buf:ident: $dty: ty| $body:expr, $reset:expr) => {
        $crate::fuzz_with_reset(
            $hook,
            |$buf| {
                let $buf: $dty = {
                    let mut data = ::arbitrary::Unstructured::new($buf);
                    if let Ok(d) = ::arbitrary::Arbitrary::arbitrary(&mut data).map_err(|_| "") {
                        d
                    } else {
                        return;
                    }
                };

                $body
            },
            $reset,
        );
    };
}

/// Like [`fuzz!`], but accepts a second closure that resets state after each iteration.
///
/// This is useful when the fuzz target uses static state (e.g., `OnceLock`, `lazy_static`)
/// that must be cleared between iterations in AFL++ persistent mode.
///
/// ```rust,no_run
/// # #[macro_use] extern crate afl;
/// # use std::sync::Mutex;
/// # static CACHE: Mutex<Option<Vec<u8>>> = Mutex::new(None);
/// # fn main() {
/// fuzz_with_reset!(|data: &[u8]| {
///     let mut cache = CACHE.lock().unwrap();
///     if cache.is_none() {
///         *cache = Some(data.to_vec());
///     }
/// }, || {
///     *CACHE.lock().unwrap() = None;
/// });
/// # }
/// ```
#[macro_export]
macro_rules! fuzz_with_reset {
    ( $($x:tt)* ) => { $crate::__fuzz!(true, $($x)*) }
}

/// Like [`fuzz_with_reset!`], but panics that are caught inside the fuzzed code are not turned
/// into crashes.
#[macro_export]
macro_rules! fuzz_with_reset_nohook {
    ( $($x:tt)* ) => { $crate::__fuzz!(false, $($x)*) }
}

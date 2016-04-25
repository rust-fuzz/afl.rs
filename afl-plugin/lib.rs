// Copyright 2015 Keegan McAllister.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// See `LICENSE` in this repository.

#![feature(plugin_registrar, rustc_private)]

extern crate rustc_plugin;

use rustc_plugin::registry::Registry;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_llvm_pass("afl-coverage");
}

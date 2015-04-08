// Copyright 2015 Keegan McAllister.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// See `LICENSE` in this repository.

#[cfg(not(test))]
#[link(name="afl_cov_rt", kind="static")]
extern "C" { }

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![no_main]

use aptos_fuzzer::FuzzTarget;
use libfuzzer_sys::fuzz_target;

// contains FUZZ_TARGET
include!(concat!(env!("OUT_DIR"), "/fuzzer.rs"));

fuzz_target!(|data: &[u8]| {
    let fuzzer = FuzzTarget::by_name(FUZZ_TARGET).unwrap();
    fuzzer.fuzz(data);
});

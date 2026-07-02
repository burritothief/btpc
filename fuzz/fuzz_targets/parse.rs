#![no_main]

use btpc_core::bencode::parse;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = parse(data);
});

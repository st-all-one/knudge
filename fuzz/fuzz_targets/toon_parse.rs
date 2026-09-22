#![no_main]

//! Fuzz do parser TOON: nenhuma entrada UTF-8 pode causar panic (E13-T08).

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        let _parsed = knudge_core::toon::parse(text);
    }
});

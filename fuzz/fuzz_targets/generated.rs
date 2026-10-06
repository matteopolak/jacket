#![no_main]
libfuzzer_sys::fuzz_target!(|data: &[u8]| jai_fuzz_harness::generated(data));

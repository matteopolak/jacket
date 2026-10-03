#![no_main]
libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    jai_fuzz::lexer_utf8(data);
});

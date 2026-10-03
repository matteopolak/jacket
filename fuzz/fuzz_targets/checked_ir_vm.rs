#![no_main]
libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    jai_fuzz::checked_ir_vm(data);
});

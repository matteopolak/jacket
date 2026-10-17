// Independent ISA fixture. Optional execution is guarded by the C/Rust oracle's real CPUID.
#program_export "machine_tsc"
machine_tsc :: () -> u64 #c_call {
    hi: u64 = 0xffffffffffffffff;
    lo: u64 = 0xffffffffffffffff;
    #asm { rdtsc hi, lo; }
    return (hi << 32) | lo;
}

#program_export "machine_tscp"
machine_tscp :: (aux: *u32) -> u64 #c_call {
    hi: u64 = 0xffffffffffffffff;
    lo: u64 = 0xffffffffffffffff;
    processor: u64 = 0xffffffffffffffff;

    #asm { rdtscp hi, lo, processor; }
    aux.* = cast(u32) processor;

    return (hi << 32) | lo;
}

#program_export "machine_fences"
machine_fences :: () #c_call {
    #asm { mfence; lfence; sfence; pause; }
}

#program_export "machine_prefetch"
machine_prefetch :: (pointer: *u64) #c_call {
    #asm { prefetcht0 [pointer]; prefetcht1 [pointer]; prefetcht2 [pointer]; prefetchnta [pointer]; }
}

#program_export "machine_random16"
machine_random16 :: (output: *u64) -> u32 #c_call {
    value: u64 = 0x123456789abc0000;
    carry: u8;
    zero: u8;
    sign: u8;
    overflow: u8;
    parity: u8;

    #asm { rdrand.w value; setc carry; setz zero; sets sign; seto overflow; setp parity; }
    output.* = value;

    return (cast(u32) carry) | ((cast(u32) zero) << 1) | ((cast(u32) sign) << 2) | ((cast(u32) overflow) << 3) | ((cast(u32) parity) << 4);
}

#program_export "machine_random32"
machine_random32 :: (output: *u64) -> u32 #c_call {
    value: u64 = 0x123456789abc0000;
    carry: u8;
    zero: u8;
    sign: u8;
    overflow: u8;
    parity: u8;

    #asm { rdrand.d value; setc carry; setz zero; sets sign; seto overflow; setp parity; }
    output.* = value;

    return (cast(u32) carry) | ((cast(u32) zero) << 1) | ((cast(u32) sign) << 2) | ((cast(u32) overflow) << 3) | ((cast(u32) parity) << 4);
}

#program_export "machine_random64"
machine_random64 :: (output: *u64) -> u32 #c_call {
    value: u64;
    carry: u8;
    zero: u8;
    sign: u8;
    overflow: u8;
    parity: u8;

    #asm { rdrand.q value; setc carry; setz zero; sets sign; seto overflow; setp parity; }
    output.* = value;

    return (cast(u32) carry) | ((cast(u32) zero) << 1) | ((cast(u32) sign) << 2) | ((cast(u32) overflow) << 3) | ((cast(u32) parity) << 4);
}

#program_export "machine_seed64"
machine_seed64 :: (output: *u64) -> u32 #c_call {
    value: u64;
    carry: u8;
    zero: u8;
    sign: u8;
    overflow: u8;
    parity: u8;

    #asm { rdseed.q value; setc carry; setz zero; sets sign; seto overflow; setp parity; }
    output.* = value;

    return (cast(u32) carry) | ((cast(u32) zero) << 1) | ((cast(u32) sign) << 2) | ((cast(u32) overflow) << 3) | ((cast(u32) parity) << 4);
}

#program_export "machine_seed16"
machine_seed16 :: (output: *u64) -> u32 #c_call {
    value: u64 = 0x123456789abc0000;
    carry: u8;
    zero: u8;
    sign: u8;
    overflow: u8;
    parity: u8;

    #asm { rdseed.w value; setc carry; setz zero; sets sign; seto overflow; setp parity; }
    output.* = value;

    return (cast(u32) carry) | ((cast(u32) zero) << 1) | ((cast(u32) sign) << 2) | ((cast(u32) overflow) << 3) | ((cast(u32) parity) << 4);
}

#program_export "machine_seed32"
machine_seed32 :: (output: *u64) -> u32 #c_call {
    value: u64 = 0x123456789abc0000;
    carry: u8;
    zero: u8;
    sign: u8;
    overflow: u8;
    parity: u8;

    #asm { rdseed.d value; setc carry; setz zero; sets sign; seto overflow; setp parity; }
    output.* = value;

    return (cast(u32) carry) | ((cast(u32) zero) << 1) | ((cast(u32) sign) << 2) | ((cast(u32) overflow) << 3) | ((cast(u32) parity) << 4);
}

main :: () {}

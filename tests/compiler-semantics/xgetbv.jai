// Independent ISA queries. Call only after real CPUID XSAVE/OSXSAVE checks.
#program_export "xgetbv_read"
xgetbv_read :: (index: u64, high: *u64, low: *u64) -> u64 #c_call {
    hi: u64 = 0xffffffffffffffff;
    lo: u64 = 0xffffffffffffffff;

    #asm { xgetbv hi, lo, index; }
    high.* = hi;
    low.* = lo;

    return (hi << 32) | lo;
}

#program_export "xgetbv_read32"
xgetbv_read32 :: (index: u32) -> u64 #c_call {
    hi: u32;
    lo: u32;
    #asm { xgetbv hi, lo, index; }
    return (cast(u64) hi << 32) | cast(u64) lo;
}

#program_export "xgetbv_alias"
xgetbv_alias :: (index: u64) -> u64 #c_call {
    lo: u64 = 0xffffffffffffffff;
    #asm { xgetbv index, lo, index; }
    return (index << 32) | lo;
}

#program_export "xgetbv_discard"
xgetbv_discard :: (index: u32) #c_call {
    hi: u32;
    lo: u32;
    #asm { xgetbv hi, lo, index; }
}

#program_export "xgetbv_flags"
xgetbv_flags :: (index: u32, lhs: u64, rhs: u64) -> bool #c_call {
    hi: u64;
    lo: u64;
    carry_before: u8;
    carry_after: u8;
    zero_before: u8;
    zero_after: u8;
    sign_before: u8;
    sign_after: u8;
    overflow_before: u8;
    overflow_after: u8;
    parity_before: u8;
    parity_after: u8;

    #asm {
        cmp lhs, rhs;
        setc carry_before; setz zero_before; sets sign_before;
        seto overflow_before; setp parity_before;
        xgetbv hi, lo, index;
        setc carry_after; setz zero_after; sets sign_after;
        seto overflow_after; setp parity_after;
    }

    return carry_before == carry_after && zero_before == zero_after
        && sign_before == sign_after && overflow_before == overflow_after
        && parity_before == parity_after;
}

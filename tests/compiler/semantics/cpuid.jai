#program_export "cpu_query"
cpu_query :: (leaf: u32, subleaf: u32, output: *[4] u64) #c_call {
    a: u64 = cast(u64) leaf | 0xffff_ffff_0000_0000;
    b: u64 = 0xffff_ffff_ffff_ffff;
    c: u64 = cast(u64) subleaf | 0xffff_ffff_0000_0000;
    d: u64 = 0xffff_ffff_ffff_ffff;

    #asm { cpuid a, b, c, d; }
    output.* = .[a, b, c, d];
}

main :: () {}

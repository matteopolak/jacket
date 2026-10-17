// A wasm module without Wasi_Runtime (`-target wasm64-unknown-unknown`): the host supplies the
// imports and calls `#program_export` procedures directly, as a browser page would.

// No library: imported from "env".
host_report :: (label: *u8, value: s64) #foreign;

graphics :: #system_library "host_graphics";
host_draw :: (x: s32, y: s32) -> s32 #foreign graphics "draw";

bit_count :: (x: u64) -> u64 #intrinsic "llvm.ctpop.i64";

#program_export
triangle :: (n: s64) -> s64 #c_call {
    total: s64;
    for 1..n total += it;
    host_report("triangle".data, total);
    return total;
}

#program_export "count_bits"
count_bits_export :: (x: u64) -> s64 #c_call {
    return cast(s64) bit_count(x) + host_draw(1, 2);
}

main :: () {}

// test: only(.WINDOWS) timeout(30)
// Compiles HLSL with the string wrapper: the container header of the bytecode, a pixel shader
// target, and the diagnostics of a shader that does not compile.
#import "Basic";
#import "String";
C :: #import "d3d_compiler";

check :: (condition: bool, what: string) {
    if !condition {
        print("FAIL: %\n", what);
        exit(1);
    }
}

read_u32 :: (bytes: string, offset: s64) -> u32 {
    value: u32;
    for 0..3 {
        byte: u32 = bytes[offset + it];
        value |= byte << cast(u32) (8 * it);
    }
    return value;
}

main :: () {
    vertex := "float4 main(float4 position : POSITION) : SV_POSITION { return position; }";
    code, errors, hr := C.D3DCompile(vertex, "vertex.hlsl", .[], null, "main", "vs_5_0", 0, 0);
    check(hr >= 0, "vertex shader compiles");
    check(code.count > 32, "bytecode has a header");
    print("magic: %\n", slice(code, 0, 4));
    check(read_u32(code, 24) == cast(u32) code.count, "container size field equals the blob size");
    check(read_u32(code, 28) > 0, "container has chunks");
    print("version field: %\n", read_u32(code, 20));

    pixel := "float4 main() : SV_TARGET { return float4(1, 0, 0, 1); }";
    pixel_code, pixel_errors, pixel_hr := C.D3DCompile(pixel, "pixel.hlsl", .[], null, "main", "ps_5_0", 0, 0);
    check(pixel_hr >= 0, "pixel shader compiles");
    check(pixel_code.count > 32 && slice(pixel_code, 0, 4) == "DXBC", "pixel shader bytecode is a DXBC container");
    check(pixel_code.count != code.count || pixel_code != code, "pixel and vertex bytecode differ");

    broken := "float4 main() : SV_TARGET { return undefined_name; }";
    broken_code, diagnostics, broken_hr := C.D3DCompile(broken, "broken.hlsl", .[], null, "main", "ps_5_0", 0, 0);
    check(broken_hr < 0, "an undefined name fails to compile");
    check(broken_code.count == 0, "a failed compile has no bytecode");
    check(contains(diagnostics, "undefined_name"), "diagnostics name the undefined identifier");
    check(contains(diagnostics, "broken.hlsl"), "diagnostics name the source");
    print("diagnostics: ok\n");
}

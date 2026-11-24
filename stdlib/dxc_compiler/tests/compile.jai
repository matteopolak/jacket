// test: only(.WINDOWS) needs(dxc) timeout(60)
// Needs dxcompiler.dll and dxil.dll on the DLL search path (the Windows SDK's Redist\D3D folder).
// Compiles HLSL with the high-level wrapper and checks the DXIL container header, then
// reflects the result.
#import "Basic";
#import "String";
D :: #import "dxc_compiler";
W :: #import "Windows";

check :: (condition: bool, what: string) {
    if !condition {
        print("FAIL: %\n", what);
        exit(1);
    }
}

main :: () {
    version := D.dxc_get_version_string();

    check(begins_with(version, "v1."), "version string starts with v1.");

    args: D.Dxc_Shader_Compiler_Args;
    args.shader_name = "compute.hlsl";
    args.shader_source = "RWStructuredBuffer<uint> output : register(u0); [numthreads(8, 1, 1)] void main(uint3 id : SV_DispatchThreadID) { output[id.x] = id.x * 2; }";
    args.shader_model_version = "6_0";
    args.shader_type = .Compute;
    args.entry_point = "main";

    dxil := D.DXC_Compile(args);

    check(dxil.count > 32, "DXC returned a container");
    magic := string.{ count = 4, data = dxil.data };

    print("magic: %\n", magic);

    reflection, success := D.DXC_Reflect(dxil);

    check(success && reflection != null, "reflection of the compiled shader");
    W.IUnknown_Release(cast(*W.IUnknown) reflection);
    print("reflected: ok\n");
}

// Independent SDK route unit. Full public DXC source checks remain mandatory;
// this does not replace them or remove their D3D12 forward-overlay failure.
#load "../../../../internal/bindings/dxc-compile/module.jai";
TestBasic :: #import "Basic";

test_libc :: #library,system "c";
fail :: () #foreign test_libc "abort";

check :: (value: bool) {
    if !value fail();
}

main :: () {
    #assert size_of(DxcBuffer) == 24;
    #assert offset_of(DxcBuffer, "Ptr") == 0;
    #assert offset_of(DxcBuffer, "Size") == 8;
    #assert offset_of(DxcBuffer, "Encoding") == 16;
    #assert size_of(IDxcCompiler3) == 8 && size_of(IDxcUtils) == 8;
    #assert size_of(IDxcOperationResult) == 8 && size_of(IDxcResult) == 8;
    #assert size_of(IDxcBlob) == 8 && size_of(IDxcBlobEncoding) == 8;
    args: Dxc_Shader_Compiler_Args;
    args.shader_name = "binding-core-shader.hlsl";
    args.shader_source = "float4 main() : SV_Target { return float4(VALUE,0,0,1); }";
    args.shader_model_version = "6_0";
    args.shader_type = .Fragment;
    args.entry_point = "main";
    defines: [1] string = .["VALUE=1"];

    args.defines = defines;
    args.add_debug_info = true;
    args.warnings_as_error = true;
    object := compile(args);

    defer TestBasic.array_free(object);
    check(object.count > 4 && object[0] == #char "D" && object[1] == #char "X" &&
          object[2] == #char "B" && object[3] == #char "C");
    args.spirv_target_env = "vulkan1.3";
    args.spirv_generate_non_semanantic_debug_info = true;
    spirv := compile(args);

    defer TestBasic.array_free(spirv);
    check(spirv.count >= 4 && spirv[0] == 3 && spirv[1] == 2 && spirv[2] == 0x23 && spirv[3] == 7);
    args.shader_source = "this is not valid HLSL";
    failed := compile(args);

    defer TestBasic.array_free(failed);
    check(failed.count == 0);
}

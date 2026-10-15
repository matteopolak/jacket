// Actual DXC compilation followed by the public reflection wrapper. Requires
// Windows dxcompiler runtime. No compiler, blob, vtable or reflection is mocked.
D :: #import "dxc_compiler";
R :: #import "d3d12";
W :: #import "Windows";

libc :: #library,system "c";
abort :: () #foreign libc;

check :: (value: bool) {
    if !value abort();
}

main :: () {
    // Complete pinned SDK probe asserts these native buffer ABI requirements.
    #assert size_of(D.DxcBuffer) == 24;
    #assert offset_of(D.DxcBuffer, "Ptr") == 0;
    #assert offset_of(D.DxcBuffer, "Size") == 8;
    #assert offset_of(D.DxcBuffer, "Encoding") == 16;
    invalid: [] u8;
    absent, valid := D.DXC_Reflect(invalid);

    check(absent == null && !valid);
    compiler: *D.IDxcCompiler3;

    check(D.DxcCreateInstance(D.CLSID_DxcCompiler, D.IDxcCompiler3_UID, cast(**void) *compiler) >= 0 && compiler != null);
    defer W.IUnknown_Release(cast(*W.IUnknown) compiler);
    entry_flag: [3] s16 = .[45, 69, 0];
    entry: [5] s16 = .[109, 97, 105, 110, 0];
    target_flag: [3] s16 = .[45, 84, 0];
    target: [7] s16 = .[112, 115, 95, 54, 95, 48, 0];
    arguments: [4] *s16 = .[entry_flag.data, entry.data, target_flag.data, target.data];
    shader := "float4 main() : SV_Target { return float4(1,0,0,1); }";
    source: D.DxcBuffer;
    source.Ptr = shader.data;
    source.Size = cast(u64) shader.count;
    source.Encoding = D.DXC_CP_UTF8;
    operation: *D.IDxcResult;
    iid := D.IDxcResult_UID;

    check(D.IDxcCompiler3_Compile(compiler, *source, arguments.data, 4, null, *iid, cast(**void) *operation) >= 0 && operation != null);
    defer W.IUnknown_Release(cast(*W.IUnknown) operation);
    status: W.HRESULT;

    check(D.IDxcOperationResult_GetStatus(cast(*D.IDxcOperationResult) operation, *status) >= 0 && status >= 0);
    blob: *D.IDxcBlob;

    check(D.IDxcOperationResult_GetResult(cast(*D.IDxcOperationResult) operation, *blob) >= 0 && blob != null);
    defer W.IUnknown_Release(cast(*W.IUnknown) blob);
    size := D.IDxcBlob_GetBufferSize(blob);

    check(size > 0 && size <= 0x7fff_ffff_ffff_ffff);
    bytes: [] u8 = .{ data = cast(*u8) D.IDxcBlob_GetBufferPointer(blob), count = cast(s64) size };
    reflection, success := D.DXC_Reflect(bytes);

    #assert type_of(reflection) == *R.ID3D12ShaderReflection;
    check(success && reflection != null);
    defer W.IUnknown_Release(cast(*W.IUnknown) reflection);
    check(R.ID3D12ShaderReflection_GetNumInterfaceSlots(reflection) == 0);
    bad: [] u8 = .{ data = shader.data, count = shader.count };

    absent, valid = D.DXC_Reflect(bad);
    check(absent == null && !valid);
}

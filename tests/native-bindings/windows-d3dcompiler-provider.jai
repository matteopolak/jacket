// Real SDK shader compilation and provider identity, no adapter/GPU required.
#assert OS == .WINDOWS;

D :: #import "d3d_compiler";
W :: #import "Windows";

required :: #system_library,link_always "d3dcompiler";
kernel :: #system_library "kernel32";
module :: (name: *u8) -> *void #foreign kernel "GetModuleHandleA";
symbol :: (handle: *void, name: *u8) -> *void #foreign kernel "GetProcAddress";
exit :: (status: u32) #foreign kernel "ExitProcess";

// Which check failed is the exit status: 91 is the first one made.
checks_made: u32;

check :: (value: bool) {
    if !value exit(91 + checks_made);
    checks_made += 1;
}

release :: (blob: *D.ID3DBlob) {
    if blob W.IUnknown_Release(cast(*W.IUnknown) blob);
}

main :: () {
    native := module("d3dcompiler_47.dll");

    check(native != null);
    // The SDK import-library name is served by that physical DLL: it is the one loaded, and it
    // exports what the declarations call. (A compiled program's address of an imported procedure
    // is the import library's jump stub, not the export itself, so the addresses are not compared.)
    check(symbol(native, "D3DCreateBlob") != null);
    blob: *D.ID3DBlob;

    check(D.D3DCreateBlob(32, *blob) >= 0 && blob != null);
    check(D.ID3D10Blob_GetBufferSize(blob) == 32);
    bytes := cast(*u8) D.ID3D10Blob_GetBufferPointer(blob);

    check(bytes != null);
    for i: 0..31 bytes[i] = cast(u8) i;
    for i: 0..31 check(bytes[i] == cast(u8) i);

    release(blob);

    shader := "float4 main(float4 position : POSITION) : SV_Position { return position; }";
    code, errors: *D.ID3DBlob;
    status := D.D3DCompile(shader.data, cast(W.SIZE_T) shader.count, "independent-provider.hlsl",
                           null, null, "main", "vs_5_0", 0, 0, *code, *errors);

    check(status >= 0 && code != null);
    check(D.ID3D10Blob_GetBufferPointer(code) != null && D.ID3D10Blob_GetBufferSize(code) > 0);
    release(code);
    release(errors);
    code = null;
    errors = null;
    invalid := "this is not valid HLSL";

    status = D.D3DCompile(invalid.data, cast(W.SIZE_T) invalid.count, "independent-invalid.hlsl",
                         null, null, "main", "vs_5_0", 0, 0, *code, *errors);
    check(status < 0 && errors != null);
    check(D.ID3D10Blob_GetBufferPointer(errors) != null && D.ID3D10Blob_GetBufferSize(errors) > 0);
    release(code);
    release(errors);
}

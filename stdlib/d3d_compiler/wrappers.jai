// Reviewed string overloads deliberately ignore define slices. Native blob
// ownership differs between the copied-memory and direct file interfaces.
D3DCompile :: (shader: string, source_name: string, pDefines: [] string, pInclude: *ID3DInclude, entry_point: string, target: string, Flags1: u32, Flags2: u32) -> bytecode: string, errors: string, HRESULT {
    source := Native.terminated_copy(source_name);

    defer Native.release(source);
    entry := Native.terminated_copy(entry_point);

    defer Native.release(entry);
    profile := Native.terminated_copy(target);

    defer Native.release(profile);
    code: *ID3DBlob;
    diagnostics: *ID3DBlob;
    status := D3DCompile(shader.data, cast(SIZE_T) shader.count, source, null, pInclude, entry, profile, Flags1, Flags2, *code, *diagnostics);

    defer if code Win.IUnknown_Release(cast(*Win.IUnknown) code);
    defer if diagnostics Win.IUnknown_Release(cast(*Win.IUnknown) diagnostics);
    bytecode: string;
    errors: string;

    if code bytecode = Bytes.copy(ID3D10Blob_GetBufferPointer(code), ID3D10Blob_GetBufferSize(code));
    if diagnostics errors = Bytes.copy(ID3D10Blob_GetBufferPointer(diagnostics), ID3D10Blob_GetBufferSize(diagnostics));

    return bytecode, errors, status;
}

D3DCompileFromFile :: (file_name: string, pDefines: [] string, pInclude: *ID3DInclude, entry_point: string, target: string, Flags1: u32, Flags2: u32) -> pCode: *ID3DBlob, pErrorMsgs: *ID3DBlob, HRESULT {
    file := Wide.argument(file_name);
    if !file return null, null, cast,trunc(HRESULT) 0x80070057;

    defer Basic.free(file);
    entry := Native.terminated_copy(entry_point);

    defer Native.release(entry);
    profile := Native.terminated_copy(target);

    defer Native.release(profile);
    code: *ID3DBlob;
    errors: *ID3DBlob;
    status := D3DCompileFromFile(cast(*s16) file, null, pInclude, entry, profile, Flags1, Flags2, *code, *errors);

    return code, errors, status;
}

#scope_file
Win :: #import "Windows";

HRESULT :: Win.HRESULT;
SIZE_T :: Win.SIZE_T;

Basic :: #import "Basic";
Bytes :: #import "internal/bindings/shader-bytes";
Wide :: #import "internal/bindings/dxc-wide";
Native :: #import "internal/bindings/native";

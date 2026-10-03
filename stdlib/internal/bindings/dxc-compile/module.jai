// Native compile route shares our complete generated SDK schema. No D3D12
// reflection dependency is needed to compile HLSL or return owned bytes.
#assert OS == .WINDOWS, "DXC native compilation requires Windows";

#load "../../../dxc_compiler/declarations.jai";

compile :: (args: $T) -> [..] u8 {
    result: [..] u8;
    stages: [8] string = .["vs", "ps", "cs", "hs", "ds", "ms", "as", "lib"];
    stage := cast(s64) args.shader_type;

    if stage < 0 || stage >= stages.count {
        binding_dxc_message("Invalid shader stage");
        return result;
    }

    arguments := Options.build(stages[stage], args);

    defer Options.release(arguments);

    if arguments.count > 0xffff_ffff {
        binding_dxc_message("Too many native compiler arguments");
        return result;
    }

    wide_arguments: [..] *s16;

    defer {
        for text: wide_arguments Basic.free(text);
        Basic.array_free(wide_arguments);
    }

    for text: arguments {
        wide := Wide.argument(text);
        if !wide {
            binding_dxc_message("Compiler argument cannot be represented as a terminated UTF-16 string");
            return result;
        }

        Basic.array_add(*wide_arguments, cast(*s16) wide);
    }

    compiler: *IDxcCompiler3;
    status := DxcCreateInstance(CLSID_DxcCompiler, IDxcCompiler3_UID, cast(**void) *compiler);

    defer if compiler Win.IUnknown_Release(cast(*Win.IUnknown) compiler);

    if status < 0 || !compiler {
        binding_dxc_message("Native compiler creation failed");
        return result;
    }

    utils: *IDxcUtils;

    status = DxcCreateInstance(CLSID_DxcUtils, IDxcUtils_UID, cast(**void) *utils);
    defer if utils Win.IUnknown_Release(cast(*Win.IUnknown) utils);

    if status < 0 || !utils {
        binding_dxc_message("Native utility creation failed");
        return result;
    }

    includes: *IDxcIncludeHandler;

    status = IDxcUtils_CreateDefaultIncludeHandler(utils, *includes);
    defer if includes Win.IUnknown_Release(cast(*Win.IUnknown) includes);

    if status < 0 || !includes {
        binding_dxc_message("Native include handler creation failed");
        return result;
    }

    source: DxcBuffer;
    source.Ptr = args.shader_source.data;
    source.Size = cast(u64) args.shader_source.count;
    source.Encoding = DXC_CP_UTF8;
    iid := IDxcResult_UID;
    operation: *IDxcResult;

    status = IDxcCompiler3_Compile(compiler, *source, wide_arguments.data,
                                  cast(u32) wide_arguments.count, includes, *iid, cast(**void) *operation);
    defer if operation Win.IUnknown_Release(cast(*Win.IUnknown) operation);

    if status < 0 || !operation {
        binding_dxc_message("Native Compile invocation failed");
        return result;
    }

    base := cast(*IDxcOperationResult) operation;
    errors: *IDxcBlobEncoding;

    status = IDxcOperationResult_GetErrorBuffer(base, *errors);
    defer if errors Win.IUnknown_Release(cast(*Win.IUnknown) errors);

    if status < 0 {
        binding_dxc_message("Native diagnostic retrieval failed");
        return result;
    }

    if errors binding_dxc_diagnostics(cast(*IDxcBlob) errors);

    compilation: Win.HRESULT;

    status = IDxcOperationResult_GetStatus(base, *compilation);
    if status < 0 || compilation < 0 {
        binding_dxc_message("Shader compilation failed");
        return result;
    }

    object: *IDxcBlob;

    status = IDxcOperationResult_GetResult(base, *object);
    defer if object Win.IUnknown_Release(cast(*Win.IUnknown) object);

    if status < 0 || !object {
        binding_dxc_message("Native compiled object retrieval failed");
        return result;
    }

    size := IDxcBlob_GetBufferSize(object);
    if size > 0x7fff_ffff_ffff_ffff {
        binding_dxc_message("Native object exceeds the public array length range");
        return result;
    }

    bytes := IDxcBlob_GetBufferPointer(object);
    if size > 0 && !bytes {
        binding_dxc_message("Native object has no data for its advertised size");
        return result;
    }

    Basic.array_resize(*result, cast(s64) size);
    if size > 0 binding_dxc_copy(result.data, bytes, size);

    return result;
}

#scope_file
Win :: #import "Windows";
Basic :: #import "Basic";
Options :: #import "internal/bindings/dxc-options";
Wide :: #import "internal/bindings/dxc-wide";

libc :: #library,system "c";
binding_dxc_printf :: (format: *u8, args: ..Any) -> s32 #foreign libc "printf";
binding_dxc_flush :: (stream: *void) -> s32 #foreign libc "fflush";
binding_dxc_write :: (fd: s32, bytes: *void, count: u32) -> s32 #foreign libc "_write";
binding_dxc_abort :: () #foreign libc "abort";
binding_dxc_copy :: (destination: *void, source: *void, count: u64) -> *void #foreign libc "memcpy";

binding_dxc_message :: (text: *u8) {
    if binding_dxc_printf("DXC: %s\n", text) < 0 binding_dxc_abort();
    if binding_dxc_flush(null) != 0 binding_dxc_abort();
}

binding_dxc_diagnostics :: (blob: *IDxcBlob) {
    remaining := IDxcBlob_GetBufferSize(blob);
    cursor := cast(*u8) IDxcBlob_GetBufferPointer(blob);

    if remaining > 0 && !cursor binding_dxc_abort();

    while remaining > 0 {
        chunk := ifx remaining > 0x7fff_ffff then 0x7fff_ffff else cast(u32) remaining;
        written := binding_dxc_write(2, cursor, chunk);

        if written <= 0 binding_dxc_abort();

        remaining -= cast(u64) written;
        cursor += written;
    }
}

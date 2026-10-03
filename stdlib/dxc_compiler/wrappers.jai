dxc_get_version_string :: () -> string {
    compiler: *IDxcCompiler3;
    status := DxcCreateInstance(CLSID_DxcCompiler, IDxcCompiler3_UID, cast(**void) *compiler);

    defer if compiler Win.IUnknown_Release(cast(*Win.IUnknown) compiler);
    version_check(status);
    VersionBytes.require(compiler != null, "DXC creation returned a null compiler");
    version: *IDxcVersionInfo;
    iid := IDxcVersionInfo_UID;

    status = Win.IUnknown_QueryInterface(cast(*Win.IUnknown) compiler, *iid, cast(**void) *version);
    defer if version Win.IUnknown_Release(cast(*Win.IUnknown) version);
    version_check(status);
    VersionBytes.require(version != null, "DXC query returned null version information");
    major: u32;
    minor: u32;

    version_check(IDxcVersionInfo_GetVersion(version, *major, *minor));
    custom: *IDxcVersionInfo3;

    iid = IDxcVersionInfo3_UID;
    status = Win.IUnknown_QueryInterface(cast(*Win.IUnknown) compiler, *iid, cast(**void) *custom);
    defer if custom Win.IUnknown_Release(cast(*Win.IUnknown) custom);
    version_check(status);
    VersionBytes.require(custom != null, "DXC query returned null custom version information");
    text: *u8;

    status = IDxcVersionInfo3_GetCustomVersionString(custom, *text);
    defer if text version_free_custom(text);
    version_check(status);
    custom_version: string;
    if text custom_version = .{ data = text, count = cast(s64) version_text_length(text) };

    return VersionBasic.sprint("v%.% [%]", major, minor, custom_version);
}

DXC_Compile :: (args: Dxc_Shader_Compiler_Args) -> [..] u8 {
    return NativeCompile.compile(args);
}

DXC_Reflect :: (dxil: [] u8) -> *ID3D12ShaderReflection, bool {
    utils: *IDxcUtils;
    status := DxcCreateInstance(CLSID_DxcUtils, IDxcUtils_UID, cast(**void) *utils);

    if status < 0 {
        // Release any SDK-supplied partial interface, retaining native ownership.
        if utils Win.IUnknown_Release(cast(*Win.IUnknown) utils);
        return null, false;
    }

    if !utils return null, false;

    defer Win.IUnknown_Release(cast(*Win.IUnknown) utils);
    data: DxcBuffer;
    data.Ptr = dxil.data;
    data.Size = cast(u64) dxil.count;
    data.Encoding = DXC_CP_ACP;
    iid := Win.uid(D3D.ID3D12ShaderReflection_UUID);
    reflection: *ID3D12ShaderReflection;

    status = IDxcUtils_CreateReflection(utils, *data, *iid, cast(**void) *reflection);
    if status < 0 {
        if reflection Win.IUnknown_Release(cast(*Win.IUnknown) reflection);
        return null, false;
    }

    if !reflection return null, false;

    // The returned interface owns its native COM reference. Its caller releases
    // it; releasing IDxcUtils does not consume that independent reference.
    return reflection, true;
}

#scope_file
Win :: #import "Windows";
D3D :: #import "d3d12";
NativeCompile :: #import "internal/bindings/dxc-compile";

ID3D12ShaderReflection :: D3D.ID3D12ShaderReflection;

VersionBasic :: #import "Basic";
VersionBytes :: #import "internal/bindings/shader-bytes";

VersionCOM :: #library,system "ole32";
VersionC :: #library,system "c";
version_free_custom :: (memory: *void) #foreign VersionCOM "CoTaskMemFree";
version_text_length :: (text: *u8) -> u64 #foreign VersionC "strlen";

version_check :: (status: Win.HRESULT) {
    VersionBytes.require(status >= 0, "DXC COM version operation failed");
}

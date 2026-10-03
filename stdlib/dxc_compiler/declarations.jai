// Generated from docs/specs/stdlib/modules/dxc-compiler.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

#scope_file
#import "Windows";

IMalloc :: struct {};
IStream :: struct {};
BSTR :: *u16;
dxcompiler :: #library,system "dxcompiler";
#scope_export

// Types and layouts

DXC_OUT_KIND :: enum s32 {
    NONE :: 0;
    OBJECT :: 1;
    ERRORS :: 2;
    PDB :: 3;
    SHADER_HASH :: 4;
    DISASSEMBLY :: 5;
    HLSL :: 6;
    TEXT :: 7;
    REFLECTION :: 8;
    ROOT_SIGNATURE :: 9;
    EXTRA_OUTPUTS :: 10;
    REMARKS :: 11;
    TIME_REPORT :: 12;
    TIME_TRACE :: 13;
    LAST :: 13;
    NUM_ENUMS :: 14;
    FORCE_DWORD :: -1;
    DXC_OUT_NONE :: NONE;
    DXC_OUT_OBJECT :: OBJECT;
    DXC_OUT_ERRORS :: ERRORS;
    DXC_OUT_PDB :: PDB;
    DXC_OUT_SHADER_HASH :: SHADER_HASH;
    DXC_OUT_DISASSEMBLY :: DISASSEMBLY;
    DXC_OUT_HLSL :: HLSL;
    DXC_OUT_TEXT :: TEXT;
    DXC_OUT_REFLECTION :: REFLECTION;
    DXC_OUT_ROOT_SIGNATURE :: ROOT_SIGNATURE;
    DXC_OUT_EXTRA_OUTPUTS :: EXTRA_OUTPUTS;
    DXC_OUT_REMARKS :: REMARKS;
    DXC_OUT_TIME_REPORT :: TIME_REPORT;
    DXC_OUT_TIME_TRACE :: TIME_TRACE;
    DXC_OUT_LAST :: LAST;
    DXC_OUT_NUM_ENUMS :: NUM_ENUMS;
    DXC_OUT_FORCE_DWORD :: FORCE_DWORD;
}

Dxc_Shader_Compiler_Args :: struct {
    shader_name: string;
    shader_source: string;
    shader_model_version: string;
    shader_type: Shader_Type;
    entry_point: string;
    defines: [] string;
    extra_args: [] string;
    add_debug_info: bool;
    warnings_as_error: bool;
    skip_optimization: bool;
    spirv_target_env: string;
    spirv_generate_non_semanantic_debug_info := true;
    turn_off_validation: bool;
}

DxcArgPair :: struct {
    pName: *s16;
    pValue: *s16;
}

DxcBuffer :: struct {
    Ptr: *void;
    Size: SIZE_T;
    Encoding: u32;
}

DxcDefine :: struct {
    Name: *s16;
    Value: *s16;
}

DxcShaderHash :: struct {
    Flags: u32;
    HashDigest: [16] u8;
}

IDxcAssembler :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcAssembler_VTable;
}

IDxcAssembler_VTable :: struct #type_info_none {
    using vtable: IUnknown_VTable;
    AssembleToContainer: #type (this: *IDxcAssembler, pShader: *IDxcBlob, ppResult: **IDxcOperationResult) -> HRESULT #cpp_method;
}

IDxcBlob :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcBlob_VTable;
}

IDxcBlob_VTable :: struct #type_info_none {
    using vtable: IUnknown_VTable;
    GetBufferPointer: #type (this: *IDxcBlob) -> *void #cpp_method;
    GetBufferSize: #type (this: *IDxcBlob) -> SIZE_T #cpp_method;
}

IDxcBlobEncoding :: struct {
    #as using,except(vtable) idxcblob: IDxcBlob;
    #overlay (idxcblob) vtable: *IDxcBlobEncoding_VTable;
}

IDxcBlobEncoding_VTable :: struct #type_info_none {
    using,except(vtable) vtable: IDxcBlob_VTable;
    GetEncoding: #type (this: *IDxcBlobEncoding, pKnown: *BOOL, pCodePage: *u32) -> HRESULT #cpp_method;
}

IDxcBlobUtf8 :: struct {
    #as using,except(vtable) idxcblobencoding: IDxcBlobEncoding;
    #overlay (idxcblobencoding) vtable: *IDxcBlobUtf8_VTable;
}

IDxcBlobUtf8_VTable :: struct #type_info_none {
    using,except(vtable) vtable: IDxcBlobEncoding_VTable;
    GetStringPointer: #type (this: *IDxcBlobUtf8) -> *u8 #cpp_method;
    GetStringLength: #type (this: *IDxcBlobUtf8) -> SIZE_T #cpp_method;
}

IDxcBlobWide :: struct {
    #as using,except(vtable) idxcblobencoding: IDxcBlobEncoding;
    #overlay (idxcblobencoding) vtable: *IDxcBlobWide_VTable;
}

IDxcBlobWide_VTable :: struct #type_info_none {
    using,except(vtable) vtable: IDxcBlobEncoding_VTable;
    GetStringPointer: #type (this: *IDxcBlobWide) -> *s16 #cpp_method;
    GetStringLength: #type (this: *IDxcBlobWide) -> SIZE_T #cpp_method;
}

IDxcCompiler :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcCompiler_VTable;
}

IDxcCompiler2 :: struct {
    #as using,except(vtable) idxccompiler: IDxcCompiler;
    #overlay (idxccompiler) vtable: *IDxcCompiler2_VTable;
}

IDxcCompiler2_VTable :: struct #type_info_none {
    using,except(vtable) vtable: IDxcCompiler_VTable;
    CompileWithDebug: #type (this: *IDxcCompiler2, pSource: *IDxcBlob, pSourceName: *s16, pEntryPoint: *s16, pTargetProfile: *s16, pArguments: **s16, argCount: u32, pDefines: *DxcDefine, defineCount: u32, pIncludeHandler: *IDxcIncludeHandler, ppResult: **IDxcOperationResult, ppDebugBlobName: **s16, ppDebugBlob: **IDxcBlob) -> HRESULT #cpp_method;
}

IDxcCompiler3 :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcCompiler3_VTable;
}

IDxcCompiler3_VTable :: struct #type_info_none {
    using vtable: IUnknown_VTable;
    Compile: #type (this: *IDxcCompiler3, pSource: *DxcBuffer, pArguments: **s16, argCount: u32, pIncludeHandler: *IDxcIncludeHandler, riid: *IID, ppResult: **void) -> HRESULT #cpp_method;
    Disassemble: #type (this: *IDxcCompiler3, pObject: *DxcBuffer, riid: *IID, ppResult: **void) -> HRESULT #cpp_method;
}

IDxcCompiler_VTable :: struct #type_info_none {
    using vtable: IUnknown_VTable;
    Compile: #type (this: *IDxcCompiler, pSource: *IDxcBlob, pSourceName: *s16, pEntryPoint: *s16, pTargetProfile: *s16, pArguments: **s16, argCount: u32, pDefines: *DxcDefine, defineCount: u32, pIncludeHandler: *IDxcIncludeHandler, ppResult: **IDxcOperationResult) -> HRESULT #cpp_method;
    Preprocess: #type (this: *IDxcCompiler, pSource: *IDxcBlob, pSourceName: *s16, pArguments: **s16, argCount: u32, pDefines: *DxcDefine, defineCount: u32, pIncludeHandler: *IDxcIncludeHandler, ppResult: **IDxcOperationResult) -> HRESULT #cpp_method;
    Disassemble: #type (this: *IDxcCompiler, pSource: *IDxcBlob, ppDisassembly: **IDxcBlobEncoding) -> HRESULT #cpp_method;
}

IDxcCompilerArgs :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcCompilerArgs_VTable;
}

IDxcCompilerArgs_VTable :: struct #type_info_none {
    using vtable: IUnknown_VTable;
    GetArguments: #type (this: *IDxcCompilerArgs) -> **s16 #cpp_method;
    GetCount: #type (this: *IDxcCompilerArgs) -> u32 #cpp_method;
    AddArguments: #type (this: *IDxcCompilerArgs, pArguments: **s16, argCount: u32) -> HRESULT #cpp_method;
    AddArgumentsUTF8: #type (this: *IDxcCompilerArgs, pArguments: **u8, argCount: u32) -> HRESULT #cpp_method;
    AddDefines: #type (this: *IDxcCompilerArgs, pDefines: *DxcDefine, defineCount: u32) -> HRESULT #cpp_method;
}

IDxcContainerBuilder :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcContainerBuilder_VTable;
}

IDxcContainerBuilder_VTable :: struct #type_info_none {
    using vtable: IUnknown_VTable;
    Load: #type (this: *IDxcContainerBuilder, pDxilContainerHeader: *IDxcBlob) -> HRESULT #cpp_method;
    AddPart: #type (this: *IDxcContainerBuilder, fourCC: u32, pSource: *IDxcBlob) -> HRESULT #cpp_method;
    RemovePart: #type (this: *IDxcContainerBuilder, fourCC: u32) -> HRESULT #cpp_method;
    SerializeContainer: #type (this: *IDxcContainerBuilder, ppResult: **IDxcOperationResult) -> HRESULT #cpp_method;
}

IDxcContainerReflection :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcContainerReflection_VTable;
}

IDxcContainerReflection_VTable :: struct #type_info_none {
    using vtable: IUnknown_VTable;
    Load: #type (this: *IDxcContainerReflection, pContainer: *IDxcBlob) -> HRESULT #cpp_method;
    GetPartCount: #type (this: *IDxcContainerReflection, pResult: *u32) -> HRESULT #cpp_method;
    GetPartKind: #type (this: *IDxcContainerReflection, idx: u32, pResult: *u32) -> HRESULT #cpp_method;
    GetPartContent: #type (this: *IDxcContainerReflection, idx: u32, ppResult: **IDxcBlob) -> HRESULT #cpp_method;
    FindFirstPartKind: #type (this: *IDxcContainerReflection, kind: u32, pResult: *u32) -> HRESULT #cpp_method;
    GetPartReflection: #type (this: *IDxcContainerReflection, idx: u32, iid: *IID, ppvObject: **void) -> HRESULT #cpp_method;
}

IDxcExtraOutputs :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcExtraOutputs_VTable;
}

IDxcExtraOutputs_VTable :: struct #type_info_none {
    using vtable: IUnknown_VTable;
    GetOutputCount: #type (this: *IDxcExtraOutputs) -> u32 #cpp_method;
    GetOutput: #type (this: *IDxcExtraOutputs, uIndex: u32, iid: *IID, ppvObject: **void, ppOutputType: **IDxcBlobWide, ppOutputName: **IDxcBlobWide) -> HRESULT #cpp_method;
}

IDxcIncludeHandler :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcIncludeHandler_VTable;
}

IDxcIncludeHandler_VTable :: struct #type_info_none {
    using vtable: IUnknown_VTable;
    LoadSource: #type (this: *IDxcIncludeHandler, pFilename: *s16, ppIncludeSource: **IDxcBlob) -> HRESULT #cpp_method;
}

IDxcLibrary :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcLibrary_VTable;
}

IDxcLibrary_VTable :: struct #type_info_none {
    using vtable: IUnknown_VTable;
    SetMalloc: #type (this: *IDxcLibrary, pMalloc: *IMalloc) -> HRESULT #cpp_method;
    CreateBlobFromBlob: #type (this: *IDxcLibrary, pBlob: *IDxcBlob, offset: u32, length: u32, ppResult: **IDxcBlob) -> HRESULT #cpp_method;
    CreateBlobFromFile: #type (this: *IDxcLibrary, pFileName: *s16, codePage: *u32, pBlobEncoding: **IDxcBlobEncoding) -> HRESULT #cpp_method;
    CreateBlobWithEncodingFromPinned: #type (this: *IDxcLibrary, pText: *void, size: u32, codePage: u32, pBlobEncoding: **IDxcBlobEncoding) -> HRESULT #cpp_method;
    CreateBlobWithEncodingOnHeapCopy: #type (this: *IDxcLibrary, pText: *void, size: u32, codePage: u32, pBlobEncoding: **IDxcBlobEncoding) -> HRESULT #cpp_method;
    CreateBlobWithEncodingOnMalloc: #type (this: *IDxcLibrary, pText: *void, pIMalloc: *IMalloc, size: u32, codePage: u32, pBlobEncoding: **IDxcBlobEncoding) -> HRESULT #cpp_method;
    CreateIncludeHandler: #type (this: *IDxcLibrary, ppResult: **IDxcIncludeHandler) -> HRESULT #cpp_method;
    CreateStreamFromBlobReadOnly: #type (this: *IDxcLibrary, pBlob: *IDxcBlob, ppStream: **IStream) -> HRESULT #cpp_method;
    GetBlobAsUtf8: #type (this: *IDxcLibrary, pBlob: *IDxcBlob, pBlobEncoding: **IDxcBlobEncoding) -> HRESULT #cpp_method;
    GetBlobAsWide: #type (this: *IDxcLibrary, pBlob: *IDxcBlob, pBlobEncoding: **IDxcBlobEncoding) -> HRESULT #cpp_method;
}

IDxcLinker :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcLinker_VTable;
}

IDxcLinker_VTable :: struct #type_info_none {
    using vtable: IUnknown_VTable;
    RegisterLibrary: #type (this: *IDxcLinker, pLibName: *s16, pLib: *IDxcBlob) -> HRESULT #cpp_method;
    Link: #type (this: *IDxcLinker, pEntryName: *s16, pTargetProfile: *s16, pLibNames: **s16, libCount: u32, pArguments: **s16, argCount: u32, ppResult: **IDxcOperationResult) -> HRESULT #cpp_method;
}

IDxcOperationResult :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcOperationResult_VTable;
}

IDxcOperationResult_VTable :: struct #type_info_none {
    using vtable: IUnknown_VTable;
    GetStatus: #type (this: *IDxcOperationResult, pStatus: *HRESULT) -> HRESULT #cpp_method;
    GetResult: #type (this: *IDxcOperationResult, ppResult: **IDxcBlob) -> HRESULT #cpp_method;
    GetErrorBuffer: #type (this: *IDxcOperationResult, ppErrors: **IDxcBlobEncoding) -> HRESULT #cpp_method;
}

IDxcOptimizer :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcOptimizer_VTable;
}

IDxcOptimizer_VTable :: struct #type_info_none {
    using vtable: IUnknown_VTable;
    GetAvailablePassCount: #type (this: *IDxcOptimizer, pCount: *u32) -> HRESULT #cpp_method;
    GetAvailablePass: #type (this: *IDxcOptimizer, index: u32, ppResult: **IDxcOptimizerPass) -> HRESULT #cpp_method;
    RunOptimizer: #type (this: *IDxcOptimizer, pBlob: *IDxcBlob, ppOptions: **s16, optionCount: u32, pOutputModule: **IDxcBlob, ppOutputText: **IDxcBlobEncoding) -> HRESULT #cpp_method;
}

IDxcOptimizerPass :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcOptimizerPass_VTable;
}

IDxcOptimizerPass_VTable :: struct #type_info_none {
    using vtable: IUnknown_VTable;
    GetOptionName: #type (this: *IDxcOptimizerPass, ppResult: **s16) -> HRESULT #cpp_method;
    GetDescription: #type (this: *IDxcOptimizerPass, ppResult: **s16) -> HRESULT #cpp_method;
    GetOptionArgCount: #type (this: *IDxcOptimizerPass, pCount: *u32) -> HRESULT #cpp_method;
    GetOptionArgName: #type (this: *IDxcOptimizerPass, argIndex: u32, ppResult: **s16) -> HRESULT #cpp_method;
    GetOptionArgDescription: #type (this: *IDxcOptimizerPass, argIndex: u32, ppResult: **s16) -> HRESULT #cpp_method;
}

IDxcPdbUtils :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcPdbUtils_VTable;
}

IDxcPdbUtils2 :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcPdbUtils2_VTable;
}

IDxcPdbUtils2_VTable :: struct #type_info_none {
    using vtable: IUnknown_VTable;
    Load: #type (this: *IDxcPdbUtils2, pPdbOrDxil: *IDxcBlob) -> HRESULT #cpp_method;
    GetSourceCount: #type (this: *IDxcPdbUtils2, pCount: *u32) -> HRESULT #cpp_method;
    GetSource: #type (this: *IDxcPdbUtils2, uIndex: u32, ppResult: **IDxcBlobEncoding) -> HRESULT #cpp_method;
    GetSourceName: #type (this: *IDxcPdbUtils2, uIndex: u32, ppResult: **IDxcBlobWide) -> HRESULT #cpp_method;
    GetLibraryPDBCount: #type (this: *IDxcPdbUtils2, pCount: *u32) -> HRESULT #cpp_method;
    GetLibraryPDB: #type (this: *IDxcPdbUtils2, uIndex: u32, ppOutPdbUtils: **IDxcPdbUtils2, ppLibraryName: **IDxcBlobWide) -> HRESULT #cpp_method;
    GetFlagCount: #type (this: *IDxcPdbUtils2, pCount: *u32) -> HRESULT #cpp_method;
    GetFlag: #type (this: *IDxcPdbUtils2, uIndex: u32, ppResult: **IDxcBlobWide) -> HRESULT #cpp_method;
    GetArgCount: #type (this: *IDxcPdbUtils2, pCount: *u32) -> HRESULT #cpp_method;
    GetArg: #type (this: *IDxcPdbUtils2, uIndex: u32, ppResult: **IDxcBlobWide) -> HRESULT #cpp_method;
    GetArgPairCount: #type (this: *IDxcPdbUtils2, pCount: *u32) -> HRESULT #cpp_method;
    GetArgPair: #type (this: *IDxcPdbUtils2, uIndex: u32, ppName: **IDxcBlobWide, ppValue: **IDxcBlobWide) -> HRESULT #cpp_method;
    GetDefineCount: #type (this: *IDxcPdbUtils2, pCount: *u32) -> HRESULT #cpp_method;
    GetDefine: #type (this: *IDxcPdbUtils2, uIndex: u32, ppResult: **IDxcBlobWide) -> HRESULT #cpp_method;
    GetTargetProfile: #type (this: *IDxcPdbUtils2, ppResult: **IDxcBlobWide) -> HRESULT #cpp_method;
    GetEntryPoint: #type (this: *IDxcPdbUtils2, ppResult: **IDxcBlobWide) -> HRESULT #cpp_method;
    GetMainFileName: #type (this: *IDxcPdbUtils2, ppResult: **IDxcBlobWide) -> HRESULT #cpp_method;
    GetHash: #type (this: *IDxcPdbUtils2, ppResult: **IDxcBlob) -> HRESULT #cpp_method;
    GetName: #type (this: *IDxcPdbUtils2, ppResult: **IDxcBlobWide) -> HRESULT #cpp_method;
    GetVersionInfo: #type (this: *IDxcPdbUtils2, ppVersionInfo: **IDxcVersionInfo) -> HRESULT #cpp_method;
    GetCustomToolchainID: #type (this: *IDxcPdbUtils2, pID: *u32) -> HRESULT #cpp_method;
    GetCustomToolchainData: #type (this: *IDxcPdbUtils2, ppBlob: **IDxcBlob) -> HRESULT #cpp_method;
    GetWholeDxil: #type (this: *IDxcPdbUtils2, ppResult: **IDxcBlob) -> HRESULT #cpp_method;
    IsFullPDB: #type (this: *IDxcPdbUtils2) -> BOOL #cpp_method;
    IsPDBRef: #type (this: *IDxcPdbUtils2) -> BOOL #cpp_method;
}

IDxcPdbUtils_VTable :: struct #type_info_none {
    using vtable: IUnknown_VTable;
    Load: #type (this: *IDxcPdbUtils, pPdbOrDxil: *IDxcBlob) -> HRESULT #cpp_method;
    GetSourceCount: #type (this: *IDxcPdbUtils, pCount: *u32) -> HRESULT #cpp_method;
    GetSource: #type (this: *IDxcPdbUtils, uIndex: u32, ppResult: **IDxcBlobEncoding) -> HRESULT #cpp_method;
    GetSourceName: #type (this: *IDxcPdbUtils, uIndex: u32, pResult: *BSTR) -> HRESULT #cpp_method;
    GetFlagCount: #type (this: *IDxcPdbUtils, pCount: *u32) -> HRESULT #cpp_method;
    GetFlag: #type (this: *IDxcPdbUtils, uIndex: u32, pResult: *BSTR) -> HRESULT #cpp_method;
    GetArgCount: #type (this: *IDxcPdbUtils, pCount: *u32) -> HRESULT #cpp_method;
    GetArg: #type (this: *IDxcPdbUtils, uIndex: u32, pResult: *BSTR) -> HRESULT #cpp_method;
    GetArgPairCount: #type (this: *IDxcPdbUtils, pCount: *u32) -> HRESULT #cpp_method;
    GetArgPair: #type (this: *IDxcPdbUtils, uIndex: u32, pName: *BSTR, pValue: *BSTR) -> HRESULT #cpp_method;
    GetDefineCount: #type (this: *IDxcPdbUtils, pCount: *u32) -> HRESULT #cpp_method;
    GetDefine: #type (this: *IDxcPdbUtils, uIndex: u32, pResult: *BSTR) -> HRESULT #cpp_method;
    GetTargetProfile: #type (this: *IDxcPdbUtils, pResult: *BSTR) -> HRESULT #cpp_method;
    GetEntryPoint: #type (this: *IDxcPdbUtils, pResult: *BSTR) -> HRESULT #cpp_method;
    GetMainFileName: #type (this: *IDxcPdbUtils, pResult: *BSTR) -> HRESULT #cpp_method;
    GetHash: #type (this: *IDxcPdbUtils, ppResult: **IDxcBlob) -> HRESULT #cpp_method;
    GetName: #type (this: *IDxcPdbUtils, pResult: *BSTR) -> HRESULT #cpp_method;
    IsFullPDB: #type (this: *IDxcPdbUtils) -> BOOL #cpp_method;
    GetFullPDB: #type (this: *IDxcPdbUtils, ppFullPDB: **IDxcBlob) -> HRESULT #cpp_method;
    GetVersionInfo: #type (this: *IDxcPdbUtils, ppVersionInfo: **IDxcVersionInfo) -> HRESULT #cpp_method;
    SetCompiler: #type (this: *IDxcPdbUtils, pCompiler: *IDxcCompiler3) -> HRESULT #cpp_method;
    CompileForFullPDB: #type (this: *IDxcPdbUtils, ppResult: **IDxcResult) -> HRESULT #cpp_method;
    OverrideArgs: #type (this: *IDxcPdbUtils, pArgPairs: *DxcArgPair, uNumArgPairs: u32) -> HRESULT #cpp_method;
    OverrideRootSignature: #type (this: *IDxcPdbUtils, pRootSignature: *s16) -> HRESULT #cpp_method;
}

IDxcResult :: struct {
    #as using,except(vtable) idxcoperationresult: IDxcOperationResult;
    #overlay (idxcoperationresult) vtable: *IDxcResult_VTable;
}

IDxcResult_VTable :: struct #type_info_none {
    using,except(vtable) vtable: IDxcOperationResult_VTable;
    HasOutput: #type (this: *IDxcResult, dxcOutKind: DXC_OUT_KIND) -> BOOL #cpp_method;
    GetOutput: #type (this: *IDxcResult, dxcOutKind: DXC_OUT_KIND, iid: *IID, ppvObject: **void, ppOutputName: **IDxcBlobWide) -> HRESULT #cpp_method;
    GetNumOutputs: #type (this: *IDxcResult) -> u32 #cpp_method;
    GetOutputByIndex: #type (this: *IDxcResult, Index: u32) -> DXC_OUT_KIND #cpp_method;
    PrimaryOutput: #type (this: *IDxcResult) -> DXC_OUT_KIND #cpp_method;
}

IDxcUtils :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcUtils_VTable;
}

IDxcUtils_VTable :: struct #type_info_none {
    using vtable: IUnknown_VTable;
    CreateBlobFromBlob: #type (this: *IDxcUtils, pBlob: *IDxcBlob, offset: u32, length: u32, ppResult: **IDxcBlob) -> HRESULT #cpp_method;
    CreateBlobFromPinned: #type (this: *IDxcUtils, pData: *void, size: u32, codePage: u32, ppBlobEncoding: **IDxcBlobEncoding) -> HRESULT #cpp_method;
    MoveToBlob: #type (this: *IDxcUtils, pData: *void, pIMalloc: *IMalloc, size: u32, codePage: u32, ppBlobEncoding: **IDxcBlobEncoding) -> HRESULT #cpp_method;
    CreateBlob: #type (this: *IDxcUtils, pData: *void, size: u32, codePage: u32, ppBlobEncoding: **IDxcBlobEncoding) -> HRESULT #cpp_method;
    LoadFile: #type (this: *IDxcUtils, pFileName: *s16, pCodePage: *u32, ppBlobEncoding: **IDxcBlobEncoding) -> HRESULT #cpp_method;
    CreateReadOnlyStreamFromBlob: #type (this: *IDxcUtils, pBlob: *IDxcBlob, ppStream: **IStream) -> HRESULT #cpp_method;
    CreateDefaultIncludeHandler: #type (this: *IDxcUtils, ppResult: **IDxcIncludeHandler) -> HRESULT #cpp_method;
    GetBlobAsUtf8: #type (this: *IDxcUtils, pBlob: *IDxcBlob, ppBlobEncoding: **IDxcBlobUtf8) -> HRESULT #cpp_method;
    GetBlobAsWide: #type (this: *IDxcUtils, pBlob: *IDxcBlob, ppBlobEncoding: **IDxcBlobWide) -> HRESULT #cpp_method;
    GetDxilContainerPart: #type (this: *IDxcUtils, pShader: *DxcBuffer, DxcPart: u32, ppPartData: **void, pPartSizeInBytes: *u32) -> HRESULT #cpp_method;
    CreateReflection: #type (this: *IDxcUtils, pData: *DxcBuffer, iid: *IID, ppvReflection: **void) -> HRESULT #cpp_method;
    BuildArguments: #type (this: *IDxcUtils, pSourceName: *s16, pEntryPoint: *s16, pTargetProfile: *s16, pArguments: **s16, argCount: u32, pDefines: *DxcDefine, defineCount: u32, ppArgs: **IDxcCompilerArgs) -> HRESULT #cpp_method;
    GetPDBContents: #type (this: *IDxcUtils, pPDBBlob: *IDxcBlob, ppHash: **IDxcBlob, ppContainer: **IDxcBlob) -> HRESULT #cpp_method;
}

IDxcValidator :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcValidator_VTable;
}

IDxcValidator2 :: struct {
    #as using,except(vtable) idxcvalidator: IDxcValidator;
    #overlay (idxcvalidator) vtable: *IDxcValidator2_VTable;
}

IDxcValidator2_VTable :: struct #type_info_none {
    using,except(vtable) vtable: IDxcValidator_VTable;
    ValidateWithDebug: #type (this: *IDxcValidator2, pShader: *IDxcBlob, Flags: u32, pOptDebugBitcode: *DxcBuffer, ppResult: **IDxcOperationResult) -> HRESULT #cpp_method;
}

IDxcValidator_VTable :: struct #type_info_none {
    using vtable: IUnknown_VTable;
    Validate: #type (this: *IDxcValidator, pShader: *IDxcBlob, Flags: u32, ppResult: **IDxcOperationResult) -> HRESULT #cpp_method;
}

IDxcVersionInfo :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcVersionInfo_VTable;
}

IDxcVersionInfo2 :: struct {
    #as using,except(vtable) idxcversioninfo: IDxcVersionInfo;
    #overlay (idxcversioninfo) vtable: *IDxcVersionInfo2_VTable;
}

IDxcVersionInfo2_VTable :: struct #type_info_none {
    using,except(vtable) vtable: IDxcVersionInfo_VTable;
    GetCommitInfo: #type (this: *IDxcVersionInfo2, pCommitCount: *u32, pCommitHash: **u8) -> HRESULT #cpp_method;
}

IDxcVersionInfo3 :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcVersionInfo3_VTable;
}

IDxcVersionInfo3_VTable :: struct #type_info_none {
    using vtable: IUnknown_VTable;
    GetCustomVersionString: #type (this: *IDxcVersionInfo3, pVersionString: **u8) -> HRESULT #cpp_method;
}

IDxcVersionInfo_VTable :: struct #type_info_none {
    using vtable: IUnknown_VTable;
    GetVersion: #type (this: *IDxcVersionInfo, pMajor: *u32, pMinor: *u32) -> HRESULT #cpp_method;
    GetFlags: #type (this: *IDxcVersionInfo, pFlags: *u32) -> HRESULT #cpp_method;
}

Shader_Type :: enum {
    Vertex;
    Fragment;
    Compute;
    Hull;
    Domain;
    Mesh;
    Amplification;
    Library;
}

// Callback and opaque types

DxcCreateInstance2Proc :: #type (pMalloc: *IMalloc, rclsid: *IID, riid: *IID, ppv: **void) -> HRESULT #c_call;

DxcCreateInstanceProc :: #type (rclsid: *IID, riid: *IID, ppv: **void) -> HRESULT #c_call;

// Aliases and constants

CLSID_DxcCompiler :: GUID.{ 0x73E22D93, 0xE6CE, 0x47F3, .[0xB5, 0xBF, 0xF0, 0x66, 0x4F, 0x39, 0xC1, 0xB0] };

CLSID_DxcPdbUtils :: GUID.{ 0x54621DFB, 0xF2CE, 0x457E, .[0xAE, 0x8C, 0xEC, 0x35, 0x5F, 0xAE, 0xEC, 0x7C] };

CLSID_DxcUtils :: GUID.{ 0x6245D6AF, 0x66E0, 0x48FD, .[0x80, 0xB4, 0x4D, 0x27, 0x17, 0x96, 0x74, 0x8C] };

DXC_CP_ACP :: 0;

DXC_CP_UTF16 :: 1200;

DXC_CP_UTF32 :: 12000;

DXC_CP_UTF8 :: 65001;

DXC_CP_WIDE :: DXC_CP_UTF16;

DXC_HASHFLAG_INCLUDES_SOURCE :: 1;

DxcText :: DxcBuffer;

IDxcBlobUtf16 :: IDxcBlobWide;

IDxcBlobUtf8_UID :: GUID.{ 0x3DA636C9, 0xBA71, 0x4024, .[0xA3, 0x01, 0x30, 0xCB, 0xF1, 0x25, 0x30, 0x5B] };

IDxcCompiler3_UID :: GUID.{ 0x228B4687, 0x5A6A, 0x4730, .[0x90, 0x0C, 0x97, 0x02, 0xB2, 0x20, 0x3F, 0x54] };

IDxcPdbUtils_UID :: GUID.{ 0xE6C9647E, 0x9D6A, 0x4C3B, .[0xB9, 0x4C, 0x52, 0x4B, 0x5A, 0x6C, 0x34, 0x3D] };

IDxcResult_UID :: GUID.{ 0x58346CDA, 0xDDE7, 0x4497, .[0x94, 0x61, 0x6F, 0x87, 0xAF, 0x5E, 0x06, 0x59] };

IDxcUtils_UID :: GUID.{ 0x4605C4CB, 0x2019, 0x492A, .[0xAD, 0xA4, 0x65, 0xF2, 0x0B, 0xB7, 0xD6, 0x7F] };

IDxcVersionInfo2_UID :: GUID.{ 0xFB6904C4, 0x42F0, 0x4B62, .[0x9C, 0x46, 0x98, 0x3A, 0xF7, 0xDA, 0x7C, 0x83] };

IDxcVersionInfo3_UID :: GUID.{ 0x5E13E843, 0x9D25, 0x473C, .[0x9A, 0xD2, 0x03, 0xB2, 0xD0, 0xB4, 0x4B, 0x1E] };

IDxcVersionInfo_UID :: GUID.{ 0xB04F5B50, 0x2059, 0x4F12, .[0xA8, 0xFF, 0xA1, 0xE0, 0xCD, 0xE1, 0xCC, 0x7E] };

// Native storage

DxcValidatorFlags_Default: u32 : 0;

DxcValidatorFlags_InPlaceEdit: u32 : 1;

DxcValidatorFlags_RootSignatureOnly: u32 : 2;

DxcValidatorFlags_ModuleOnly: u32 : 4;

DxcValidatorFlags_ValidMask: u32 : 7;

DxcVersionInfoFlags_None: u32 : 0;

DxcVersionInfoFlags_Debug: u32 : 1;

DxcVersionInfoFlags_Internal: u32 : 2;

// Native entry points

DxcCreateInstance :: (rclsid: *IID, riid: *IID, ppv: **void) -> HRESULT #foreign dxcompiler;

DxcCreateInstance2 :: (pMalloc: *IMalloc, rclsid: *IID, riid: *IID, ppv: **void) -> HRESULT #foreign dxcompiler;

// Convenience operations

DxcCreateInstance :: (rclsid: IID, riid: IID, ppv: **void) -> HRESULT #no_context {
    return DxcCreateInstance(*rclsid, *riid, ppv);
}

DxcCreateInstance2 :: (pMalloc: *IMalloc, rclsid: IID, riid: IID, ppv: **void) -> HRESULT #no_context {
    return DxcCreateInstance2(pMalloc, *rclsid, *riid, ppv);
}

IDxcAssembler_AssembleToContainer :: inline (this: *IDxcAssembler, pShader: *IDxcBlob, ppResult: **IDxcOperationResult) -> HRESULT {
    return this.vtable.AssembleToContainer(this, pShader, ppResult);
}

IDxcBlob_GetBufferPointer :: inline (this: *IDxcBlob) -> *void {
    return this.vtable.GetBufferPointer(this);
}

IDxcBlob_GetBufferSize :: inline (this: *IDxcBlob) -> SIZE_T {
    return this.vtable.GetBufferSize(this);
}

IDxcBlobEncoding_GetEncoding :: inline (this: *IDxcBlobEncoding, pKnown: *BOOL, pCodePage: *u32) -> HRESULT {
    return this.vtable.GetEncoding(this, pKnown, pCodePage);
}

IDxcBlobUtf8_GetStringLength :: inline (this: *IDxcBlobUtf8) -> SIZE_T {
    return this.vtable.GetStringLength(this);
}

IDxcBlobUtf8_GetStringPointer :: inline (this: *IDxcBlobUtf8) -> *u8 {
    return this.vtable.GetStringPointer(this);
}

IDxcBlobWide_GetStringLength :: inline (this: *IDxcBlobUtf16) -> SIZE_T {
    return this.vtable.GetStringLength(this);
}

IDxcBlobWide_GetStringPointer :: inline (this: *IDxcBlobUtf16) -> *s16 {
    return this.vtable.GetStringPointer(this);
}

IDxcCompiler2_CompileWithDebug :: inline (this: *IDxcCompiler2, pSource: *IDxcBlob, pSourceName: *s16, pEntryPoint: *s16, pTargetProfile: *s16, pArguments: **s16, argCount: u32, pDefines: *DxcDefine, defineCount: u32, pIncludeHandler: *IDxcIncludeHandler, ppResult: **IDxcOperationResult, ppDebugBlobName: **s16, ppDebugBlob: **IDxcBlob) -> HRESULT {
    return this.vtable.CompileWithDebug(this, pSource, pSourceName, pEntryPoint, pTargetProfile, pArguments, argCount, pDefines, defineCount, pIncludeHandler, ppResult, ppDebugBlobName, ppDebugBlob);
}

IDxcCompiler3_Compile :: inline (this: *IDxcCompiler3, pSource: *DxcText, pArguments: **s16, argCount: u32, pIncludeHandler: *IDxcIncludeHandler, riid: *IID, ppResult: **void) -> HRESULT {
    return this.vtable.Compile(this, pSource, pArguments, argCount, pIncludeHandler, riid, ppResult);
}

IDxcCompiler3_Disassemble :: inline (this: *IDxcCompiler3, pObject: *DxcText, riid: *IID, ppResult: **void) -> HRESULT {
    return this.vtable.Disassemble(this, pObject, riid, ppResult);
}

IDxcCompiler_Compile :: inline (this: *IDxcCompiler, pSource: *IDxcBlob, pSourceName: *s16, pEntryPoint: *s16, pTargetProfile: *s16, pArguments: **s16, argCount: u32, pDefines: *DxcDefine, defineCount: u32, pIncludeHandler: *IDxcIncludeHandler, ppResult: **IDxcOperationResult) -> HRESULT {
    return this.vtable.Compile(this, pSource, pSourceName, pEntryPoint, pTargetProfile, pArguments, argCount, pDefines, defineCount, pIncludeHandler, ppResult);
}

IDxcCompiler_Disassemble :: inline (this: *IDxcCompiler, pSource: *IDxcBlob, ppDisassembly: **IDxcBlobEncoding) -> HRESULT {
    return this.vtable.Disassemble(this, pSource, ppDisassembly);
}

IDxcCompiler_Preprocess :: inline (this: *IDxcCompiler, pSource: *IDxcBlob, pSourceName: *s16, pArguments: **s16, argCount: u32, pDefines: *DxcDefine, defineCount: u32, pIncludeHandler: *IDxcIncludeHandler, ppResult: **IDxcOperationResult) -> HRESULT {
    return this.vtable.Preprocess(this, pSource, pSourceName, pArguments, argCount, pDefines, defineCount, pIncludeHandler, ppResult);
}

IDxcCompilerArgs_AddArguments :: inline (this: *IDxcCompilerArgs, pArguments: **s16, argCount: u32) -> HRESULT {
    return this.vtable.AddArguments(this, pArguments, argCount);
}

IDxcCompilerArgs_AddArgumentsUTF8 :: inline (this: *IDxcCompilerArgs, pArguments: **u8, argCount: u32) -> HRESULT {
    return this.vtable.AddArgumentsUTF8(this, pArguments, argCount);
}

IDxcCompilerArgs_AddDefines :: inline (this: *IDxcCompilerArgs, pDefines: *DxcDefine, defineCount: u32) -> HRESULT {
    return this.vtable.AddDefines(this, pDefines, defineCount);
}

IDxcCompilerArgs_GetArguments :: inline (this: *IDxcCompilerArgs) -> **s16 {
    return this.vtable.GetArguments(this);
}

IDxcCompilerArgs_GetCount :: inline (this: *IDxcCompilerArgs) -> u32 {
    return this.vtable.GetCount(this);
}

IDxcContainerBuilder_AddPart :: inline (this: *IDxcContainerBuilder, fourCC: u32, pSource: *IDxcBlob) -> HRESULT {
    return this.vtable.AddPart(this, fourCC, pSource);
}

IDxcContainerBuilder_Load :: inline (this: *IDxcContainerBuilder, pDxilContainerHeader: *IDxcBlob) -> HRESULT {
    return this.vtable.Load(this, pDxilContainerHeader);
}

IDxcContainerBuilder_RemovePart :: inline (this: *IDxcContainerBuilder, fourCC: u32) -> HRESULT {
    return this.vtable.RemovePart(this, fourCC);
}

IDxcContainerBuilder_SerializeContainer :: inline (this: *IDxcContainerBuilder, ppResult: **IDxcOperationResult) -> HRESULT {
    return this.vtable.SerializeContainer(this, ppResult);
}

IDxcContainerReflection_FindFirstPartKind :: inline (this: *IDxcContainerReflection, kind: u32, pResult: *u32) -> HRESULT {
    return this.vtable.FindFirstPartKind(this, kind, pResult);
}

IDxcContainerReflection_GetPartContent :: inline (this: *IDxcContainerReflection, idx: u32, ppResult: **IDxcBlob) -> HRESULT {
    return this.vtable.GetPartContent(this, idx, ppResult);
}

IDxcContainerReflection_GetPartCount :: inline (this: *IDxcContainerReflection, pResult: *u32) -> HRESULT {
    return this.vtable.GetPartCount(this, pResult);
}

IDxcContainerReflection_GetPartKind :: inline (this: *IDxcContainerReflection, idx: u32, pResult: *u32) -> HRESULT {
    return this.vtable.GetPartKind(this, idx, pResult);
}

IDxcContainerReflection_GetPartReflection :: inline (this: *IDxcContainerReflection, idx: u32, iid: *IID, ppvObject: **void) -> HRESULT {
    return this.vtable.GetPartReflection(this, idx, iid, ppvObject);
}

IDxcContainerReflection_Load :: inline (this: *IDxcContainerReflection, pContainer: *IDxcBlob) -> HRESULT {
    return this.vtable.Load(this, pContainer);
}

IDxcExtraOutputs_GetOutput :: inline (this: *IDxcExtraOutputs, uIndex: u32, iid: *IID, ppvObject: **void, ppOutputType: **IDxcBlobUtf16, ppOutputName: **IDxcBlobUtf16) -> HRESULT {
    return this.vtable.GetOutput(this, uIndex, iid, ppvObject, ppOutputType, ppOutputName);
}

IDxcExtraOutputs_GetOutputCount :: inline (this: *IDxcExtraOutputs) -> u32 {
    return this.vtable.GetOutputCount(this);
}

IDxcIncludeHandler_LoadSource :: inline (this: *IDxcIncludeHandler, pFilename: *s16, ppIncludeSource: **IDxcBlob) -> HRESULT {
    return this.vtable.LoadSource(this, pFilename, ppIncludeSource);
}

IDxcLibrary_CreateBlobFromBlob :: inline (this: *IDxcLibrary, pBlob: *IDxcBlob, offset: u32, length: u32, ppResult: **IDxcBlob) -> HRESULT {
    return this.vtable.CreateBlobFromBlob(this, pBlob, offset, length, ppResult);
}

IDxcLibrary_CreateBlobFromFile :: inline (this: *IDxcLibrary, pFileName: *s16, codePage: *u32, pBlobEncoding: **IDxcBlobEncoding) -> HRESULT {
    return this.vtable.CreateBlobFromFile(this, pFileName, codePage, pBlobEncoding);
}

IDxcLibrary_CreateBlobWithEncodingFromPinned :: inline (this: *IDxcLibrary, pText: *void, size: u32, codePage: u32, pBlobEncoding: **IDxcBlobEncoding) -> HRESULT {
    return this.vtable.CreateBlobWithEncodingFromPinned(this, pText, size, codePage, pBlobEncoding);
}

IDxcLibrary_CreateBlobWithEncodingOnHeapCopy :: inline (this: *IDxcLibrary, pText: *void, size: u32, codePage: u32, pBlobEncoding: **IDxcBlobEncoding) -> HRESULT {
    return this.vtable.CreateBlobWithEncodingOnHeapCopy(this, pText, size, codePage, pBlobEncoding);
}

IDxcLibrary_CreateBlobWithEncodingOnMalloc :: inline (this: *IDxcLibrary, pText: *void, pIMalloc: *IMalloc, size: u32, codePage: u32, pBlobEncoding: **IDxcBlobEncoding) -> HRESULT {
    return this.vtable.CreateBlobWithEncodingOnMalloc(this, pText, pIMalloc, size, codePage, pBlobEncoding);
}

IDxcLibrary_CreateIncludeHandler :: inline (this: *IDxcLibrary, ppResult: **IDxcIncludeHandler) -> HRESULT {
    return this.vtable.CreateIncludeHandler(this, ppResult);
}

IDxcLibrary_CreateStreamFromBlobReadOnly :: inline (this: *IDxcLibrary, pBlob: *IDxcBlob, ppStream: **IStream) -> HRESULT {
    return this.vtable.CreateStreamFromBlobReadOnly(this, pBlob, ppStream);
}

IDxcLibrary_GetBlobAsUtf8 :: inline (this: *IDxcLibrary, pBlob: *IDxcBlob, pBlobEncoding: **IDxcBlobEncoding) -> HRESULT {
    return this.vtable.GetBlobAsUtf8(this, pBlob, pBlobEncoding);
}

IDxcLibrary_GetBlobAsWide :: inline (this: *IDxcLibrary, pBlob: *IDxcBlob, pBlobEncoding: **IDxcBlobEncoding) -> HRESULT {
    return this.vtable.GetBlobAsWide(this, pBlob, pBlobEncoding);
}

IDxcLibrary_SetMalloc :: inline (this: *IDxcLibrary, pMalloc: *IMalloc) -> HRESULT {
    return this.vtable.SetMalloc(this, pMalloc);
}

IDxcLinker_Link :: inline (this: *IDxcLinker, pEntryName: *s16, pTargetProfile: *s16, pLibNames: **s16, libCount: u32, pArguments: **s16, argCount: u32, ppResult: **IDxcOperationResult) -> HRESULT {
    return this.vtable.Link(this, pEntryName, pTargetProfile, pLibNames, libCount, pArguments, argCount, ppResult);
}

IDxcLinker_RegisterLibrary :: inline (this: *IDxcLinker, pLibName: *s16, pLib: *IDxcBlob) -> HRESULT {
    return this.vtable.RegisterLibrary(this, pLibName, pLib);
}

IDxcOperationResult_GetErrorBuffer :: inline (this: *IDxcOperationResult, ppErrors: **IDxcBlobEncoding) -> HRESULT {
    return this.vtable.GetErrorBuffer(this, ppErrors);
}

IDxcOperationResult_GetResult :: inline (this: *IDxcOperationResult, ppResult: **IDxcBlob) -> HRESULT {
    return this.vtable.GetResult(this, ppResult);
}

IDxcOperationResult_GetStatus :: inline (this: *IDxcOperationResult, pStatus: *HRESULT) -> HRESULT {
    return this.vtable.GetStatus(this, pStatus);
}

IDxcOptimizer_GetAvailablePass :: inline (this: *IDxcOptimizer, index: u32, ppResult: **IDxcOptimizerPass) -> HRESULT {
    return this.vtable.GetAvailablePass(this, index, ppResult);
}

IDxcOptimizer_GetAvailablePassCount :: inline (this: *IDxcOptimizer, pCount: *u32) -> HRESULT {
    return this.vtable.GetAvailablePassCount(this, pCount);
}

IDxcOptimizer_RunOptimizer :: inline (this: *IDxcOptimizer, pBlob: *IDxcBlob, ppOptions: **s16, optionCount: u32, pOutputModule: **IDxcBlob, ppOutputText: **IDxcBlobEncoding) -> HRESULT {
    return this.vtable.RunOptimizer(this, pBlob, ppOptions, optionCount, pOutputModule, ppOutputText);
}

IDxcOptimizerPass_GetDescription :: inline (this: *IDxcOptimizerPass, ppResult: **s16) -> HRESULT {
    return this.vtable.GetDescription(this, ppResult);
}

IDxcOptimizerPass_GetOptionArgCount :: inline (this: *IDxcOptimizerPass, pCount: *u32) -> HRESULT {
    return this.vtable.GetOptionArgCount(this, pCount);
}

IDxcOptimizerPass_GetOptionArgDescription :: inline (this: *IDxcOptimizerPass, argIndex: u32, ppResult: **s16) -> HRESULT {
    return this.vtable.GetOptionArgDescription(this, argIndex, ppResult);
}

IDxcOptimizerPass_GetOptionArgName :: inline (this: *IDxcOptimizerPass, argIndex: u32, ppResult: **s16) -> HRESULT {
    return this.vtable.GetOptionArgName(this, argIndex, ppResult);
}

IDxcOptimizerPass_GetOptionName :: inline (this: *IDxcOptimizerPass, ppResult: **s16) -> HRESULT {
    return this.vtable.GetOptionName(this, ppResult);
}

IDxcPdbUtils2_GetArg :: inline (this: *IDxcPdbUtils2, uIndex: u32, ppResult: **IDxcBlobUtf16) -> HRESULT {
    return this.vtable.GetArg(this, uIndex, ppResult);
}

IDxcPdbUtils2_GetArgCount :: inline (this: *IDxcPdbUtils2, pCount: *u32) -> HRESULT {
    return this.vtable.GetArgCount(this, pCount);
}

IDxcPdbUtils2_GetArgPair :: inline (this: *IDxcPdbUtils2, uIndex: u32, ppName: **IDxcBlobUtf16, ppValue: **IDxcBlobUtf16) -> HRESULT {
    return this.vtable.GetArgPair(this, uIndex, ppName, ppValue);
}

IDxcPdbUtils2_GetArgPairCount :: inline (this: *IDxcPdbUtils2, pCount: *u32) -> HRESULT {
    return this.vtable.GetArgPairCount(this, pCount);
}

IDxcPdbUtils2_GetCustomToolchainData :: inline (this: *IDxcPdbUtils2, ppBlob: **IDxcBlob) -> HRESULT {
    return this.vtable.GetCustomToolchainData(this, ppBlob);
}

IDxcPdbUtils2_GetCustomToolchainID :: inline (this: *IDxcPdbUtils2, pID: *u32) -> HRESULT {
    return this.vtable.GetCustomToolchainID(this, pID);
}

IDxcPdbUtils2_GetDefine :: inline (this: *IDxcPdbUtils2, uIndex: u32, ppResult: **IDxcBlobUtf16) -> HRESULT {
    return this.vtable.GetDefine(this, uIndex, ppResult);
}

IDxcPdbUtils2_GetDefineCount :: inline (this: *IDxcPdbUtils2, pCount: *u32) -> HRESULT {
    return this.vtable.GetDefineCount(this, pCount);
}

IDxcPdbUtils2_GetEntryPoint :: inline (this: *IDxcPdbUtils2, ppResult: **IDxcBlobUtf16) -> HRESULT {
    return this.vtable.GetEntryPoint(this, ppResult);
}

IDxcPdbUtils2_GetFlag :: inline (this: *IDxcPdbUtils2, uIndex: u32, ppResult: **IDxcBlobUtf16) -> HRESULT {
    return this.vtable.GetFlag(this, uIndex, ppResult);
}

IDxcPdbUtils2_GetFlagCount :: inline (this: *IDxcPdbUtils2, pCount: *u32) -> HRESULT {
    return this.vtable.GetFlagCount(this, pCount);
}

IDxcPdbUtils2_GetHash :: inline (this: *IDxcPdbUtils2, ppResult: **IDxcBlob) -> HRESULT {
    return this.vtable.GetHash(this, ppResult);
}

IDxcPdbUtils2_GetLibraryPDB :: inline (this: *IDxcPdbUtils2, uIndex: u32, ppOutPdbUtils: **IDxcPdbUtils2, ppLibraryName: **IDxcBlobUtf16) -> HRESULT {
    return this.vtable.GetLibraryPDB(this, uIndex, ppOutPdbUtils, ppLibraryName);
}

IDxcPdbUtils2_GetLibraryPDBCount :: inline (this: *IDxcPdbUtils2, pCount: *u32) -> HRESULT {
    return this.vtable.GetLibraryPDBCount(this, pCount);
}

IDxcPdbUtils2_GetMainFileName :: inline (this: *IDxcPdbUtils2, ppResult: **IDxcBlobUtf16) -> HRESULT {
    return this.vtable.GetMainFileName(this, ppResult);
}

IDxcPdbUtils2_GetName :: inline (this: *IDxcPdbUtils2, ppResult: **IDxcBlobUtf16) -> HRESULT {
    return this.vtable.GetName(this, ppResult);
}

IDxcPdbUtils2_GetSource :: inline (this: *IDxcPdbUtils2, uIndex: u32, ppResult: **IDxcBlobEncoding) -> HRESULT {
    return this.vtable.GetSource(this, uIndex, ppResult);
}

IDxcPdbUtils2_GetSourceCount :: inline (this: *IDxcPdbUtils2, pCount: *u32) -> HRESULT {
    return this.vtable.GetSourceCount(this, pCount);
}

IDxcPdbUtils2_GetSourceName :: inline (this: *IDxcPdbUtils2, uIndex: u32, ppResult: **IDxcBlobUtf16) -> HRESULT {
    return this.vtable.GetSourceName(this, uIndex, ppResult);
}

IDxcPdbUtils2_GetTargetProfile :: inline (this: *IDxcPdbUtils2, ppResult: **IDxcBlobUtf16) -> HRESULT {
    return this.vtable.GetTargetProfile(this, ppResult);
}

IDxcPdbUtils2_GetVersionInfo :: inline (this: *IDxcPdbUtils2, ppVersionInfo: **IDxcVersionInfo) -> HRESULT {
    return this.vtable.GetVersionInfo(this, ppVersionInfo);
}

IDxcPdbUtils2_GetWholeDxil :: inline (this: *IDxcPdbUtils2, ppResult: **IDxcBlob) -> HRESULT {
    return this.vtable.GetWholeDxil(this, ppResult);
}

IDxcPdbUtils2_IsFullPDB :: inline (this: *IDxcPdbUtils2) -> BOOL {
    return this.vtable.IsFullPDB(this);
}

IDxcPdbUtils2_IsPDBRef :: inline (this: *IDxcPdbUtils2) -> BOOL {
    return this.vtable.IsPDBRef(this);
}

IDxcPdbUtils2_Load :: inline (this: *IDxcPdbUtils2, pPdbOrDxil: *IDxcBlob) -> HRESULT {
    return this.vtable.Load(this, pPdbOrDxil);
}

IDxcPdbUtils_CompileForFullPDB :: inline (this: *IDxcPdbUtils, ppResult: **IDxcResult) -> HRESULT {
    return this.vtable.CompileForFullPDB(this, ppResult);
}

IDxcPdbUtils_GetArg :: inline (this: *IDxcPdbUtils, uIndex: u32, pResult: *BSTR) -> HRESULT {
    return this.vtable.GetArg(this, uIndex, pResult);
}

IDxcPdbUtils_GetArgCount :: inline (this: *IDxcPdbUtils, pCount: *u32) -> HRESULT {
    return this.vtable.GetArgCount(this, pCount);
}

IDxcPdbUtils_GetArgPair :: inline (this: *IDxcPdbUtils, uIndex: u32, pName: *BSTR, pValue: *BSTR) -> HRESULT {
    return this.vtable.GetArgPair(this, uIndex, pName, pValue);
}

IDxcPdbUtils_GetArgPairCount :: inline (this: *IDxcPdbUtils, pCount: *u32) -> HRESULT {
    return this.vtable.GetArgPairCount(this, pCount);
}

IDxcPdbUtils_GetDefine :: inline (this: *IDxcPdbUtils, uIndex: u32, pResult: *BSTR) -> HRESULT {
    return this.vtable.GetDefine(this, uIndex, pResult);
}

IDxcPdbUtils_GetDefineCount :: inline (this: *IDxcPdbUtils, pCount: *u32) -> HRESULT {
    return this.vtable.GetDefineCount(this, pCount);
}

IDxcPdbUtils_GetEntryPoint :: inline (this: *IDxcPdbUtils, pResult: *BSTR) -> HRESULT {
    return this.vtable.GetEntryPoint(this, pResult);
}

IDxcPdbUtils_GetFlag :: inline (this: *IDxcPdbUtils, uIndex: u32, pResult: *BSTR) -> HRESULT {
    return this.vtable.GetFlag(this, uIndex, pResult);
}

IDxcPdbUtils_GetFlagCount :: inline (this: *IDxcPdbUtils, pCount: *u32) -> HRESULT {
    return this.vtable.GetFlagCount(this, pCount);
}

IDxcPdbUtils_GetFullPDB :: inline (this: *IDxcPdbUtils, ppFullPDB: **IDxcBlob) -> HRESULT {
    return this.vtable.GetFullPDB(this, ppFullPDB);
}

IDxcPdbUtils_GetHash :: inline (this: *IDxcPdbUtils, ppResult: **IDxcBlob) -> HRESULT {
    return this.vtable.GetHash(this, ppResult);
}

IDxcPdbUtils_GetMainFileName :: inline (this: *IDxcPdbUtils, pResult: *BSTR) -> HRESULT {
    return this.vtable.GetMainFileName(this, pResult);
}

IDxcPdbUtils_GetName :: inline (this: *IDxcPdbUtils, pResult: *BSTR) -> HRESULT {
    return this.vtable.GetName(this, pResult);
}

IDxcPdbUtils_GetSource :: inline (this: *IDxcPdbUtils, uIndex: u32, ppResult: **IDxcBlobEncoding) -> HRESULT {
    return this.vtable.GetSource(this, uIndex, ppResult);
}

IDxcPdbUtils_GetSourceCount :: inline (this: *IDxcPdbUtils, pCount: *u32) -> HRESULT {
    return this.vtable.GetSourceCount(this, pCount);
}

IDxcPdbUtils_GetSourceName :: inline (this: *IDxcPdbUtils, uIndex: u32, pResult: *BSTR) -> HRESULT {
    return this.vtable.GetSourceName(this, uIndex, pResult);
}

IDxcPdbUtils_GetTargetProfile :: inline (this: *IDxcPdbUtils, pResult: *BSTR) -> HRESULT {
    return this.vtable.GetTargetProfile(this, pResult);
}

IDxcPdbUtils_GetVersionInfo :: inline (this: *IDxcPdbUtils, ppVersionInfo: **IDxcVersionInfo) -> HRESULT {
    return this.vtable.GetVersionInfo(this, ppVersionInfo);
}

IDxcPdbUtils_IsFullPDB :: inline (this: *IDxcPdbUtils) -> BOOL {
    return this.vtable.IsFullPDB(this);
}

IDxcPdbUtils_Load :: inline (this: *IDxcPdbUtils, pPdbOrDxil: *IDxcBlob) -> HRESULT {
    return this.vtable.Load(this, pPdbOrDxil);
}

IDxcPdbUtils_OverrideArgs :: inline (this: *IDxcPdbUtils, pArgPairs: *DxcArgPair, uNumArgPairs: u32) -> HRESULT {
    return this.vtable.OverrideArgs(this, pArgPairs, uNumArgPairs);
}

IDxcPdbUtils_OverrideRootSignature :: inline (this: *IDxcPdbUtils, pRootSignature: *s16) -> HRESULT {
    return this.vtable.OverrideRootSignature(this, pRootSignature);
}

IDxcPdbUtils_SetCompiler :: inline (this: *IDxcPdbUtils, pCompiler: *IDxcCompiler3) -> HRESULT {
    return this.vtable.SetCompiler(this, pCompiler);
}

IDxcResult_GetNumOutputs :: inline (this: *IDxcResult) -> u32 {
    return this.vtable.GetNumOutputs(this);
}

IDxcResult_GetOutput :: inline (this: *IDxcResult, dxcOutKind: DXC_OUT_KIND, iid: *IID, ppvObject: **void, ppOutputName: **IDxcBlobUtf16) -> HRESULT {
    return this.vtable.GetOutput(this, dxcOutKind, iid, ppvObject, ppOutputName);
}

IDxcResult_GetOutputByIndex :: inline (this: *IDxcResult, Index: u32) -> DXC_OUT_KIND {
    return this.vtable.GetOutputByIndex(this, Index);
}

IDxcResult_HasOutput :: inline (this: *IDxcResult, dxcOutKind: DXC_OUT_KIND) -> BOOL {
    return this.vtable.HasOutput(this, dxcOutKind);
}

IDxcResult_PrimaryOutput :: inline (this: *IDxcResult) -> DXC_OUT_KIND {
    return this.vtable.PrimaryOutput(this);
}

IDxcUtils_BuildArguments :: inline (this: *IDxcUtils, pSourceName: *s16, pEntryPoint: *s16, pTargetProfile: *s16, pArguments: **s16, argCount: u32, pDefines: *DxcDefine, defineCount: u32, ppArgs: **IDxcCompilerArgs) -> HRESULT {
    return this.vtable.BuildArguments(this, pSourceName, pEntryPoint, pTargetProfile, pArguments, argCount, pDefines, defineCount, ppArgs);
}

IDxcUtils_CreateBlob :: inline (this: *IDxcUtils, pData: *void, size: u32, codePage: u32, ppBlobEncoding: **IDxcBlobEncoding) -> HRESULT {
    return this.vtable.CreateBlob(this, pData, size, codePage, ppBlobEncoding);
}

IDxcUtils_CreateBlobFromBlob :: inline (this: *IDxcUtils, pBlob: *IDxcBlob, offset: u32, length: u32, ppResult: **IDxcBlob) -> HRESULT {
    return this.vtable.CreateBlobFromBlob(this, pBlob, offset, length, ppResult);
}

IDxcUtils_CreateBlobFromPinned :: inline (this: *IDxcUtils, pData: *void, size: u32, codePage: u32, ppBlobEncoding: **IDxcBlobEncoding) -> HRESULT {
    return this.vtable.CreateBlobFromPinned(this, pData, size, codePage, ppBlobEncoding);
}

IDxcUtils_CreateDefaultIncludeHandler :: inline (this: *IDxcUtils, ppResult: **IDxcIncludeHandler) -> HRESULT {
    return this.vtable.CreateDefaultIncludeHandler(this, ppResult);
}

IDxcUtils_CreateReadOnlyStreamFromBlob :: inline (this: *IDxcUtils, pBlob: *IDxcBlob, ppStream: **IStream) -> HRESULT {
    return this.vtable.CreateReadOnlyStreamFromBlob(this, pBlob, ppStream);
}

IDxcUtils_CreateReflection :: inline (this: *IDxcUtils, pData: *DxcText, iid: *IID, ppvReflection: **void) -> HRESULT {
    return this.vtable.CreateReflection(this, pData, iid, ppvReflection);
}

IDxcUtils_GetBlobAsUtf8 :: inline (this: *IDxcUtils, pBlob: *IDxcBlob, ppBlobEncoding: **IDxcBlobUtf8) -> HRESULT {
    return this.vtable.GetBlobAsUtf8(this, pBlob, ppBlobEncoding);
}

IDxcUtils_GetBlobAsWide :: inline (this: *IDxcUtils, pBlob: *IDxcBlob, ppBlobEncoding: **IDxcBlobUtf16) -> HRESULT {
    return this.vtable.GetBlobAsWide(this, pBlob, ppBlobEncoding);
}

IDxcUtils_GetDxilContainerPart :: inline (this: *IDxcUtils, pShader: *DxcText, DxcPart: u32, ppPartData: **void, pPartSizeInBytes: *u32) -> HRESULT {
    return this.vtable.GetDxilContainerPart(this, pShader, DxcPart, ppPartData, pPartSizeInBytes);
}

IDxcUtils_GetPDBContents :: inline (this: *IDxcUtils, pPDBBlob: *IDxcBlob, ppHash: **IDxcBlob, ppContainer: **IDxcBlob) -> HRESULT {
    return this.vtable.GetPDBContents(this, pPDBBlob, ppHash, ppContainer);
}

IDxcUtils_LoadFile :: inline (this: *IDxcUtils, pFileName: *s16, pCodePage: *u32, ppBlobEncoding: **IDxcBlobEncoding) -> HRESULT {
    return this.vtable.LoadFile(this, pFileName, pCodePage, ppBlobEncoding);
}

IDxcUtils_MoveToBlob :: inline (this: *IDxcUtils, pData: *void, pIMalloc: *IMalloc, size: u32, codePage: u32, ppBlobEncoding: **IDxcBlobEncoding) -> HRESULT {
    return this.vtable.MoveToBlob(this, pData, pIMalloc, size, codePage, ppBlobEncoding);
}

IDxcValidator2_ValidateWithDebug :: inline (this: *IDxcValidator2, pShader: *IDxcBlob, Flags: u32, pOptDebugBitcode: *DxcText, ppResult: **IDxcOperationResult) -> HRESULT {
    return this.vtable.ValidateWithDebug(this, pShader, Flags, pOptDebugBitcode, ppResult);
}

IDxcValidator_Validate :: inline (this: *IDxcValidator, pShader: *IDxcBlob, Flags: u32, ppResult: **IDxcOperationResult) -> HRESULT {
    return this.vtable.Validate(this, pShader, Flags, ppResult);
}

IDxcVersionInfo2_GetCommitInfo :: inline (this: *IDxcVersionInfo2, pCommitCount: *u32, pCommitHash: **u8) -> HRESULT {
    return this.vtable.GetCommitInfo(this, pCommitCount, pCommitHash);
}

IDxcVersionInfo3_GetCustomVersionString :: inline (this: *IDxcVersionInfo3, pVersionString: **u8) -> HRESULT {
    return this.vtable.GetCustomVersionString(this, pVersionString);
}

IDxcVersionInfo_GetFlags :: inline (this: *IDxcVersionInfo, pFlags: *u32) -> HRESULT {
    return this.vtable.GetFlags(this, pFlags);
}

IDxcVersionInfo_GetVersion :: inline (this: *IDxcVersionInfo, pMajor: *u32, pMinor: *u32) -> HRESULT {
    return this.vtable.GetVersion(this, pMajor, pMinor);
}

vtable :: (obj: *IDxcBlob) -> *IDxcBlob_VTable {
    return obj.vtable;
}

vtable :: (obj: *IDxcBlobEncoding) -> *IDxcBlobEncoding_VTable {
    return obj.vtable;
}

vtable :: (obj: *IDxcBlobUtf8) -> *IDxcBlobUtf8_VTable {
    return obj.vtable;
}

vtable :: (obj: *IDxcBlobUtf16) -> *IDxcBlobWide_VTable {
    return obj.vtable;
}

vtable :: (obj: *IDxcIncludeHandler) -> *IDxcIncludeHandler_VTable {
    return obj.vtable;
}

vtable :: (obj: *IDxcCompilerArgs) -> *IDxcCompilerArgs_VTable {
    return obj.vtable;
}

vtable :: (obj: *IDxcLibrary) -> *IDxcLibrary_VTable {
    return obj.vtable;
}

vtable :: (obj: *IDxcOperationResult) -> *IDxcOperationResult_VTable {
    return obj.vtable;
}

vtable :: (obj: *IDxcCompiler) -> *IDxcCompiler_VTable {
    return obj.vtable;
}

vtable :: (obj: *IDxcCompiler2) -> *IDxcCompiler2_VTable {
    return obj.vtable;
}

vtable :: (obj: *IDxcLinker) -> *IDxcLinker_VTable {
    return obj.vtable;
}

vtable :: (obj: *IDxcUtils) -> *IDxcUtils_VTable {
    return obj.vtable;
}

vtable :: (obj: *IDxcResult) -> *IDxcResult_VTable {
    return obj.vtable;
}

vtable :: (obj: *IDxcExtraOutputs) -> *IDxcExtraOutputs_VTable {
    return obj.vtable;
}

vtable :: (obj: *IDxcCompiler3) -> *IDxcCompiler3_VTable {
    return obj.vtable;
}

vtable :: (obj: *IDxcValidator) -> *IDxcValidator_VTable {
    return obj.vtable;
}

vtable :: (obj: *IDxcValidator2) -> *IDxcValidator2_VTable {
    return obj.vtable;
}

vtable :: (obj: *IDxcContainerBuilder) -> *IDxcContainerBuilder_VTable {
    return obj.vtable;
}

vtable :: (obj: *IDxcAssembler) -> *IDxcAssembler_VTable {
    return obj.vtable;
}

vtable :: (obj: *IDxcContainerReflection) -> *IDxcContainerReflection_VTable {
    return obj.vtable;
}

vtable :: (obj: *IDxcOptimizerPass) -> *IDxcOptimizerPass_VTable {
    return obj.vtable;
}

vtable :: (obj: *IDxcOptimizer) -> *IDxcOptimizer_VTable {
    return obj.vtable;
}

vtable :: (obj: *IDxcVersionInfo) -> *IDxcVersionInfo_VTable {
    return obj.vtable;
}

vtable :: (obj: *IDxcVersionInfo2) -> *IDxcVersionInfo2_VTable {
    return obj.vtable;
}

vtable :: (obj: *IDxcVersionInfo3) -> *IDxcVersionInfo3_VTable {
    return obj.vtable;
}

vtable :: (obj: *IDxcPdbUtils) -> *IDxcPdbUtils_VTable {
    return obj.vtable;
}

vtable :: (obj: *IDxcPdbUtils2) -> *IDxcPdbUtils2_VTable {
    return obj.vtable;
}

// Generated from docs/specs/stdlib/modules/d3d-compiler.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

#scope_file
#import "Windows";

d3dcompiler :: #library,system "d3dcompiler";
#scope_export

// Types and layouts

D3D_BLOB_PART :: enum s32 {
    INPUT_SIGNATURE_BLOB :: 0;
    OUTPUT_SIGNATURE_BLOB :: 1;
    INPUT_AND_OUTPUT_SIGNATURE_BLOB :: 2;
    PATCH_CONSTANT_SIGNATURE_BLOB :: 3;
    ALL_SIGNATURE_BLOB :: 4;
    DEBUG_INFO :: 5;
    LEGACY_SHADER :: 6;
    XNA_PREPASS_SHADER :: 7;
    XNA_SHADER :: 8;
    PDB :: 9;
    PRIVATE_DATA :: 10;
    ROOT_SIGNATURE :: 11;
    DEBUG_NAME :: 12;
    TEST_ALTERNATE_SHADER :: 32768;
    TEST_COMPILE_DETAILS :: 32769;
    TEST_COMPILE_PERF :: 32770;
    TEST_COMPILE_REPORT :: 32771;
    D3D_BLOB_INPUT_SIGNATURE_BLOB :: INPUT_SIGNATURE_BLOB;
    D3D_BLOB_OUTPUT_SIGNATURE_BLOB :: OUTPUT_SIGNATURE_BLOB;
    D3D_BLOB_INPUT_AND_OUTPUT_SIGNATURE_BLOB :: INPUT_AND_OUTPUT_SIGNATURE_BLOB;
    D3D_BLOB_PATCH_CONSTANT_SIGNATURE_BLOB :: PATCH_CONSTANT_SIGNATURE_BLOB;
    D3D_BLOB_ALL_SIGNATURE_BLOB :: ALL_SIGNATURE_BLOB;
    D3D_BLOB_DEBUG_INFO :: DEBUG_INFO;
    D3D_BLOB_LEGACY_SHADER :: LEGACY_SHADER;
    D3D_BLOB_XNA_PREPASS_SHADER :: XNA_PREPASS_SHADER;
    D3D_BLOB_XNA_SHADER :: XNA_SHADER;
    D3D_BLOB_PDB :: PDB;
    D3D_BLOB_PRIVATE_DATA :: PRIVATE_DATA;
    D3D_BLOB_ROOT_SIGNATURE :: ROOT_SIGNATURE;
    D3D_BLOB_DEBUG_NAME :: DEBUG_NAME;
    D3D_BLOB_TEST_ALTERNATE_SHADER :: TEST_ALTERNATE_SHADER;
    D3D_BLOB_TEST_COMPILE_DETAILS :: TEST_COMPILE_DETAILS;
    D3D_BLOB_TEST_COMPILE_PERF :: TEST_COMPILE_PERF;
    D3D_BLOB_TEST_COMPILE_REPORT :: TEST_COMPILE_REPORT;
}

D3D_INCLUDE_TYPE :: enum s32 {
    INCLUDE_LOCAL :: 0;
    INCLUDE_SYSTEM :: 1;
    _10_INCLUDE_LOCAL :: 0;
    _10_INCLUDE_SYSTEM :: 1;
    INCLUDE_FORCE_DWORD :: 2147483647;
    D3D_INCLUDE_LOCAL :: INCLUDE_LOCAL;
    D3D_INCLUDE_SYSTEM :: INCLUDE_SYSTEM;
    D3D10_INCLUDE_LOCAL :: _10_INCLUDE_LOCAL;
    D3D10_INCLUDE_SYSTEM :: _10_INCLUDE_SYSTEM;
    D3D_INCLUDE_FORCE_DWORD :: INCLUDE_FORCE_DWORD;
}

D3D_SHADER_DATA :: struct {
    pBytecode: *void;
    BytecodeLength: SIZE_T;
}

D3D_SHADER_MACRO :: struct {
    Name: *u8;
    Definition: *u8;
}

D3DCOMPILE :: enum_flags {
    DEBUG :: 1 << 0;
    SKIP_VALIDATION :: 1 << 1;
    SKIP_OPTIMIZATION :: 1 << 2;
    PACK_MATRIX_ROW_MAJOR :: 1 << 3;
    PACK_MATRIX_COLUMN_MAJOR :: 1 << 4;
    PARTIAL_PRECISION :: 1 << 5;
    FORCE_VS_SOFTWARE_NO_OPT :: 1 << 6;
    FORCE_PS_SOFTWARE_NO_OPT :: 1 << 7;
    NO_PRESHADER :: 1 << 8;
    AVOID_FLOW_CONTROL :: 1 << 9;
    PREFER_FLOW_CONTROL :: 1 << 10;
    ENABLE_STRICTNESS :: 1 << 11;
    ENABLE_BACKWARDS_COMPATIBILITY :: 1 << 12;
    IEEE_STRICTNESS :: 1 << 13;
    OPTIMIZATION_LEVEL0 :: 1 << 14;
    OPTIMIZATION_LEVEL1 :: 0;
    OPTIMIZATION_LEVEL2 :: (1 << 14) | (1 << 15);
    OPTIMIZATION_LEVEL3 :: 1 << 15;
    RESERVED16 :: 1 << 16;
    RESERVED17 :: 1 << 17;
    WARNINGS_ARE_ERRORS :: 1 << 18;
    RESOURCES_MAY_ALIAS :: 1 << 19;
    ENABLE_UNBOUNDED_DESCRIPTOR_TABLES :: 1 << 20;
    ALL_RESOURCES_BOUND :: 1 << 21;
    DEBUG_NAME_FOR_SOURCE :: 1 << 22;
    DEBUG_NAME_FOR_BINARY :: 1 << 23;
    D3DCOMPILE_DEBUG :: DEBUG;
    D3DCOMPILE_SKIP_VALIDATION :: SKIP_VALIDATION;
    D3DCOMPILE_SKIP_OPTIMIZATION :: SKIP_OPTIMIZATION;
    D3DCOMPILE_PACK_MATRIX_ROW_MAJOR :: PACK_MATRIX_ROW_MAJOR;
    D3DCOMPILE_PACK_MATRIX_COLUMN_MAJOR :: PACK_MATRIX_COLUMN_MAJOR;
    D3DCOMPILE_PARTIAL_PRECISION :: PARTIAL_PRECISION;
    D3DCOMPILE_FORCE_VS_SOFTWARE_NO_OPT :: FORCE_VS_SOFTWARE_NO_OPT;
    D3DCOMPILE_FORCE_PS_SOFTWARE_NO_OPT :: FORCE_PS_SOFTWARE_NO_OPT;
    D3DCOMPILE_NO_PRESHADER :: NO_PRESHADER;
    D3DCOMPILE_AVOID_FLOW_CONTROL :: AVOID_FLOW_CONTROL;
    D3DCOMPILE_PREFER_FLOW_CONTROL :: PREFER_FLOW_CONTROL;
    D3DCOMPILE_ENABLE_STRICTNESS :: ENABLE_STRICTNESS;
    D3DCOMPILE_ENABLE_BACKWARDS_COMPATIBILITY :: ENABLE_BACKWARDS_COMPATIBILITY;
    D3DCOMPILE_IEEE_STRICTNESS :: IEEE_STRICTNESS;
    D3DCOMPILE_OPTIMIZATION_LEVEL0 :: OPTIMIZATION_LEVEL0;
    D3DCOMPILE_OPTIMIZATION_LEVEL1 :: OPTIMIZATION_LEVEL1;
    D3DCOMPILE_OPTIMIZATION_LEVEL2 :: OPTIMIZATION_LEVEL2;
    D3DCOMPILE_OPTIMIZATION_LEVEL3 :: OPTIMIZATION_LEVEL3;
    D3DCOMPILE_RESERVED16 :: RESERVED16;
    D3DCOMPILE_RESERVED17 :: RESERVED17;
    D3DCOMPILE_WARNINGS_ARE_ERRORS :: WARNINGS_ARE_ERRORS;
    D3DCOMPILE_RESOURCES_MAY_ALIAS :: RESOURCES_MAY_ALIAS;
    D3DCOMPILE_ENABLE_UNBOUNDED_DESCRIPTOR_TABLES :: ENABLE_UNBOUNDED_DESCRIPTOR_TABLES;
    D3DCOMPILE_ALL_RESOURCES_BOUND :: ALL_RESOURCES_BOUND;
    D3DCOMPILE_DEBUG_NAME_FOR_SOURCE :: DEBUG_NAME_FOR_SOURCE;
    D3DCOMPILE_DEBUG_NAME_FOR_BINARY :: DEBUG_NAME_FOR_BINARY;
}

D3DCOMPILE_EFFECT :: enum_flags {
    CHILD_EFFECT :: 1 << 0;
    ALLOW_SLOW_OPS :: 1 << 1;
    D3DCOMPILE_EFFECT_CHILD_EFFECT :: CHILD_EFFECT;
    D3DCOMPILE_EFFECT_ALLOW_SLOW_OPS :: ALLOW_SLOW_OPS;
}

D3DCOMPILE_FLAGS2 :: enum_flags {
    FORCE_ROOT_SIGNATURE_LATEST :: 0;
    FORCE_ROOT_SIGNATURE_1_0 :: 1 << 4;
    FORCE_ROOT_SIGNATURE_1_1 :: 1 << 5;
    D3DCOMPILE_FLAGS2_FORCE_ROOT_SIGNATURE_LATEST :: FORCE_ROOT_SIGNATURE_LATEST;
    D3DCOMPILE_FLAGS2_FORCE_ROOT_SIGNATURE_1_0 :: FORCE_ROOT_SIGNATURE_1_0;
    D3DCOMPILE_FLAGS2_FORCE_ROOT_SIGNATURE_1_1 :: FORCE_ROOT_SIGNATURE_1_1;
}

D3DCOMPILE_SECDATA :: enum_flags {
    MERGE_UAV_SLOTS :: 0x00000001;
    PRESERVE_TEMPLATE_SLOTS :: 0x00000002;
    REQUIRE_TEMPLATE_MATCH :: 0x00000004;
    D3DCOMPILE_SECDATA_MERGE_UAV_SLOTS :: MERGE_UAV_SLOTS;
    D3DCOMPILE_SECDATA_PRESERVE_TEMPLATE_SLOTS :: PRESERVE_TEMPLATE_SLOTS;
    D3DCOMPILE_SECDATA_REQUIRE_TEMPLATE_MATCH :: REQUIRE_TEMPLATE_MATCH;
}

D3DCOMPILER_STRIP_FLAGS :: enum_flags u32 {
    REFLECTION_DATA :: 0x1;
    DEBUG_INFO :: 0x2;
    TEST_BLOBS :: 0x4;
    PRIVATE_DATA :: 0x8;
    ROOT_SIGNATURE :: 0x10;
    FORCE_DWORD :: 0x7fffffff;
    D3DCOMPILER_STRIP_REFLECTION_DATA :: REFLECTION_DATA;
    D3DCOMPILER_STRIP_DEBUG_INFO :: DEBUG_INFO;
    D3DCOMPILER_STRIP_TEST_BLOBS :: TEST_BLOBS;
    D3DCOMPILER_STRIP_PRIVATE_DATA :: PRIVATE_DATA;
    D3DCOMPILER_STRIP_ROOT_SIGNATURE :: ROOT_SIGNATURE;
    D3DCOMPILER_STRIP_FORCE_DWORD :: FORCE_DWORD;
}

ID3D10Blob :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *ID3D10Blob_VTable;
}

ID3D10Blob_VTable :: struct #type_info_none {
    using vtable: IUnknown_VTable;
    GetBufferPointer: #type (this: *ID3D10Blob) -> *void #cpp_method;
    GetBufferSize: #type (this: *ID3D10Blob) -> SIZE_T #cpp_method;
}

ID3D10Effect :: struct {}

ID3D11Linker :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *ID3D11Linker_VTable;
}

ID3D11Linker_VTable :: struct #type_info_none {
    using vtable: IUnknown_VTable;
    Link: #type (this: *ID3D11Linker, pEntry: *ID3D11ModuleInstance, pEntryName: *u8, pTargetName: *u8, uFlags: u32, ppShaderBlob: **ID3DBlob, ppErrorBuffer: **ID3DBlob) -> HRESULT #cpp_method;
    UseLibrary: #type (this: *ID3D11Linker, pLibraryMI: *ID3D11ModuleInstance) -> HRESULT #cpp_method;
    AddClipPlaneFromCBuffer: #type (this: *ID3D11Linker, uCBufferSlot: u32, uCBufferEntry: u32) -> HRESULT #cpp_method;
}

ID3D11Module :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *ID3D11Module_VTable;
}

ID3D11Module_VTable :: struct #type_info_none {
    using vtable: IUnknown_VTable;
    CreateInstance: #type (this: *ID3D11Module, pNamespace: *u8, ppModuleInstance: **ID3D11ModuleInstance) -> HRESULT #cpp_method;
}

ID3D11ModuleInstance :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *ID3D11ModuleInstance_VTable;
}

ID3D11ModuleInstance_VTable :: struct #type_info_none {
    using vtable: IUnknown_VTable;
    BindConstantBuffer: #type (this: *ID3D11ModuleInstance, uSrcSlot: u32, uDstSlot: u32, cbDstOffset: u32) -> HRESULT #cpp_method;
    BindConstantBufferByName: #type (this: *ID3D11ModuleInstance, pName: *u8, uDstSlot: u32, cbDstOffset: u32) -> HRESULT #cpp_method;
    BindResource: #type (this: *ID3D11ModuleInstance, uSrcSlot: u32, uDstSlot: u32, uCount: u32) -> HRESULT #cpp_method;
    BindResourceByName: #type (this: *ID3D11ModuleInstance, pName: *u8, uDstSlot: u32, uCount: u32) -> HRESULT #cpp_method;
    BindSampler: #type (this: *ID3D11ModuleInstance, uSrcSlot: u32, uDstSlot: u32, uCount: u32) -> HRESULT #cpp_method;
    BindSamplerByName: #type (this: *ID3D11ModuleInstance, pName: *u8, uDstSlot: u32, uCount: u32) -> HRESULT #cpp_method;
    BindUnorderedAccessView: #type (this: *ID3D11ModuleInstance, uSrcSlot: u32, uDstSlot: u32, uCount: u32) -> HRESULT #cpp_method;
    BindUnorderedAccessViewByName: #type (this: *ID3D11ModuleInstance, pName: *u8, uDstSlot: u32, uCount: u32) -> HRESULT #cpp_method;
    BindResourceAsUnorderedAccessView: #type (this: *ID3D11ModuleInstance, uSrcSrvSlot: u32, uDstUavSlot: u32, uCount: u32) -> HRESULT #cpp_method;
    BindResourceAsUnorderedAccessViewByName: #type (this: *ID3D11ModuleInstance, pSrvName: *u8, uDstUavSlot: u32, uCount: u32) -> HRESULT #cpp_method;
}

ID3DInclude :: struct {
    vtable: *ID3DInclude_VTable;
}

ID3DInclude_VTable :: struct #type_info_none {
    Open: #type (this: *ID3DInclude, IncludeType: D3D_INCLUDE_TYPE, pFileName: *u8, pParentData: *void, ppData: **void, pBytes: *u32) -> HRESULT #cpp_method;
    Close: #type (this: *ID3DInclude, pData: *void) -> HRESULT #cpp_method;
}

// Callback and opaque types

pD3DCompile :: #type (pSrcData: *void, SrcDataSize: SIZE_T, pFileName: *u8, pDefines: *D3D_SHADER_MACRO, pInclude: *ID3DInclude, pEntrypoint: *u8, pTarget: *u8, Flags1: u32, Flags2: u32, ppCode: **ID3DBlob, ppErrorMsgs: **ID3DBlob) -> HRESULT #c_call;

pD3DDisassemble :: #type (pSrcData: *void, SrcDataSize: SIZE_T, Flags: u32, szComments: *u8, ppDisassembly: **ID3DBlob) -> HRESULT #c_call;

pD3DPreprocess :: #type (pSrcData: *void, SrcDataSize: SIZE_T, pFileName: *u8, pDefines: *D3D_SHADER_MACRO, pInclude: *ID3DInclude, ppCodeText: **ID3DBlob, ppErrorMsgs: **ID3DBlob) -> HRESULT #c_call;

// Aliases and constants

D3D_COMPILE_STANDARD_FILE_INCLUDE :: cast(*ID3DInclude) cast(u64) 1;

D3D_COMPILER_VERSION :: 47;

D3D_COMPRESS_SHADER_KEEP_ALL_PARTS :: 0x00000001;

D3D_DISASM_DISABLE_DEBUG_INFO :: 0x00000010;

D3D_DISASM_ENABLE_COLOR_CODE :: 0x00000001;

D3D_DISASM_ENABLE_DEFAULT_VALUE_PRINTS :: 0x00000002;

D3D_DISASM_ENABLE_INSTRUCTION_CYCLE :: 0x00000008;

D3D_DISASM_ENABLE_INSTRUCTION_NUMBERING :: 0x00000004;

D3D_DISASM_ENABLE_INSTRUCTION_OFFSET :: 0x00000020;

D3D_DISASM_INSTRUCTION_ONLY :: 0x00000040;

D3D_DISASM_PRINT_HEX_LITERALS :: 0x00000080;

D3D_GET_INST_OFFSETS_INCLUDE_NON_EXECUTABLE :: 0x00000001;

D3DCOMPILER_DLL :: D3DCOMPILER_DLL_A;

D3DCOMPILER_DLL_A :: "d3dcompiler_47.dll";

ID3DBlob :: ID3D10Blob;

// Native entry points

D3DCompile :: (pSrcData: *void, SrcDataSize: SIZE_T, pSourceName: *u8, pDefines: *D3D_SHADER_MACRO, pInclude: *ID3DInclude, pEntrypoint: *u8, pTarget: *u8, Flags1: u32, Flags2: u32, ppCode: **ID3DBlob, ppErrorMsgs: **ID3DBlob) -> HRESULT #foreign d3dcompiler;

D3DCompile2 :: (pSrcData: *void, SrcDataSize: SIZE_T, pSourceName: *u8, pDefines: *D3D_SHADER_MACRO, pInclude: *ID3DInclude, pEntrypoint: *u8, pTarget: *u8, Flags1: u32, Flags2: u32, SecondaryDataFlags: u32, pSecondaryData: *void, SecondaryDataSize: SIZE_T, ppCode: **ID3DBlob, ppErrorMsgs: **ID3DBlob) -> HRESULT #foreign d3dcompiler;

D3DCompileFromFile :: (pFileName: *s16, pDefines: *D3D_SHADER_MACRO, pInclude: *ID3DInclude, pEntrypoint: *u8, pTarget: *u8, Flags1: u32, Flags2: u32, ppCode: **ID3DBlob, ppErrorMsgs: **ID3DBlob) -> HRESULT #foreign d3dcompiler;

D3DCompressShaders :: (uNumShaders: u32, pShaderData: *D3D_SHADER_DATA, uFlags: u32, ppCompressedData: **ID3DBlob) -> HRESULT #foreign d3dcompiler;

D3DCreateBlob :: (Size: SIZE_T, ppBlob: **ID3DBlob) -> HRESULT #foreign d3dcompiler;

D3DCreateLinker :: (ppLinker: **ID3D11Linker) -> HRESULT #foreign d3dcompiler;

D3DDecompressShaders :: (pSrcData: *void, SrcDataSize: SIZE_T, uNumShaders: u32, uStartIndex: u32, pIndices: *u32, uFlags: u32, ppShaders: **ID3DBlob, pTotalShaders: *u32) -> HRESULT #foreign d3dcompiler;

D3DDisassemble :: (pSrcData: *void, SrcDataSize: SIZE_T, Flags: u32, szComments: *u8, ppDisassembly: **ID3DBlob) -> HRESULT #foreign d3dcompiler;

D3DDisassemble10Effect :: (pEffect: *ID3D10Effect, Flags: u32, ppDisassembly: **ID3DBlob) -> HRESULT #foreign d3dcompiler;

D3DDisassembleRegion :: (pSrcData: *void, SrcDataSize: SIZE_T, Flags: u32, szComments: *u8, StartByteOffset: SIZE_T, NumInsts: SIZE_T, pFinishByteOffset: *SIZE_T, ppDisassembly: **ID3DBlob) -> HRESULT #foreign d3dcompiler;

D3DGetBlobPart :: (pSrcData: *void, SrcDataSize: SIZE_T, Part: D3D_BLOB_PART, Flags: u32, ppPart: **ID3DBlob) -> HRESULT #foreign d3dcompiler;

D3DGetDebugInfo :: (pSrcData: *void, SrcDataSize: SIZE_T, ppDebugInfo: **ID3DBlob) -> HRESULT #foreign d3dcompiler;

D3DGetInputAndOutputSignatureBlob :: (pSrcData: *void, SrcDataSize: SIZE_T, ppSignatureBlob: **ID3DBlob) -> HRESULT #foreign d3dcompiler;

D3DGetInputSignatureBlob :: (pSrcData: *void, SrcDataSize: SIZE_T, ppSignatureBlob: **ID3DBlob) -> HRESULT #foreign d3dcompiler;

D3DGetOutputSignatureBlob :: (pSrcData: *void, SrcDataSize: SIZE_T, ppSignatureBlob: **ID3DBlob) -> HRESULT #foreign d3dcompiler;

D3DGetTraceInstructionOffsets :: (pSrcData: *void, SrcDataSize: SIZE_T, Flags: u32, StartInstIndex: SIZE_T, NumInsts: SIZE_T, pOffsets: *SIZE_T, pTotalInsts: *SIZE_T) -> HRESULT #foreign d3dcompiler;

D3DLoadModule :: (pSrcData: *void, cbSrcDataSize: SIZE_T, ppModule: **ID3D11Module) -> HRESULT #foreign d3dcompiler;

D3DPreprocess :: (pSrcData: *void, SrcDataSize: SIZE_T, pSourceName: *u8, pDefines: *D3D_SHADER_MACRO, pInclude: *ID3DInclude, ppCodeText: **ID3DBlob, ppErrorMsgs: **ID3DBlob) -> HRESULT #foreign d3dcompiler;

D3DReadFileToBlob :: (pFileName: *s16, ppContents: **ID3DBlob) -> HRESULT #foreign d3dcompiler;

D3DReflect :: (pSrcData: *void, SrcDataSize: SIZE_T, pInterface: *IID, ppReflector: **void) -> HRESULT #foreign d3dcompiler;

D3DReflectLibrary :: (pSrcData: *void, SrcDataSize: SIZE_T, riid: *IID, ppReflector: **void) -> HRESULT #foreign d3dcompiler;

D3DSetBlobPart :: (pSrcData: *void, SrcDataSize: SIZE_T, Part: D3D_BLOB_PART, Flags: u32, pPart: *void, PartSize: SIZE_T, ppNewShader: **ID3DBlob) -> HRESULT #foreign d3dcompiler;

D3DStripShader :: (pShaderBytecode: *void, BytecodeLength: SIZE_T, uStripFlags: u32, ppStrippedBlob: **ID3DBlob) -> HRESULT #foreign d3dcompiler;

D3DWriteBlobToFile :: (pBlob: *ID3DBlob, pFileName: *s16, bOverwrite: BOOL) -> HRESULT #foreign d3dcompiler;

// Convenience operations

D3DReflect :: (pSrcData: *void, SrcDataSize: SIZE_T, pInterface: IID, ppReflector: **void) -> HRESULT #no_context {
    return D3DReflect(pSrcData, SrcDataSize, *pInterface, ppReflector);
}

D3DReflectLibrary :: (pSrcData: *void, SrcDataSize: SIZE_T, riid: IID, ppReflector: **void) -> HRESULT #no_context {
    return D3DReflectLibrary(pSrcData, SrcDataSize, *riid, ppReflector);
}

ID3D10Blob_GetBufferPointer :: inline (this: *ID3DBlob) -> *void {
    return this.vtable.GetBufferPointer(this);
}

ID3D10Blob_GetBufferSize :: inline (this: *ID3DBlob) -> SIZE_T {
    return this.vtable.GetBufferSize(this);
}

ID3D11Linker_AddClipPlaneFromCBuffer :: inline (this: *ID3D11Linker, uCBufferSlot: u32, uCBufferEntry: u32) -> HRESULT {
    return this.vtable.AddClipPlaneFromCBuffer(this, uCBufferSlot, uCBufferEntry);
}

ID3D11Linker_Link :: inline (this: *ID3D11Linker, pEntry: *ID3D11ModuleInstance, pEntryName: *u8, pTargetName: *u8, uFlags: u32, ppShaderBlob: **ID3DBlob, ppErrorBuffer: **ID3DBlob) -> HRESULT {
    return this.vtable.Link(this, pEntry, pEntryName, pTargetName, uFlags, ppShaderBlob, ppErrorBuffer);
}

ID3D11Linker_UseLibrary :: inline (this: *ID3D11Linker, pLibraryMI: *ID3D11ModuleInstance) -> HRESULT {
    return this.vtable.UseLibrary(this, pLibraryMI);
}

ID3D11Module_CreateInstance :: inline (this: *ID3D11Module, pNamespace: *u8, ppModuleInstance: **ID3D11ModuleInstance) -> HRESULT {
    return this.vtable.CreateInstance(this, pNamespace, ppModuleInstance);
}

ID3D11ModuleInstance_BindConstantBuffer :: inline (this: *ID3D11ModuleInstance, uSrcSlot: u32, uDstSlot: u32, cbDstOffset: u32) -> HRESULT {
    return this.vtable.BindConstantBuffer(this, uSrcSlot, uDstSlot, cbDstOffset);
}

ID3D11ModuleInstance_BindConstantBufferByName :: inline (this: *ID3D11ModuleInstance, pName: *u8, uDstSlot: u32, cbDstOffset: u32) -> HRESULT {
    return this.vtable.BindConstantBufferByName(this, pName, uDstSlot, cbDstOffset);
}

ID3D11ModuleInstance_BindResource :: inline (this: *ID3D11ModuleInstance, uSrcSlot: u32, uDstSlot: u32, uCount: u32) -> HRESULT {
    return this.vtable.BindResource(this, uSrcSlot, uDstSlot, uCount);
}

ID3D11ModuleInstance_BindResourceAsUnorderedAccessView :: inline (this: *ID3D11ModuleInstance, uSrcSrvSlot: u32, uDstUavSlot: u32, uCount: u32) -> HRESULT {
    return this.vtable.BindResourceAsUnorderedAccessView(this, uSrcSrvSlot, uDstUavSlot, uCount);
}

ID3D11ModuleInstance_BindResourceAsUnorderedAccessViewByName :: inline (this: *ID3D11ModuleInstance, pSrvName: *u8, uDstUavSlot: u32, uCount: u32) -> HRESULT {
    return this.vtable.BindResourceAsUnorderedAccessViewByName(this, pSrvName, uDstUavSlot, uCount);
}

ID3D11ModuleInstance_BindResourceByName :: inline (this: *ID3D11ModuleInstance, pName: *u8, uDstSlot: u32, uCount: u32) -> HRESULT {
    return this.vtable.BindResourceByName(this, pName, uDstSlot, uCount);
}

ID3D11ModuleInstance_BindSampler :: inline (this: *ID3D11ModuleInstance, uSrcSlot: u32, uDstSlot: u32, uCount: u32) -> HRESULT {
    return this.vtable.BindSampler(this, uSrcSlot, uDstSlot, uCount);
}

ID3D11ModuleInstance_BindSamplerByName :: inline (this: *ID3D11ModuleInstance, pName: *u8, uDstSlot: u32, uCount: u32) -> HRESULT {
    return this.vtable.BindSamplerByName(this, pName, uDstSlot, uCount);
}

ID3D11ModuleInstance_BindUnorderedAccessView :: inline (this: *ID3D11ModuleInstance, uSrcSlot: u32, uDstSlot: u32, uCount: u32) -> HRESULT {
    return this.vtable.BindUnorderedAccessView(this, uSrcSlot, uDstSlot, uCount);
}

ID3D11ModuleInstance_BindUnorderedAccessViewByName :: inline (this: *ID3D11ModuleInstance, pName: *u8, uDstSlot: u32, uCount: u32) -> HRESULT {
    return this.vtable.BindUnorderedAccessViewByName(this, pName, uDstSlot, uCount);
}

ID3DInclude_Close :: inline (this: *ID3DInclude, pData: *void) -> HRESULT {
    return this.vtable.Close(this, pData);
}

ID3DInclude_Open :: inline (this: *ID3DInclude, IncludeType: D3D_INCLUDE_TYPE, pFileName: *u8, pParentData: *void, ppData: **void, pBytes: *u32) -> HRESULT {
    return this.vtable.Open(this, IncludeType, pFileName, pParentData, ppData, pBytes);
}

vtable :: (obj: *ID3DBlob) -> *ID3D10Blob_VTable {
    return obj.vtable;
}

vtable :: (obj: *ID3DInclude) -> *ID3DInclude_VTable {
    return obj.vtable;
}

vtable :: (obj: *ID3D11ModuleInstance) -> *ID3D11ModuleInstance_VTable {
    return obj.vtable;
}

vtable :: (obj: *ID3D11Module) -> *ID3D11Module_VTable {
    return obj.vtable;
}

vtable :: (obj: *ID3D11Linker) -> *ID3D11Linker_VTable {
    return obj.vtable;
}

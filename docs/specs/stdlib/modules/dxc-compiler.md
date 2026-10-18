# dxc_compiler

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

The module parameter `NINTENDO_SUPPORT := false` selects the alternate binding surface when enabled; the DXC helper and declarations described here are documented for the default Windows-only configuration.

The signatures use `IMalloc`, `IStream`, and `BSTR` as opaque nominal placeholders supplied by a scope private to the module. They are not exported names of `dxc_compiler`; caller-facing signatures that mention them still require compatible declarations from the surrounding Windows COM/OLE ABI.

- `DXC_CP_UTF8 :: 65001`
- `DXC_CP_UTF16 :: 1200`
- `DXC_CP_UTF32 :: 12000`
- `DXC_CP_ACP :: 0`
- `DXC_CP_WIDE :: DXC_CP_UTF16`
- `DXC_HASHFLAG_INCLUDES_SOURCE :: 1`
- `DxcCreateInstanceProc :: #type (rclsid: *IID, riid: *IID, ppv: **void) -> HRESULT #c_call`
- `DxcCreateInstance2Proc :: #type (pMalloc: *IMalloc, rclsid: *IID, riid: *IID, ppv: **void) -> HRESULT #c_call`
- `DxcCreateInstance :: (rclsid: *IID, riid: *IID, ppv: **void) -> HRESULT #foreign dxcompiler`
- `DxcCreateInstance :: (rclsid: IID, riid: IID, ppv: **void) -> HRESULT #no_context`
- `DxcCreateInstance2 :: (pMalloc: *IMalloc, rclsid: *IID, riid: *IID, ppv: **void) -> HRESULT #foreign dxcompiler`
- `DxcCreateInstance2 :: (pMalloc: *IMalloc, rclsid: IID, riid: IID, ppv: **void) -> HRESULT #no_context`
- `DxcShaderHash :: struct`
- `IDxcBlob :: struct`
- `IDxcBlob_VTable :: struct #type_info_none`
- `IDxcBlob_GetBufferPointer :: inline (this: *IDxcBlob) -> *void`
- `IDxcBlob_GetBufferSize :: inline (this: *IDxcBlob) -> SIZE_T`
- `vtable :: (obj: *IDxcBlob) -> *IDxcBlob_VTable`
- `IDxcBlobEncoding :: struct`
- `IDxcBlobEncoding_VTable :: struct #type_info_none`
- `IDxcBlobEncoding_GetEncoding :: inline (this: *IDxcBlobEncoding, pKnown: *BOOL, pCodePage: *u32) -> HRESULT`
- `vtable :: (obj: *IDxcBlobEncoding) -> *IDxcBlobEncoding_VTable`
- `IDxcBlobWide :: struct`
- `IDxcBlobWide_VTable :: struct #type_info_none`
- `IDxcBlobWide_GetStringPointer :: inline (this: *IDxcBlobUtf16) -> *s16`
- `IDxcBlobWide_GetStringLength :: inline (this: *IDxcBlobUtf16) -> SIZE_T`
- `vtable :: (obj: *IDxcBlobUtf8) -> *IDxcBlobUtf8_VTable`
- `IDxcBlobUtf8 :: struct`
- `IDxcBlobUtf8_VTable :: struct #type_info_none`
- `IDxcBlobUtf8_GetStringPointer :: inline (this: *IDxcBlobUtf8) -> *u8`
- `IDxcBlobUtf8_GetStringLength :: inline (this: *IDxcBlobUtf8) -> SIZE_T`
- `vtable :: (obj: *IDxcBlobUtf16) -> *IDxcBlobWide_VTable`
- `IDxcBlobUtf16 :: IDxcBlobWide`
- `IDxcIncludeHandler :: struct`
- `IDxcIncludeHandler_VTable :: struct #type_info_none`
- `IDxcIncludeHandler_LoadSource :: inline (this: *IDxcIncludeHandler, pFilename: *s16, ppIncludeSource: **IDxcBlob) -> HRESULT`
- `vtable :: (obj: *IDxcIncludeHandler) -> *IDxcIncludeHandler_VTable`
- `DxcBuffer :: struct`
- `DxcText :: DxcBuffer`
- `DxcDefine :: struct`
- `IDxcCompilerArgs :: struct`
- `IDxcCompilerArgs_VTable :: struct #type_info_none`
- `IDxcCompilerArgs_GetArguments :: inline (this: *IDxcCompilerArgs) -> **s16`
- `IDxcCompilerArgs_GetCount :: inline (this: *IDxcCompilerArgs) -> u32`
- `IDxcCompilerArgs_AddArguments :: inline (this: *IDxcCompilerArgs, pArguments: **s16, argCount: u32) -> HRESULT`
- `IDxcCompilerArgs_AddArgumentsUTF8 :: inline (this: *IDxcCompilerArgs, pArguments: **u8, argCount: u32) -> HRESULT`
- `IDxcCompilerArgs_AddDefines :: inline (this: *IDxcCompilerArgs, pDefines: *DxcDefine, defineCount: u32) -> HRESULT`
- `vtable :: (obj: *IDxcCompilerArgs) -> *IDxcCompilerArgs_VTable`
- `IDxcLibrary :: struct`
- `IDxcLibrary_VTable :: struct #type_info_none`
- `IDxcLibrary_SetMalloc :: inline (this: *IDxcLibrary, pMalloc: *IMalloc) -> HRESULT`
- `IDxcLibrary_CreateBlobFromBlob :: inline (this: *IDxcLibrary, pBlob: *IDxcBlob, offset: u32, length: u32, ppResult: **IDxcBlob) -> HRESULT`
- `IDxcLibrary_CreateBlobFromFile :: inline (this: *IDxcLibrary, pFileName: *s16, codePage: *u32, pBlobEncoding: **IDxcBlobEncoding) -> HRESULT`
- `IDxcLibrary_CreateBlobWithEncodingFromPinned :: inline (this: *IDxcLibrary, pText: *void, size: u32, codePage: u32, pBlobEncoding: **IDxcBlobEncoding) -> HRESULT`
- `IDxcLibrary_CreateBlobWithEncodingOnHeapCopy :: inline (this: *IDxcLibrary, pText: *void, size: u32, codePage: u32, pBlobEncoding: **IDxcBlobEncoding) -> HRESULT`
- `IDxcLibrary_CreateBlobWithEncodingOnMalloc :: inline (this: *IDxcLibrary, pText: *void, pIMalloc: *IMalloc, size: u32, codePage: u32, pBlobEncoding: **IDxcBlobEncoding) -> HRESULT`
- `IDxcLibrary_CreateIncludeHandler :: inline (this: *IDxcLibrary, ppResult: **IDxcIncludeHandler) -> HRESULT`
- `IDxcLibrary_CreateStreamFromBlobReadOnly :: inline (this: *IDxcLibrary, pBlob: *IDxcBlob, ppStream: **IStream) -> HRESULT`
- `IDxcLibrary_GetBlobAsUtf8 :: inline (this: *IDxcLibrary, pBlob: *IDxcBlob, pBlobEncoding: **IDxcBlobEncoding) -> HRESULT`
- `IDxcLibrary_GetBlobAsWide :: inline (this: *IDxcLibrary, pBlob: *IDxcBlob, pBlobEncoding: **IDxcBlobEncoding) -> HRESULT`
- `vtable :: (obj: *IDxcLibrary) -> *IDxcLibrary_VTable`
- `IDxcOperationResult :: struct`
- `IDxcOperationResult_VTable :: struct #type_info_none`
- `IDxcOperationResult_GetStatus :: inline (this: *IDxcOperationResult, pStatus: *HRESULT) -> HRESULT`
- `IDxcOperationResult_GetResult :: inline (this: *IDxcOperationResult, ppResult: **IDxcBlob) -> HRESULT`
- `IDxcOperationResult_GetErrorBuffer :: inline (this: *IDxcOperationResult, ppErrors: **IDxcBlobEncoding) -> HRESULT`
- `vtable :: (obj: *IDxcOperationResult) -> *IDxcOperationResult_VTable`
- `IDxcCompiler :: struct`
- `IDxcCompiler_VTable :: struct #type_info_none`
- `IDxcCompiler_Compile :: inline (this: *IDxcCompiler, pSource: *IDxcBlob, pSourceName: *s16, pEntryPoint: *s16, pTargetProfile: *s16, pArguments: **s16, argCount: u32, pDefines: *DxcDefine, defineCount: u32, pIncludeHandler: *IDxcIncludeHandler, ppResult: **IDxcOperationResult) -> HRESULT`
- `IDxcCompiler_Preprocess :: inline (this: *IDxcCompiler, pSource: *IDxcBlob, pSourceName: *s16, pArguments: **s16, argCount: u32, pDefines: *DxcDefine, defineCount: u32, pIncludeHandler: *IDxcIncludeHandler, ppResult: **IDxcOperationResult) -> HRESULT`
- `IDxcCompiler_Disassemble :: inline (this: *IDxcCompiler, pSource: *IDxcBlob, ppDisassembly: **IDxcBlobEncoding) -> HRESULT`
- `vtable :: (obj: *IDxcCompiler) -> *IDxcCompiler_VTable`
- `IDxcCompiler2 :: struct`
- `IDxcCompiler2_VTable :: struct #type_info_none`
- `IDxcCompiler2_CompileWithDebug :: inline (this: *IDxcCompiler2, pSource: *IDxcBlob, pSourceName: *s16, pEntryPoint: *s16, pTargetProfile: *s16, pArguments: **s16, argCount: u32, pDefines: *DxcDefine, defineCount: u32, pIncludeHandler: *IDxcIncludeHandler, ppResult: **IDxcOperationResult, ppDebugBlobName: **s16, ppDebugBlob: **IDxcBlob) -> HRESULT`
- `vtable :: (obj: *IDxcCompiler2) -> *IDxcCompiler2_VTable`
- `IDxcLinker :: struct`
- `IDxcLinker_VTable :: struct #type_info_none`
- `IDxcLinker_RegisterLibrary :: inline (this: *IDxcLinker, pLibName: *s16, pLib: *IDxcBlob) -> HRESULT`
- `IDxcLinker_Link :: inline (this: *IDxcLinker, pEntryName: *s16, pTargetProfile: *s16, pLibNames: **s16, libCount: u32, pArguments: **s16, argCount: u32, ppResult: **IDxcOperationResult) -> HRESULT`
- `vtable :: (obj: *IDxcLinker) -> *IDxcLinker_VTable`
- `IDxcUtils :: struct`
- `IDxcUtils_VTable :: struct #type_info_none`
- `IDxcUtils_CreateBlobFromBlob :: inline (this: *IDxcUtils, pBlob: *IDxcBlob, offset: u32, length: u32, ppResult: **IDxcBlob) -> HRESULT`
- `IDxcUtils_CreateBlobFromPinned :: inline (this: *IDxcUtils, pData: *void, size: u32, codePage: u32, ppBlobEncoding: **IDxcBlobEncoding) -> HRESULT`
- `IDxcUtils_MoveToBlob :: inline (this: *IDxcUtils, pData: *void, pIMalloc: *IMalloc, size: u32, codePage: u32, ppBlobEncoding: **IDxcBlobEncoding) -> HRESULT`
- `IDxcUtils_CreateBlob :: inline (this: *IDxcUtils, pData: *void, size: u32, codePage: u32, ppBlobEncoding: **IDxcBlobEncoding) -> HRESULT`
- `IDxcUtils_LoadFile :: inline (this: *IDxcUtils, pFileName: *s16, pCodePage: *u32, ppBlobEncoding: **IDxcBlobEncoding) -> HRESULT`
- `IDxcUtils_CreateReadOnlyStreamFromBlob :: inline (this: *IDxcUtils, pBlob: *IDxcBlob, ppStream: **IStream) -> HRESULT`
- `IDxcUtils_CreateDefaultIncludeHandler :: inline (this: *IDxcUtils, ppResult: **IDxcIncludeHandler) -> HRESULT`
- `IDxcUtils_GetBlobAsUtf8 :: inline (this: *IDxcUtils, pBlob: *IDxcBlob, ppBlobEncoding: **IDxcBlobUtf8) -> HRESULT`
- `IDxcUtils_GetBlobAsWide :: inline (this: *IDxcUtils, pBlob: *IDxcBlob, ppBlobEncoding: **IDxcBlobUtf16) -> HRESULT`
- `IDxcUtils_GetDxilContainerPart :: inline (this: *IDxcUtils, pShader: *DxcText, DxcPart: u32, ppPartData: **void, pPartSizeInBytes: *u32) -> HRESULT`
- `IDxcUtils_CreateReflection :: inline (this: *IDxcUtils, pData: *DxcText, iid: *IID, ppvReflection: **void) -> HRESULT`
- `IDxcUtils_BuildArguments :: inline (this: *IDxcUtils, pSourceName: *s16, pEntryPoint: *s16, pTargetProfile: *s16, pArguments: **s16, argCount: u32, pDefines: *DxcDefine, defineCount: u32, ppArgs: **IDxcCompilerArgs) -> HRESULT`
- `IDxcUtils_GetPDBContents :: inline (this: *IDxcUtils, pPDBBlob: *IDxcBlob, ppHash: **IDxcBlob, ppContainer: **IDxcBlob) -> HRESULT`
- `vtable :: (obj: *IDxcUtils) -> *IDxcUtils_VTable`
- `DXC_OUT_KIND :: enum s32`
- `IDxcResult :: struct`
- `IDxcResult_VTable :: struct #type_info_none`
- `IDxcResult_HasOutput :: inline (this: *IDxcResult, dxcOutKind: DXC_OUT_KIND) -> BOOL`
- `IDxcResult_GetOutput :: inline (this: *IDxcResult, dxcOutKind: DXC_OUT_KIND, iid: *IID, ppvObject: **void, ppOutputName: **IDxcBlobUtf16) -> HRESULT`
- `IDxcResult_GetNumOutputs :: inline (this: *IDxcResult) -> u32`
- `IDxcResult_GetOutputByIndex :: inline (this: *IDxcResult, Index: u32) -> DXC_OUT_KIND`
- `IDxcResult_PrimaryOutput :: inline (this: *IDxcResult) -> DXC_OUT_KIND`
- `vtable :: (obj: *IDxcResult) -> *IDxcResult_VTable`
- `IDxcExtraOutputs :: struct`
- `IDxcExtraOutputs_VTable :: struct #type_info_none`
- `IDxcExtraOutputs_GetOutputCount :: inline (this: *IDxcExtraOutputs) -> u32`
- `IDxcExtraOutputs_GetOutput :: inline (this: *IDxcExtraOutputs, uIndex: u32, iid: *IID, ppvObject: **void, ppOutputType: **IDxcBlobUtf16, ppOutputName: **IDxcBlobUtf16) -> HRESULT`
- `vtable :: (obj: *IDxcExtraOutputs) -> *IDxcExtraOutputs_VTable`
- `IDxcCompiler3 :: struct`
- `IDxcCompiler3_VTable :: struct #type_info_none`
- `IDxcCompiler3_Compile :: inline (this: *IDxcCompiler3, pSource: *DxcText, pArguments: **s16, argCount: u32, pIncludeHandler: *IDxcIncludeHandler, riid: *IID, ppResult: **void) -> HRESULT`
- `IDxcCompiler3_Disassemble :: inline (this: *IDxcCompiler3, pObject: *DxcText, riid: *IID, ppResult: **void) -> HRESULT`
- `vtable :: (obj: *IDxcCompiler3) -> *IDxcCompiler3_VTable`
- `DxcValidatorFlags_Default : u32 : 0`
- `DxcValidatorFlags_InPlaceEdit : u32 : 1`
- `DxcValidatorFlags_RootSignatureOnly : u32 : 2`
- `DxcValidatorFlags_ModuleOnly : u32 : 4`
- `DxcValidatorFlags_ValidMask : u32 : 7`

The `IMalloc`, `IStream`, and `BSTR` placeholders are each declared as an empty struct in the dependency scope. Their native ABI layouts and operations are not described by this binding; their pointer use follows the corresponding platform COM/OLE contract.
- `IDxcValidator :: struct`
- `IDxcValidator_VTable :: struct #type_info_none`
- `IDxcValidator_Validate :: inline (this: *IDxcValidator, pShader: *IDxcBlob, Flags: u32, ppResult: **IDxcOperationResult) -> HRESULT`
- `vtable :: (obj: *IDxcValidator) -> *IDxcValidator_VTable`
- `IDxcValidator2 :: struct`
- `IDxcValidator2_VTable :: struct #type_info_none`
- `IDxcValidator2_ValidateWithDebug :: inline (this: *IDxcValidator2, pShader: *IDxcBlob, Flags: u32, pOptDebugBitcode: *DxcText, ppResult: **IDxcOperationResult) -> HRESULT`
- `vtable :: (obj: *IDxcValidator2) -> *IDxcValidator2_VTable`
- `IDxcContainerBuilder :: struct`
- `IDxcContainerBuilder_VTable :: struct #type_info_none`
- `IDxcContainerBuilder_Load :: inline (this: *IDxcContainerBuilder, pDxilContainerHeader: *IDxcBlob) -> HRESULT`
- `IDxcContainerBuilder_AddPart :: inline (this: *IDxcContainerBuilder, fourCC: u32, pSource: *IDxcBlob) -> HRESULT`
- `IDxcContainerBuilder_RemovePart :: inline (this: *IDxcContainerBuilder, fourCC: u32) -> HRESULT`
- `IDxcContainerBuilder_SerializeContainer :: inline (this: *IDxcContainerBuilder, ppResult: **IDxcOperationResult) -> HRESULT`
- `vtable :: (obj: *IDxcContainerBuilder) -> *IDxcContainerBuilder_VTable`
- `IDxcAssembler :: struct`
- `IDxcAssembler_VTable :: struct #type_info_none`
- `IDxcAssembler_AssembleToContainer :: inline (this: *IDxcAssembler, pShader: *IDxcBlob, ppResult: **IDxcOperationResult) -> HRESULT`
- `vtable :: (obj: *IDxcAssembler) -> *IDxcAssembler_VTable`
- `IDxcContainerReflection :: struct`
- `IDxcContainerReflection_VTable :: struct #type_info_none`
- `IDxcContainerReflection_Load :: inline (this: *IDxcContainerReflection, pContainer: *IDxcBlob) -> HRESULT`
- `IDxcContainerReflection_GetPartCount :: inline (this: *IDxcContainerReflection, pResult: *u32) -> HRESULT`
- `IDxcContainerReflection_GetPartKind :: inline (this: *IDxcContainerReflection, idx: u32, pResult: *u32) -> HRESULT`
- `IDxcContainerReflection_GetPartContent :: inline (this: *IDxcContainerReflection, idx: u32, ppResult: **IDxcBlob) -> HRESULT`
- `IDxcContainerReflection_FindFirstPartKind :: inline (this: *IDxcContainerReflection, kind: u32, pResult: *u32) -> HRESULT`
- `IDxcContainerReflection_GetPartReflection :: inline (this: *IDxcContainerReflection, idx: u32, iid: *IID, ppvObject: **void) -> HRESULT`
- `vtable :: (obj: *IDxcContainerReflection) -> *IDxcContainerReflection_VTable`
- `IDxcOptimizerPass :: struct`
- `IDxcOptimizerPass_VTable :: struct #type_info_none`
- `IDxcOptimizerPass_GetOptionName :: inline (this: *IDxcOptimizerPass, ppResult: **s16) -> HRESULT`
- `IDxcOptimizerPass_GetDescription :: inline (this: *IDxcOptimizerPass, ppResult: **s16) -> HRESULT`
- `IDxcOptimizerPass_GetOptionArgCount :: inline (this: *IDxcOptimizerPass, pCount: *u32) -> HRESULT`
- `IDxcOptimizerPass_GetOptionArgName :: inline (this: *IDxcOptimizerPass, argIndex: u32, ppResult: **s16) -> HRESULT`
- `IDxcOptimizerPass_GetOptionArgDescription :: inline (this: *IDxcOptimizerPass, argIndex: u32, ppResult: **s16) -> HRESULT`
- `vtable :: (obj: *IDxcOptimizerPass) -> *IDxcOptimizerPass_VTable`
- `IDxcOptimizer :: struct`
- `IDxcOptimizer_VTable :: struct #type_info_none`
- `IDxcOptimizer_GetAvailablePassCount :: inline (this: *IDxcOptimizer, pCount: *u32) -> HRESULT`
- `IDxcOptimizer_GetAvailablePass :: inline (this: *IDxcOptimizer, index: u32, ppResult: **IDxcOptimizerPass) -> HRESULT`
- `IDxcOptimizer_RunOptimizer :: inline (this: *IDxcOptimizer, pBlob: *IDxcBlob, ppOptions: **s16, optionCount: u32, pOutputModule: **IDxcBlob, ppOutputText: **IDxcBlobEncoding) -> HRESULT`
- `vtable :: (obj: *IDxcOptimizer) -> *IDxcOptimizer_VTable`
- `DxcVersionInfoFlags_None : u32 : 0`
- `DxcVersionInfoFlags_Debug : u32 : 1`
- `DxcVersionInfoFlags_Internal : u32 : 2`
- `IDxcVersionInfo :: struct`
- `IDxcVersionInfo_VTable :: struct #type_info_none`
- `IDxcVersionInfo_GetVersion :: inline (this: *IDxcVersionInfo, pMajor: *u32, pMinor: *u32) -> HRESULT`
- `IDxcVersionInfo_GetFlags :: inline (this: *IDxcVersionInfo, pFlags: *u32) -> HRESULT`
- `vtable :: (obj: *IDxcVersionInfo) -> *IDxcVersionInfo_VTable`
- `IDxcVersionInfo2 :: struct`
- `IDxcVersionInfo2_VTable :: struct #type_info_none`
- `IDxcVersionInfo2_GetCommitInfo :: inline (this: *IDxcVersionInfo2, pCommitCount: *u32, pCommitHash: **u8) -> HRESULT`
- `vtable :: (obj: *IDxcVersionInfo2) -> *IDxcVersionInfo2_VTable`
- `IDxcVersionInfo3 :: struct`
- `IDxcVersionInfo3_VTable :: struct #type_info_none`
- `IDxcVersionInfo3_GetCustomVersionString :: inline (this: *IDxcVersionInfo3, pVersionString: **u8) -> HRESULT`
- `vtable :: (obj: *IDxcVersionInfo3) -> *IDxcVersionInfo3_VTable`
- `DxcArgPair :: struct`
- `IDxcPdbUtils :: struct`
- `IDxcPdbUtils_VTable :: struct #type_info_none`
- `IDxcPdbUtils_Load :: inline (this: *IDxcPdbUtils, pPdbOrDxil: *IDxcBlob) -> HRESULT`
- `IDxcPdbUtils_GetSourceCount :: inline (this: *IDxcPdbUtils, pCount: *u32) -> HRESULT`
- `IDxcPdbUtils_GetSource :: inline (this: *IDxcPdbUtils, uIndex: u32, ppResult: **IDxcBlobEncoding) -> HRESULT`
- `IDxcPdbUtils_GetSourceName :: inline (this: *IDxcPdbUtils, uIndex: u32, pResult: *BSTR) -> HRESULT`
- `IDxcPdbUtils_GetFlagCount :: inline (this: *IDxcPdbUtils, pCount: *u32) -> HRESULT`
- `IDxcPdbUtils_GetFlag :: inline (this: *IDxcPdbUtils, uIndex: u32, pResult: *BSTR) -> HRESULT`
- `IDxcPdbUtils_GetArgCount :: inline (this: *IDxcPdbUtils, pCount: *u32) -> HRESULT`
- `IDxcPdbUtils_GetArg :: inline (this: *IDxcPdbUtils, uIndex: u32, pResult: *BSTR) -> HRESULT`
- `IDxcPdbUtils_GetArgPairCount :: inline (this: *IDxcPdbUtils, pCount: *u32) -> HRESULT`
- `IDxcPdbUtils_GetArgPair :: inline (this: *IDxcPdbUtils, uIndex: u32, pName: *BSTR, pValue: *BSTR) -> HRESULT`
- `IDxcPdbUtils_GetDefineCount :: inline (this: *IDxcPdbUtils, pCount: *u32) -> HRESULT`
- `IDxcPdbUtils_GetDefine :: inline (this: *IDxcPdbUtils, uIndex: u32, pResult: *BSTR) -> HRESULT`
- `IDxcPdbUtils_GetTargetProfile :: inline (this: *IDxcPdbUtils, pResult: *BSTR) -> HRESULT`
- `IDxcPdbUtils_GetEntryPoint :: inline (this: *IDxcPdbUtils, pResult: *BSTR) -> HRESULT`
- `IDxcPdbUtils_GetMainFileName :: inline (this: *IDxcPdbUtils, pResult: *BSTR) -> HRESULT`
- `IDxcPdbUtils_GetHash :: inline (this: *IDxcPdbUtils, ppResult: **IDxcBlob) -> HRESULT`
- `IDxcPdbUtils_GetName :: inline (this: *IDxcPdbUtils, pResult: *BSTR) -> HRESULT`
- `IDxcPdbUtils_IsFullPDB :: inline (this: *IDxcPdbUtils) -> BOOL`
- `IDxcPdbUtils_GetFullPDB :: inline (this: *IDxcPdbUtils, ppFullPDB: **IDxcBlob) -> HRESULT`
- `IDxcPdbUtils_GetVersionInfo :: inline (this: *IDxcPdbUtils, ppVersionInfo: **IDxcVersionInfo) -> HRESULT`
- `IDxcPdbUtils_SetCompiler :: inline (this: *IDxcPdbUtils, pCompiler: *IDxcCompiler3) -> HRESULT`
- `IDxcPdbUtils_CompileForFullPDB :: inline (this: *IDxcPdbUtils, ppResult: **IDxcResult) -> HRESULT`
- `IDxcPdbUtils_OverrideArgs :: inline (this: *IDxcPdbUtils, pArgPairs: *DxcArgPair, uNumArgPairs: u32) -> HRESULT`
- `IDxcPdbUtils_OverrideRootSignature :: inline (this: *IDxcPdbUtils, pRootSignature: *s16) -> HRESULT`
- `vtable :: (obj: *IDxcPdbUtils) -> *IDxcPdbUtils_VTable`
- `IDxcPdbUtils2 :: struct`
- `IDxcPdbUtils2_VTable :: struct #type_info_none`
- `IDxcPdbUtils2_Load :: inline (this: *IDxcPdbUtils2, pPdbOrDxil: *IDxcBlob) -> HRESULT`
- `IDxcPdbUtils2_GetSourceCount :: inline (this: *IDxcPdbUtils2, pCount: *u32) -> HRESULT`
- `IDxcPdbUtils2_GetSource :: inline (this: *IDxcPdbUtils2, uIndex: u32, ppResult: **IDxcBlobEncoding) -> HRESULT`
- `IDxcPdbUtils2_GetSourceName :: inline (this: *IDxcPdbUtils2, uIndex: u32, ppResult: **IDxcBlobUtf16) -> HRESULT`
- `IDxcPdbUtils2_GetLibraryPDBCount :: inline (this: *IDxcPdbUtils2, pCount: *u32) -> HRESULT`
- `IDxcPdbUtils2_GetLibraryPDB :: inline (this: *IDxcPdbUtils2, uIndex: u32, ppOutPdbUtils: **IDxcPdbUtils2, ppLibraryName: **IDxcBlobUtf16) -> HRESULT`
- `IDxcPdbUtils2_GetFlagCount :: inline (this: *IDxcPdbUtils2, pCount: *u32) -> HRESULT`
- `IDxcPdbUtils2_GetFlag :: inline (this: *IDxcPdbUtils2, uIndex: u32, ppResult: **IDxcBlobUtf16) -> HRESULT`
- `IDxcPdbUtils2_GetArgCount :: inline (this: *IDxcPdbUtils2, pCount: *u32) -> HRESULT`
- `IDxcPdbUtils2_GetArg :: inline (this: *IDxcPdbUtils2, uIndex: u32, ppResult: **IDxcBlobUtf16) -> HRESULT`
- `IDxcPdbUtils2_GetArgPairCount :: inline (this: *IDxcPdbUtils2, pCount: *u32) -> HRESULT`
- `IDxcPdbUtils2_GetArgPair :: inline (this: *IDxcPdbUtils2, uIndex: u32, ppName: **IDxcBlobUtf16, ppValue: **IDxcBlobUtf16) -> HRESULT`
- `IDxcPdbUtils2_GetDefineCount :: inline (this: *IDxcPdbUtils2, pCount: *u32) -> HRESULT`
- `IDxcPdbUtils2_GetDefine :: inline (this: *IDxcPdbUtils2, uIndex: u32, ppResult: **IDxcBlobUtf16) -> HRESULT`
- `IDxcPdbUtils2_GetTargetProfile :: inline (this: *IDxcPdbUtils2, ppResult: **IDxcBlobUtf16) -> HRESULT`
- `IDxcPdbUtils2_GetEntryPoint :: inline (this: *IDxcPdbUtils2, ppResult: **IDxcBlobUtf16) -> HRESULT`
- `IDxcPdbUtils2_GetMainFileName :: inline (this: *IDxcPdbUtils2, ppResult: **IDxcBlobUtf16) -> HRESULT`
- `IDxcPdbUtils2_GetHash :: inline (this: *IDxcPdbUtils2, ppResult: **IDxcBlob) -> HRESULT`
- `IDxcPdbUtils2_GetName :: inline (this: *IDxcPdbUtils2, ppResult: **IDxcBlobUtf16) -> HRESULT`
- `IDxcPdbUtils2_GetVersionInfo :: inline (this: *IDxcPdbUtils2, ppVersionInfo: **IDxcVersionInfo) -> HRESULT`
- `IDxcPdbUtils2_GetCustomToolchainID :: inline (this: *IDxcPdbUtils2, pID: *u32) -> HRESULT`
- `IDxcPdbUtils2_GetCustomToolchainData :: inline (this: *IDxcPdbUtils2, ppBlob: **IDxcBlob) -> HRESULT`
- `IDxcPdbUtils2_GetWholeDxil :: inline (this: *IDxcPdbUtils2, ppResult: **IDxcBlob) -> HRESULT`
- `IDxcPdbUtils2_IsFullPDB :: inline (this: *IDxcPdbUtils2) -> BOOL`
- `IDxcPdbUtils2_IsPDBRef :: inline (this: *IDxcPdbUtils2) -> BOOL`
- `vtable :: (obj: *IDxcPdbUtils2) -> *IDxcPdbUtils2_VTable`
- `dxc_get_version_string :: () -> string`
- `Shader_Type :: enum`
- `Dxc_Shader_Compiler_Args :: struct`
- `DXC_Compile :: (args: Dxc_Shader_Compiler_Args) -> [..]u8`
- `DXC_Reflect :: (dxil: []u8) -> *ID3D12ShaderReflection, bool`
- `CLSID_DxcCompiler :: GUID.{0x73E22D93, 0xE6CE, 0x47F3, .[0xB5, 0xBF, 0xF0, 0x66, 0x4F, 0x39, 0xC1, 0xB0]}`
- `CLSID_DxcUtils :: GUID.{0x6245D6AF, 0x66E0, 0x48FD, .[0x80, 0xB4, 0x4D, 0x27, 0x17, 0x96, 0x74, 0x8C]}`
- `CLSID_DxcPdbUtils :: GUID.{0x54621DFB, 0xF2CE, 0x457E, .[0xAE, 0x8C, 0xEC, 0x35, 0x5F, 0xAE, 0xEC, 0x7C]}`
- `IDxcCompiler3_UID :: GUID.{0x228B4687, 0x5A6A, 0x4730, .[0x90, 0x0C, 0x97, 0x02, 0xB2, 0x20, 0x3F, 0x54]}`
- `IDxcUtils_UID :: GUID.{0x4605C4CB, 0x2019, 0x492A, .[0xAD, 0xA4, 0x65, 0xF2, 0x0B, 0xB7, 0xD6, 0x7F]}`
- `IDxcResult_UID :: GUID.{0x58346CDA, 0xDDE7, 0x4497, .[0x94, 0x61, 0x6F, 0x87, 0xAF, 0x5E, 0x06, 0x59]}`
- `IDxcBlobUtf8_UID :: GUID.{0x3DA636C9, 0xBA71, 0x4024, .[0xA3, 0x01, 0x30, 0xCB, 0xF1, 0x25, 0x30, 0x5B]}`
- `IDxcPdbUtils_UID :: GUID.{0xE6C9647E, 0x9D6A, 0x4C3B, .[0xB9, 0x4C, 0x52, 0x4B, 0x5A, 0x6C, 0x34, 0x3D]}`
- `IDxcVersionInfo_UID :: GUID.{0xB04F5B50, 0x2059, 0x4F12, .[0xA8, 0xFF, 0xA1, 0xE0, 0xCD, 0xE1, 0xCC, 0x7E]}`
- `IDxcVersionInfo2_UID :: GUID.{0xFB6904C4, 0x42F0, 0x4B62, .[0x9C, 0x46, 0x98, 0x3A, 0xF7, 0xDA, 0x7C, 0x83]}`
- `IDxcVersionInfo3_UID :: GUID.{0x5E13E843, 0x9D25, 0x473C, .[0x9A, 0xD2, 0x03, 0xB2, 0xD0, 0xB4, 0x4B, 0x1E]}`

## Public type members

#### `DxcShaderHash` (struct)

- `Flags:      u32`
- `HashDigest: [16] u8`

#### `IDxcBlob_VTable` (struct)
- Inherited vtable prefix: `using vtable:     IUnknown_VTable`; slots in order: QueryInterface, AddRef, Release.
- `GetBufferPointer: #type (this: *IDxcBlob) -> *void #cpp_method`
- `GetBufferSize:    #type (this: *IDxcBlob) -> SIZE_T #cpp_method`

#### `IDxcBlobEncoding_VTable` (struct)
- Inherited vtable prefix: `using,except(vtable) vtable:      IDxcBlob_VTable`; slots in order: QueryInterface, AddRef, Release, GetBufferPointer, GetBufferSize.

- `GetEncoding: #type (this: *IDxcBlobEncoding, pKnown: *BOOL, pCodePage: *u32) -> HRESULT #cpp_method`

#### `IDxcBlobWide_VTable` (struct)
- Inherited vtable prefix: `using,except(vtable) vtable:           IDxcBlobEncoding_VTable`; slots in order: QueryInterface, AddRef, Release, GetBufferPointer, GetBufferSize, GetEncoding.

- `GetStringPointer: #type (this: *IDxcBlobWide) -> *s16 #cpp_method`
- `GetStringLength:  #type (this: *IDxcBlobWide) -> SIZE_T #cpp_method`

#### `IDxcBlobUtf8_VTable` (struct)
- Inherited vtable prefix: `using,except(vtable) vtable:           IDxcBlobEncoding_VTable`; slots in order: QueryInterface, AddRef, Release, GetBufferPointer, GetBufferSize, GetEncoding.

- `GetStringPointer: #type (this: *IDxcBlobUtf8) -> *u8 #cpp_method`
- `GetStringLength:  #type (this: *IDxcBlobUtf8) -> SIZE_T #cpp_method`

#### `IDxcIncludeHandler_VTable` (struct)
- Inherited vtable prefix: `using vtable: IUnknown_VTable`; slots in order: QueryInterface, AddRef, Release.

- `LoadSource:   #type (this: *IDxcIncludeHandler, pFilename: *s16, ppIncludeSource: **IDxcBlob) -> HRESULT #cpp_method`

#### `DxcBuffer` (struct)

- `Ptr:      *void`
- `Size:     SIZE_T`
- `Encoding: u32`

#### `DxcDefine` (struct)

- `Name:  *s16`
- `Value: *s16`

#### `IDxcCompilerArgs_VTable` (struct)
- Inherited vtable prefix: `using vtable:     IUnknown_VTable`; slots in order: QueryInterface, AddRef, Release.

- `GetArguments:     #type (this: *IDxcCompilerArgs) -> **s16 #cpp_method`
- `GetCount:         #type (this: *IDxcCompilerArgs) -> u32 #cpp_method`
- `AddArguments:     #type (this: *IDxcCompilerArgs, pArguments: **s16, argCount: u32) -> HRESULT #cpp_method`
- `AddArgumentsUTF8: #type (this: *IDxcCompilerArgs, pArguments: **u8, argCount: u32) -> HRESULT #cpp_method`
- `AddDefines:       #type (this: *IDxcCompilerArgs, pDefines: *DxcDefine, defineCount: u32) -> HRESULT #cpp_method`

#### `IDxcLibrary_VTable` (struct)
- Inherited vtable prefix: `using vtable:                     IUnknown_VTable`; slots in order: QueryInterface, AddRef, Release.

- `SetMalloc:                        #type (this: *IDxcLibrary, pMalloc: *IMalloc) -> HRESULT #cpp_method`
- `CreateBlobFromBlob:               #type (this: *IDxcLibrary, pBlob: *IDxcBlob, offset: u32, length: u32, ppResult: **IDxcBlob) -> HRESULT #cpp_method`
- `CreateBlobFromFile:               #type (this: *IDxcLibrary, pFileName: *s16, codePage: *u32, pBlobEncoding: **IDxcBlobEncoding) -> HRESULT #cpp_method`
- `CreateBlobWithEncodingFromPinned: #type (this: *IDxcLibrary, pText: *void, size: u32, codePage: u32, pBlobEncoding: **IDxcBlobEncoding) -> HRESULT #cpp_method`
- `CreateBlobWithEncodingOnHeapCopy: #type (this: *IDxcLibrary, pText: *void, size: u32, codePage: u32, pBlobEncoding: **IDxcBlobEncoding) -> HRESULT #cpp_method`
- `CreateBlobWithEncodingOnMalloc:   #type (this: *IDxcLibrary, pText: *void, pIMalloc: *IMalloc, size: u32, codePage: u32, pBlobEncoding: **IDxcBlobEncoding) -> HRESULT #cpp_method`
- `CreateIncludeHandler:             #type (this: *IDxcLibrary, ppResult: **IDxcIncludeHandler) -> HRESULT #cpp_method`
- `CreateStreamFromBlobReadOnly:     #type (this: *IDxcLibrary, pBlob: *IDxcBlob, ppStream: **IStream) -> HRESULT #cpp_method`
- `GetBlobAsUtf8:                    #type (this: *IDxcLibrary, pBlob: *IDxcBlob, pBlobEncoding: **IDxcBlobEncoding) -> HRESULT #cpp_method`
- `GetBlobAsWide:                    #type (this: *IDxcLibrary, pBlob: *IDxcBlob, pBlobEncoding: **IDxcBlobEncoding) -> HRESULT #cpp_method`

#### `IDxcOperationResult_VTable` (struct)
- Inherited vtable prefix: `using vtable:   IUnknown_VTable`; slots in order: QueryInterface, AddRef, Release.

- `GetStatus:      #type (this: *IDxcOperationResult, pStatus: *HRESULT) -> HRESULT #cpp_method`
- `GetResult:      #type (this: *IDxcOperationResult, ppResult: **IDxcBlob) -> HRESULT #cpp_method`
- `GetErrorBuffer: #type (this: *IDxcOperationResult, ppErrors: **IDxcBlobEncoding) -> HRESULT #cpp_method`

#### `IDxcCompiler_VTable` (struct)
- Inherited vtable prefix: `using vtable: IUnknown_VTable`; slots in order: QueryInterface, AddRef, Release.

- `Compile:      #type (this: *IDxcCompiler, pSource: *IDxcBlob, pSourceName: *s16, pEntryPoint: *s16, pTargetProfile: *s16, pArguments: **s16, argCount: u32, pDefines: *DxcDefine, defineCount: u32, pIncludeHandler: *IDxcIncludeHandler, ppResult: **IDxcOperationResult) -> HRESULT #cpp_method`
- `Preprocess:   #type (this: *IDxcCompiler, pSource: *IDxcBlob, pSourceName: *s16, pArguments: **s16, argCount: u32, pDefines: *DxcDefine, defineCount: u32, pIncludeHandler: *IDxcIncludeHandler, ppResult: **IDxcOperationResult) -> HRESULT #cpp_method`
- `Disassemble:  #type (this: *IDxcCompiler, pSource: *IDxcBlob, ppDisassembly: **IDxcBlobEncoding) -> HRESULT #cpp_method`

#### `IDxcCompiler2_VTable` (struct)
- Inherited vtable prefix: `using,except(vtable) vtable:           IDxcCompiler_VTable`; slots in order: QueryInterface, AddRef, Release, Compile, Preprocess, Disassemble.

- `CompileWithDebug: #type (this: *IDxcCompiler2, pSource: *IDxcBlob, pSourceName: *s16, pEntryPoint: *s16, pTargetProfile: *s16, pArguments: **s16, argCount: u32, pDefines: *DxcDefine, defineCount: u32, pIncludeHandler: *IDxcIncludeHandler, ppResult: **IDxcOperationResult, ppDebugBlobName: **s16, ppDebugBlob: **IDxcBlob) -> HRESULT #cpp_method`

#### `IDxcLinker_VTable` (struct)
- Inherited vtable prefix: `using vtable:    IUnknown_VTable`; slots in order: QueryInterface, AddRef, Release.

- `RegisterLibrary: #type (this: *IDxcLinker, pLibName: *s16, pLib: *IDxcBlob) -> HRESULT #cpp_method`
- `Link:            #type (this: *IDxcLinker, pEntryName: *s16, pTargetProfile: *s16, pLibNames: **s16, libCount: u32, pArguments: **s16, argCount: u32, ppResult: **IDxcOperationResult) -> HRESULT #cpp_method`

#### `IDxcUtils_VTable` (struct)
- Inherited vtable prefix: `using vtable:                 IUnknown_VTable`; slots in order: QueryInterface, AddRef, Release.

- `CreateBlobFromBlob:           #type (this: *IDxcUtils, pBlob: *IDxcBlob, offset: u32, length: u32, ppResult: **IDxcBlob) -> HRESULT #cpp_method`
- `CreateBlobFromPinned:         #type (this: *IDxcUtils, pData: *void, size: u32, codePage: u32, ppBlobEncoding: **IDxcBlobEncoding) -> HRESULT #cpp_method`
- `MoveToBlob:                   #type (this: *IDxcUtils, pData: *void, pIMalloc: *IMalloc, size: u32, codePage: u32, ppBlobEncoding: **IDxcBlobEncoding) -> HRESULT #cpp_method`
- `CreateBlob:                   #type (this: *IDxcUtils, pData: *void, size: u32, codePage: u32, ppBlobEncoding: **IDxcBlobEncoding) -> HRESULT #cpp_method`
- `LoadFile:                     #type (this: *IDxcUtils, pFileName: *s16, pCodePage: *u32, ppBlobEncoding: **IDxcBlobEncoding) -> HRESULT #cpp_method`
- `CreateReadOnlyStreamFromBlob: #type (this: *IDxcUtils, pBlob: *IDxcBlob, ppStream: **IStream) -> HRESULT #cpp_method`
- `CreateDefaultIncludeHandler:  #type (this: *IDxcUtils, ppResult: **IDxcIncludeHandler) -> HRESULT #cpp_method`
- `GetBlobAsUtf8:                #type (this: *IDxcUtils, pBlob: *IDxcBlob, ppBlobEncoding: **IDxcBlobUtf8) -> HRESULT #cpp_method`
- `GetBlobAsWide:                #type (this: *IDxcUtils, pBlob: *IDxcBlob, ppBlobEncoding: **IDxcBlobWide) -> HRESULT #cpp_method`
- `GetDxilContainerPart:         #type (this: *IDxcUtils, pShader: *DxcBuffer, DxcPart: u32, ppPartData: **void, pPartSizeInBytes: *u32) -> HRESULT #cpp_method`
- `CreateReflection:             #type (this: *IDxcUtils, pData: *DxcBuffer, iid: *IID, ppvReflection: **void) -> HRESULT #cpp_method`
- `BuildArguments:               #type (this: *IDxcUtils, pSourceName: *s16, pEntryPoint: *s16, pTargetProfile: *s16, pArguments: **s16, argCount: u32, pDefines: *DxcDefine, defineCount: u32, ppArgs: **IDxcCompilerArgs) -> HRESULT #cpp_method`
- `GetPDBContents:               #type (this: *IDxcUtils, pPDBBlob: *IDxcBlob, ppHash: **IDxcBlob, ppContainer: **IDxcBlob) -> HRESULT #cpp_method`

#### `DXC_OUT_KIND` (enum)

- `NONE           :: 0`
- `OBJECT         :: 1`
- `ERRORS         :: 2`
- `PDB            :: 3`
- `SHADER_HASH    :: 4`
- `DISASSEMBLY    :: 5`
- `HLSL           :: 6`
- `TEXT           :: 7`
- `REFLECTION     :: 8`
- `ROOT_SIGNATURE :: 9`
- `EXTRA_OUTPUTS  :: 10`
- `REMARKS        :: 11`
- `TIME_REPORT    :: 12`
- `TIME_TRACE     :: 13`
- `LAST           :: 13`
- `NUM_ENUMS      :: 14`
- `FORCE_DWORD    :: -1`
- `DXC_OUT_NONE           :: NONE`
- `DXC_OUT_OBJECT         :: OBJECT`
- `DXC_OUT_ERRORS         :: ERRORS`
- `DXC_OUT_PDB            :: PDB`
- `DXC_OUT_SHADER_HASH    :: SHADER_HASH`
- `DXC_OUT_DISASSEMBLY    :: DISASSEMBLY`
- `DXC_OUT_HLSL           :: HLSL`
- `DXC_OUT_TEXT           :: TEXT`
- `DXC_OUT_REFLECTION     :: REFLECTION`
- `DXC_OUT_ROOT_SIGNATURE :: ROOT_SIGNATURE`
- `DXC_OUT_EXTRA_OUTPUTS  :: EXTRA_OUTPUTS`
- `DXC_OUT_REMARKS        :: REMARKS`
- `DXC_OUT_TIME_REPORT    :: TIME_REPORT`
- `DXC_OUT_TIME_TRACE     :: TIME_TRACE`
- `DXC_OUT_LAST           :: LAST`
- `DXC_OUT_NUM_ENUMS      :: NUM_ENUMS`
- `DXC_OUT_FORCE_DWORD    :: FORCE_DWORD`

#### `IDxcResult_VTable` (struct)
- Inherited vtable prefix: `using,except(vtable) vtable:           IDxcOperationResult_VTable`; slots in order: QueryInterface, AddRef, Release, GetStatus, GetResult, GetErrorBuffer.

- `HasOutput:        #type (this: *IDxcResult, dxcOutKind: DXC_OUT_KIND) -> BOOL #cpp_method`
- `GetOutput:        #type (this: *IDxcResult, dxcOutKind: DXC_OUT_KIND, iid: *IID, ppvObject: **void, ppOutputName: **IDxcBlobWide) -> HRESULT #cpp_method`
- `GetNumOutputs:    #type (this: *IDxcResult) -> u32 #cpp_method`
- `GetOutputByIndex: #type (this: *IDxcResult, Index: u32) -> DXC_OUT_KIND #cpp_method`
- `PrimaryOutput:    #type (this: *IDxcResult) -> DXC_OUT_KIND #cpp_method`

#### `IDxcExtraOutputs_VTable` (struct)
- Inherited vtable prefix: `using vtable:   IUnknown_VTable`; slots in order: QueryInterface, AddRef, Release.

- `GetOutputCount: #type (this: *IDxcExtraOutputs) -> u32 #cpp_method`
- `GetOutput:      #type (this: *IDxcExtraOutputs, uIndex: u32, iid: *IID, ppvObject: **void, ppOutputType: **IDxcBlobWide, ppOutputName: **IDxcBlobWide) -> HRESULT #cpp_method`

#### `IDxcCompiler3_VTable` (struct)
- Inherited vtable prefix: `using vtable: IUnknown_VTable`; slots in order: QueryInterface, AddRef, Release.

- `Compile:      #type (this: *IDxcCompiler3, pSource: *DxcBuffer, pArguments: **s16, argCount: u32, pIncludeHandler: *IDxcIncludeHandler, riid: *IID, ppResult: **void) -> HRESULT #cpp_method`
- `Disassemble:  #type (this: *IDxcCompiler3, pObject: *DxcBuffer, riid: *IID, ppResult: **void) -> HRESULT #cpp_method`

#### `IDxcValidator_VTable` (struct)
- Inherited vtable prefix: `using vtable: IUnknown_VTable`; slots in order: QueryInterface, AddRef, Release.

- `Validate:     #type (this: *IDxcValidator, pShader: *IDxcBlob, Flags: u32, ppResult: **IDxcOperationResult) -> HRESULT #cpp_method`

#### `IDxcValidator2_VTable` (struct)
- Inherited vtable prefix: `using,except(vtable) vtable:            IDxcValidator_VTable`; slots in order: QueryInterface, AddRef, Release, Validate.

- `ValidateWithDebug: #type (this: *IDxcValidator2, pShader: *IDxcBlob, Flags: u32, pOptDebugBitcode: *DxcBuffer, ppResult: **IDxcOperationResult) -> HRESULT #cpp_method`

#### `IDxcContainerBuilder_VTable` (struct)
- Inherited vtable prefix: `using vtable:       IUnknown_VTable`; slots in order: QueryInterface, AddRef, Release.

- `Load:               #type (this: *IDxcContainerBuilder, pDxilContainerHeader: *IDxcBlob) -> HRESULT #cpp_method`
- `AddPart:            #type (this: *IDxcContainerBuilder, fourCC: u32, pSource: *IDxcBlob) -> HRESULT #cpp_method`
- `RemovePart:         #type (this: *IDxcContainerBuilder, fourCC: u32) -> HRESULT #cpp_method`
- `SerializeContainer: #type (this: *IDxcContainerBuilder, ppResult: **IDxcOperationResult) -> HRESULT #cpp_method`

#### `IDxcAssembler_VTable` (struct)
- Inherited vtable prefix: `using vtable:        IUnknown_VTable`; slots in order: QueryInterface, AddRef, Release.

- `AssembleToContainer: #type (this: *IDxcAssembler, pShader: *IDxcBlob, ppResult: **IDxcOperationResult) -> HRESULT #cpp_method`

#### `IDxcContainerReflection_VTable` (struct)
- Inherited vtable prefix: `using vtable:      IUnknown_VTable`; slots in order: QueryInterface, AddRef, Release.

- `Load:              #type (this: *IDxcContainerReflection, pContainer: *IDxcBlob) -> HRESULT #cpp_method`
- `GetPartCount:      #type (this: *IDxcContainerReflection, pResult: *u32) -> HRESULT #cpp_method`
- `GetPartKind:       #type (this: *IDxcContainerReflection, idx: u32, pResult: *u32) -> HRESULT #cpp_method`
- `GetPartContent:    #type (this: *IDxcContainerReflection, idx: u32, ppResult: **IDxcBlob) -> HRESULT #cpp_method`
- `FindFirstPartKind: #type (this: *IDxcContainerReflection, kind: u32, pResult: *u32) -> HRESULT #cpp_method`
- `GetPartReflection: #type (this: *IDxcContainerReflection, idx: u32, iid: *IID, ppvObject: **void) -> HRESULT #cpp_method`

#### `IDxcOptimizerPass_VTable` (struct)
- Inherited vtable prefix: `using vtable:            IUnknown_VTable`; slots in order: QueryInterface, AddRef, Release.

- `GetOptionName:           #type (this: *IDxcOptimizerPass, ppResult: **s16) -> HRESULT #cpp_method`
- `GetDescription:          #type (this: *IDxcOptimizerPass, ppResult: **s16) -> HRESULT #cpp_method`
- `GetOptionArgCount:       #type (this: *IDxcOptimizerPass, pCount: *u32) -> HRESULT #cpp_method`
- `GetOptionArgName:        #type (this: *IDxcOptimizerPass, argIndex: u32, ppResult: **s16) -> HRESULT #cpp_method`
- `GetOptionArgDescription: #type (this: *IDxcOptimizerPass, argIndex: u32, ppResult: **s16) -> HRESULT #cpp_method`

#### `IDxcOptimizer_VTable` (struct)
- Inherited vtable prefix: `using vtable:          IUnknown_VTable`; slots in order: QueryInterface, AddRef, Release.

- `GetAvailablePassCount: #type (this: *IDxcOptimizer, pCount: *u32) -> HRESULT #cpp_method`
- `GetAvailablePass:      #type (this: *IDxcOptimizer, index: u32, ppResult: **IDxcOptimizerPass) -> HRESULT #cpp_method`
- `RunOptimizer:          #type (this: *IDxcOptimizer, pBlob: *IDxcBlob, ppOptions: **s16, optionCount: u32, pOutputModule: **IDxcBlob, ppOutputText: **IDxcBlobEncoding) -> HRESULT #cpp_method`

#### `IDxcVersionInfo_VTable` (struct)
- Inherited vtable prefix: `using vtable: IUnknown_VTable`; slots in order: QueryInterface, AddRef, Release.

- `GetVersion:   #type (this: *IDxcVersionInfo, pMajor: *u32, pMinor: *u32) -> HRESULT #cpp_method`
- `GetFlags:     #type (this: *IDxcVersionInfo, pFlags: *u32) -> HRESULT #cpp_method`

#### `IDxcVersionInfo2_VTable` (struct)
- Inherited vtable prefix: `using,except(vtable) vtable:        IDxcVersionInfo_VTable`; slots in order: QueryInterface, AddRef, Release, GetVersion, GetFlags.

- `GetCommitInfo: #type (this: *IDxcVersionInfo2, pCommitCount: *u32, pCommitHash: **u8) -> HRESULT #cpp_method`

#### `IDxcVersionInfo3_VTable` (struct)
- Inherited vtable prefix: `using vtable:           IUnknown_VTable`; slots in order: QueryInterface, AddRef, Release.

- `GetCustomVersionString: #type (this: *IDxcVersionInfo3, pVersionString: **u8) -> HRESULT #cpp_method`

#### `DxcArgPair` (struct)

- `pName:  *s16`
- `pValue: *s16`

#### `IDxcPdbUtils_VTable` (struct)
- Inherited vtable prefix: `using vtable:          IUnknown_VTable`; slots in order: QueryInterface, AddRef, Release.

- `Load:                  #type (this: *IDxcPdbUtils, pPdbOrDxil: *IDxcBlob) -> HRESULT #cpp_method`
- `GetSourceCount:        #type (this: *IDxcPdbUtils, pCount: *u32) -> HRESULT #cpp_method`
- `GetSource:             #type (this: *IDxcPdbUtils, uIndex: u32, ppResult: **IDxcBlobEncoding) -> HRESULT #cpp_method`
- `GetSourceName:         #type (this: *IDxcPdbUtils, uIndex: u32, pResult: *BSTR) -> HRESULT #cpp_method`
- `GetFlagCount:          #type (this: *IDxcPdbUtils, pCount: *u32) -> HRESULT #cpp_method`
- `GetFlag:               #type (this: *IDxcPdbUtils, uIndex: u32, pResult: *BSTR) -> HRESULT #cpp_method`
- `GetArgCount:           #type (this: *IDxcPdbUtils, pCount: *u32) -> HRESULT #cpp_method`
- `GetArg:                #type (this: *IDxcPdbUtils, uIndex: u32, pResult: *BSTR) -> HRESULT #cpp_method`
- `GetArgPairCount:       #type (this: *IDxcPdbUtils, pCount: *u32) -> HRESULT #cpp_method`
- `GetArgPair:            #type (this: *IDxcPdbUtils, uIndex: u32, pName: *BSTR, pValue: *BSTR) -> HRESULT #cpp_method`
- `GetDefineCount:        #type (this: *IDxcPdbUtils, pCount: *u32) -> HRESULT #cpp_method`
- `GetDefine:             #type (this: *IDxcPdbUtils, uIndex: u32, pResult: *BSTR) -> HRESULT #cpp_method`
- `GetTargetProfile:      #type (this: *IDxcPdbUtils, pResult: *BSTR) -> HRESULT #cpp_method`
- `GetEntryPoint:         #type (this: *IDxcPdbUtils, pResult: *BSTR) -> HRESULT #cpp_method`
- `GetMainFileName:       #type (this: *IDxcPdbUtils, pResult: *BSTR) -> HRESULT #cpp_method`
- `GetHash:               #type (this: *IDxcPdbUtils, ppResult: **IDxcBlob) -> HRESULT #cpp_method`
- `GetName:               #type (this: *IDxcPdbUtils, pResult: *BSTR) -> HRESULT #cpp_method`
- `IsFullPDB:             #type (this: *IDxcPdbUtils) -> BOOL #cpp_method`
- `GetFullPDB:            #type (this: *IDxcPdbUtils, ppFullPDB: **IDxcBlob) -> HRESULT #cpp_method`
- `GetVersionInfo:        #type (this: *IDxcPdbUtils, ppVersionInfo: **IDxcVersionInfo) -> HRESULT #cpp_method`
- `SetCompiler:           #type (this: *IDxcPdbUtils, pCompiler: *IDxcCompiler3) -> HRESULT #cpp_method`
- `CompileForFullPDB:     #type (this: *IDxcPdbUtils, ppResult: **IDxcResult) -> HRESULT #cpp_method`
- `OverrideArgs:          #type (this: *IDxcPdbUtils, pArgPairs: *DxcArgPair, uNumArgPairs: u32) -> HRESULT #cpp_method`
- `OverrideRootSignature: #type (this: *IDxcPdbUtils, pRootSignature: *s16) -> HRESULT #cpp_method`

#### `IDxcPdbUtils2_VTable` (struct)
- Inherited vtable prefix: `using vtable:           IUnknown_VTable`; slots in order: QueryInterface, AddRef, Release.

- `Load:                   #type (this: *IDxcPdbUtils2, pPdbOrDxil: *IDxcBlob) -> HRESULT #cpp_method`
- `GetSourceCount:         #type (this: *IDxcPdbUtils2, pCount: *u32) -> HRESULT #cpp_method`
- `GetSource:              #type (this: *IDxcPdbUtils2, uIndex: u32, ppResult: **IDxcBlobEncoding) -> HRESULT #cpp_method`
- `GetSourceName:          #type (this: *IDxcPdbUtils2, uIndex: u32, ppResult: **IDxcBlobWide) -> HRESULT #cpp_method`
- `GetLibraryPDBCount:     #type (this: *IDxcPdbUtils2, pCount: *u32) -> HRESULT #cpp_method`
- `GetLibraryPDB:          #type (this: *IDxcPdbUtils2, uIndex: u32, ppOutPdbUtils: **IDxcPdbUtils2, ppLibraryName: **IDxcBlobWide) -> HRESULT #cpp_method`
- `GetFlagCount:           #type (this: *IDxcPdbUtils2, pCount: *u32) -> HRESULT #cpp_method`
- `GetFlag:                #type (this: *IDxcPdbUtils2, uIndex: u32, ppResult: **IDxcBlobWide) -> HRESULT #cpp_method`
- `GetArgCount:            #type (this: *IDxcPdbUtils2, pCount: *u32) -> HRESULT #cpp_method`
- `GetArg:                 #type (this: *IDxcPdbUtils2, uIndex: u32, ppResult: **IDxcBlobWide) -> HRESULT #cpp_method`
- `GetArgPairCount:        #type (this: *IDxcPdbUtils2, pCount: *u32) -> HRESULT #cpp_method`
- `GetArgPair:             #type (this: *IDxcPdbUtils2, uIndex: u32, ppName: **IDxcBlobWide, ppValue: **IDxcBlobWide) -> HRESULT #cpp_method`
- `GetDefineCount:         #type (this: *IDxcPdbUtils2, pCount: *u32) -> HRESULT #cpp_method`
- `GetDefine:              #type (this: *IDxcPdbUtils2, uIndex: u32, ppResult: **IDxcBlobWide) -> HRESULT #cpp_method`
- `GetTargetProfile:       #type (this: *IDxcPdbUtils2, ppResult: **IDxcBlobWide) -> HRESULT #cpp_method`
- `GetEntryPoint:          #type (this: *IDxcPdbUtils2, ppResult: **IDxcBlobWide) -> HRESULT #cpp_method`
- `GetMainFileName:        #type (this: *IDxcPdbUtils2, ppResult: **IDxcBlobWide) -> HRESULT #cpp_method`
- `GetHash:                #type (this: *IDxcPdbUtils2, ppResult: **IDxcBlob) -> HRESULT #cpp_method`
- `GetName:                #type (this: *IDxcPdbUtils2, ppResult: **IDxcBlobWide) -> HRESULT #cpp_method`
- `GetVersionInfo:         #type (this: *IDxcPdbUtils2, ppVersionInfo: **IDxcVersionInfo) -> HRESULT #cpp_method`
- `GetCustomToolchainID:   #type (this: *IDxcPdbUtils2, pID: *u32) -> HRESULT #cpp_method`
- `GetCustomToolchainData: #type (this: *IDxcPdbUtils2, ppBlob: **IDxcBlob) -> HRESULT #cpp_method`
- `GetWholeDxil:           #type (this: *IDxcPdbUtils2, ppResult: **IDxcBlob) -> HRESULT #cpp_method`
- `IsFullPDB:              #type (this: *IDxcPdbUtils2) -> BOOL #cpp_method`
- `IsPDBRef:               #type (this: *IDxcPdbUtils2) -> BOOL #cpp_method`

#### `Dxc_Shader_Compiler_Args` (struct)

- `shader_name:          string`
- `shader_source:        string`
- `shader_model_version: string`
- `shader_type:          Shader_Type`
- `entry_point:          string`
- `defines:              [] string`
- `extra_args:           [] string`
- `add_debug_info:       bool`
- `warnings_as_error:    bool`
- `skip_optimization:    bool`
- `spirv_target_env: string`
- `spirv_generate_non_semanantic_debug_info := true`
- `turn_off_validation:  bool`

#### `IDxcAssembler` (struct)

```jai
IDxcAssembler :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcAssembler_VTable;
}
```
#### `IDxcBlob` (struct)

```jai
IDxcBlob :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcBlob_VTable;
}
```
#### `IDxcBlobEncoding` (struct)

```jai
IDxcBlobEncoding :: struct {
    #as using,except(vtable) idxcblob: IDxcBlob;
    #overlay (idxcblob) vtable: *IDxcBlobEncoding_VTable;
}
```
#### `IDxcBlobUtf8` (struct)

```jai
IDxcBlobUtf8 :: struct {
    #as using,except(vtable) idxcblobencoding: IDxcBlobEncoding;
    #overlay (idxcblobencoding) vtable: *IDxcBlobUtf8_VTable;
}
```
#### `IDxcBlobWide` (struct)

```jai
IDxcBlobWide :: struct {
    #as using,except(vtable) idxcblobencoding: IDxcBlobEncoding;
    #overlay (idxcblobencoding) vtable: *IDxcBlobWide_VTable;
}
```
#### `IDxcCompiler` (struct)

```jai
IDxcCompiler :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcCompiler_VTable;
}
```
#### `IDxcCompiler2` (struct)

```jai
IDxcCompiler2 :: struct {
    #as using,except(vtable) idxccompiler: IDxcCompiler;
    #overlay (idxccompiler) vtable: *IDxcCompiler2_VTable;
}
```
#### `IDxcCompiler3` (struct)

```jai
IDxcCompiler3 :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcCompiler3_VTable;
}
```
#### `IDxcCompilerArgs` (struct)

```jai
IDxcCompilerArgs :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcCompilerArgs_VTable;
}
```
#### `IDxcContainerBuilder` (struct)

```jai
IDxcContainerBuilder :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcContainerBuilder_VTable;
}
```
#### `IDxcContainerReflection` (struct)

```jai
IDxcContainerReflection :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcContainerReflection_VTable;
}
```
#### `IDxcExtraOutputs` (struct)

```jai
IDxcExtraOutputs :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcExtraOutputs_VTable;
}
```
#### `IDxcIncludeHandler` (struct)

```jai
IDxcIncludeHandler :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcIncludeHandler_VTable;
}
```
#### `IDxcLibrary` (struct)

```jai
IDxcLibrary :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcLibrary_VTable;
}
```
#### `IDxcLinker` (struct)

```jai
IDxcLinker :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcLinker_VTable;
}
```
#### `IDxcOperationResult` (struct)

```jai
IDxcOperationResult :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcOperationResult_VTable;
}
```
#### `IDxcOptimizer` (struct)

```jai
IDxcOptimizer :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcOptimizer_VTable;
}
```
#### `IDxcOptimizerPass` (struct)

```jai
IDxcOptimizerPass :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcOptimizerPass_VTable;
}
```
#### `IDxcPdbUtils` (struct)

```jai
IDxcPdbUtils :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcPdbUtils_VTable;
}
```
#### `IDxcPdbUtils2` (struct)

```jai
IDxcPdbUtils2 :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcPdbUtils2_VTable;
}
```
#### `IDxcResult` (struct)

```jai
IDxcResult :: struct {
    #as using,except(vtable) idxcoperationresult: IDxcOperationResult;
    #overlay (idxcoperationresult) vtable: *IDxcResult_VTable;
}
```
#### `IDxcUtils` (struct)

```jai
IDxcUtils :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcUtils_VTable;
}
```
#### `IDxcValidator` (struct)

```jai
IDxcValidator :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcValidator_VTable;
}
```
#### `IDxcValidator2` (struct)

```jai
IDxcValidator2 :: struct {
    #as using,except(vtable) idxcvalidator: IDxcValidator;
    #overlay (idxcvalidator) vtable: *IDxcValidator2_VTable;
}
```
#### `IDxcVersionInfo` (struct)

```jai
IDxcVersionInfo :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcVersionInfo_VTable;
}
```
#### `IDxcVersionInfo2` (struct)

```jai
IDxcVersionInfo2 :: struct {
    #as using,except(vtable) idxcversioninfo: IDxcVersionInfo;
    #overlay (idxcversioninfo) vtable: *IDxcVersionInfo2_VTable;
}
```
#### `IDxcVersionInfo3` (struct)

```jai
IDxcVersionInfo3 :: struct {
    #as using,except(vtable) iunknown: IUnknown;
    #overlay (iunknown) vtable: *IDxcVersionInfo3_VTable;
}
```
#### `Shader_Type` (enum)

```jai
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
```
## Behavioral contract and constraints

The public surface is a direct native binding: declared records, constants, and function signatures preserve the represented external ABI; this wrapper does not itself define a different ownership or error model. Native status/error values and handle lifetimes follow the external API. ABI compatibility across external-library versions is not specified unless a version note appears above.

`DXC_Compile` takes `Dxc_Shader_Compiler_Args`: `shader_name` supplies the source name, `shader_source` the UTF-8 source text, `entry_point` the entry point, and `shader_type` together with `shader_model_version` selects the target profile (`vs`, `ps`, `cs`, `hs`, `ds`, `ms`, `as`, or `lib` prefix). The wrapper requests HLSL language version 2021. Each `defines` entry becomes a define argument, and `extra_args` are appended as additional compiler arguments. `skip_optimization`, `add_debug_info`, and `warnings_as_error` control the corresponding compiler options; `turn_off_validation` disables validation. A non-empty `spirv_target_env` requests SPIR-V output for that environment; `spirv_generate_non_semanantic_debug_info` controls the wrapper's SPIR-V debug-info option. Compilation returns the compiled byte sequence, or an empty sequence when compilation reports failure.

`DXC_Reflect` accepts the compiled bytes and returns `(reflection, success)`: on success, `reflection` is an `*ID3D12ShaderReflection`; on failure it returns `null, false`.

External libraries/frameworks named in source: `dxcompiler`, `ole32`.

Platform coverage note: exports for the Nintendo-specific binding variant are unspecified.

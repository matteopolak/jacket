# nvidia_aftermath


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `GFSDK_Aftermath_Version :: enum u32`
- `GFSDK_Aftermath_Result :: enum u32`
- `GFSDK_Aftermath_Context_Status :: enum u32`
- `GFSDK_Aftermath_Device_Status :: enum u32`
- `GFSDK_Aftermath_FeatureFlags :: enum u32`
- `GFSDK_Aftermath_ContextHandle__ :: struct`
- `GFSDK_Aftermath_ContextHandle :: *GFSDK_Aftermath_ContextHandle__`
- `GFSDK_Aftermath_ResourceHandle__ :: struct`
- `GFSDK_Aftermath_ResourceHandle :: *GFSDK_Aftermath_ResourceHandle__`
- `GFSDK_Aftermath_ContextData :: struct`
- `GFSDK_Aftermath_ResourceDescriptor :: struct`
- `GFSDK_Aftermath_PageFaultInformation :: struct`
- `GFSDK_Aftermath_DX12_Initialize :: (version: GFSDK_Aftermath_Version, flags: u32, pDx12Device: *ID3D12Device) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_DX12_CreateContextHandle :: (pDx12Unknown: *IUnknown, pOutContextHandle: *GFSDK_Aftermath_ContextHandle) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_ReleaseContextHandle :: (contextHandle: GFSDK_Aftermath_ContextHandle) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_SetEventMarker :: (contextHandle: GFSDK_Aftermath_ContextHandle, pMarkerData: *void, markerDataSize: u32) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_GetData :: (numContexts: u32, pContextHandles: *GFSDK_Aftermath_ContextHandle, pOutContextData: *GFSDK_Aftermath_ContextData) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_GetContextError :: (pContextData: *GFSDK_Aftermath_ContextData) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_GetDeviceStatus :: (pOutStatus: *GFSDK_Aftermath_Device_Status) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_GetPageFaultInformation :: (pOutPageFaultInformation: *GFSDK_Aftermath_PageFaultInformation) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_DX12_RegisterResource :: (pApiResource: *ID3D12Resource, pOutResourceHandle: *GFSDK_Aftermath_ResourceHandle) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_DX12_UnregisterResource :: (resourceHandle: GFSDK_Aftermath_ResourceHandle) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `PFN_GFSDK_Aftermath_DX12_Initialize :: #type (version: GFSDK_Aftermath_Version, flags: u32, pDx12Device: *ID3D12Device) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_DX12_CreateContextHandle :: #type (pDx12CommandList: *IUnknown, pOutContextHandle: *GFSDK_Aftermath_ContextHandle) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_DX12_RegisterResource :: #type (pApiResource: *ID3D12Resource, pOutResourceHandle: *GFSDK_Aftermath_ResourceHandle) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_DX12_UnregisterResource :: #type (resourceHandle: GFSDK_Aftermath_ResourceHandle) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_ReleaseContextHandle :: #type (contextHandle: GFSDK_Aftermath_ContextHandle) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_SetEventMarker :: #type (contextHandle: GFSDK_Aftermath_ContextHandle, markerData: *void, markerDataSize: u32) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_GetData :: #type (numContexts: u32, ppContextHandles: *GFSDK_Aftermath_ContextHandle, pOutContextData: *GFSDK_Aftermath_ContextData) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_GetContextError :: #type (pContextData: *GFSDK_Aftermath_ContextData) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_GetDeviceStatus :: #type (pOutStatus: *GFSDK_Aftermath_Device_Status) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_GetPageFaultInformation :: #type (pOutPageFaultInformation: *GFSDK_Aftermath_PageFaultInformation) -> GFSDK_Aftermath_Result #c_call`
- `GFSDK_Aftermath_GpuCrashDumpWatchedApiFlags :: enum u32`
- `GFSDK_Aftermath_GpuCrashDumpFeatureFlags :: enum u32`
- `GFSDK_Aftermath_GpuCrashDumpDescriptionKey :: enum u32`
- `GFSDK_Aftermath_CrashDump_Status :: enum u32`
- `PFN_GFSDK_Aftermath_AddGpuCrashDumpDescription :: #type (key: u32, value: *u8) -> void #c_call`
- `PFN_GFSDK_Aftermath_GpuCrashDumpCb :: #type (pGpuCrashDump: *void, gpuCrashDumpSize: u32, pUserData: *void) -> void #c_call`
- `PFN_GFSDK_Aftermath_ShaderDebugInfoCb :: #type (pShaderDebugInfo: *void, shaderDebugInfoSize: u32, pUserData: *void) -> void #c_call`
- `PFN_GFSDK_Aftermath_GpuCrashDumpDescriptionCb :: #type (addValue: PFN_GFSDK_Aftermath_AddGpuCrashDumpDescription, pUserData: *void) -> void #c_call`
- `PFN_GFSDK_Aftermath_ResolveMarkerCb :: #type (pMarkerData: *void, markerDataSize: u32, pUserData: *void, ppResolvedMarkerData: **void, pResolvedMarkerDataSize: *u32) -> void #c_call`
- `GFSDK_Aftermath_EnableGpuCrashDumps :: (apiVersion: GFSDK_Aftermath_Version, watchedApis: u32, flags: u32, gpuCrashDumpCb: PFN_GFSDK_Aftermath_GpuCrashDumpCb, shaderDebugInfoCb: PFN_GFSDK_Aftermath_ShaderDebugInfoCb, descriptionCb: PFN_GFSDK_Aftermath_GpuCrashDumpDescriptionCb, resolveMarkerCb: PFN_GFSDK_Aftermath_ResolveMarkerCb, pUserData: *void) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_DisableGpuCrashDumps :: () -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_GetCrashDumpStatus :: (pOutStatus: *GFSDK_Aftermath_CrashDump_Status) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `PFN_GFSDK_Aftermath_EnableGpuCrashDumps :: #type (apiVersion: GFSDK_Aftermath_Version, watchedApis: u32, flags: u32, gpuCrashDumpCb: PFN_GFSDK_Aftermath_GpuCrashDumpCb, shaderDebugInfoCb: PFN_GFSDK_Aftermath_ShaderDebugInfoCb, descriptionCb: PFN_GFSDK_Aftermath_GpuCrashDumpDescriptionCb, resolveMarkerCb: PFN_GFSDK_Aftermath_ResolveMarkerCb, pUserData: *void) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_DisableGpuCrashDumps :: #type () -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_GetCrashDumpStatus :: #type (pOutStatus: *GFSDK_Aftermath_CrashDump_Status) -> GFSDK_Aftermath_Result #c_call`
- `anon_enum_3 :: enum s32`
- `GFSDK_Aftermath_ShaderDebugInfoIdentifier :: struct`
- `GFSDK_Aftermath_ShaderBinaryHash :: struct`
- `GFSDK_Aftermath_ShaderDebugName :: struct`
- `GFSDK_Aftermath_GraphicsApi :: enum u32`
- `GFSDK_Aftermath_GpuCrashDump_BaseInfo :: struct`
- `GFSDK_Aftermath_GpuCrashDump_DeviceInfo :: struct`
- `GFSDK_Aftermath_GpuCrashDump_SystemInfo :: struct`
- `GFSDK_Aftermath_GpuCrashDump_GpuInfo :: struct`
- `GFSDK_Aftermath_FaultType :: enum u32`
- `GFSDK_Aftermath_AccessType :: enum u32`
- `GFSDK_Aftermath_Engine :: enum u32`
- `GFSDK_Aftermath_Client :: enum u32`
- `GFSDK_Aftermath_GpuCrashDump_PageFaultInfo :: struct`
- `GFSDK_Aftermath_GpuCrashDump_ResourceInfo :: struct`
- `GFSDK_Aftermath_ShaderType :: enum u32`
- `GFSDK_Aftermath_GpuCrashDump_ShaderInfo :: struct`
- `GFSDK_Aftermath_Context_Type :: enum u32`
- `GFSDK_Aftermath_EventMarkerDataOwnership :: enum u32`
- `GFSDK_Aftermath_GpuCrashDump_EventMarkerInfo :: struct`
- `GFSDK_Aftermath_GpuCrashDumpDecoderFlags :: enum u32`
- `GFSDK_Aftermath_GpuCrashDumpFormatterFlags :: enum u32`
- `GFSDK_Aftermath_GpuCrashDump_Decoder__ :: struct`
- `GFSDK_Aftermath_GpuCrashDump_Decoder :: *GFSDK_Aftermath_GpuCrashDump_Decoder__`
- `PFN_GFSDK_Aftermath_SetData :: #type (pData: *void, size: u32) -> void #c_call`
- `PFN_GFSDK_Aftermath_ShaderDebugInfoLookupCb :: #type (pIdentifier: *GFSDK_Aftermath_ShaderDebugInfoIdentifier, setShaderDebugInfo: PFN_GFSDK_Aftermath_SetData, pUserData: *void) -> void #c_call`
- `PFN_GFSDK_Aftermath_ShaderLookupCb :: #type (pShaderHash: *GFSDK_Aftermath_ShaderBinaryHash, setShaderBinary: PFN_GFSDK_Aftermath_SetData, pUserData: *void) -> void #c_call`
- `PFN_GFSDK_Aftermath_ShaderSourceDebugInfoLookupCb :: #type (pShaderDebugName: *GFSDK_Aftermath_ShaderDebugName, setShaderBinary: PFN_GFSDK_Aftermath_SetData, pUserData: *void) -> void #c_call`
- `GFSDK_Aftermath_GpuCrashDump_CreateDecoder :: (apiVersion: GFSDK_Aftermath_Version, pGpuCrashDump: *void, gpuCrashDumpSize: u32, pDecoder: *GFSDK_Aftermath_GpuCrashDump_Decoder) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_GpuCrashDump_DestroyDecoder :: (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_GpuCrashDump_GetBaseInfo :: (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, pBaseInfo: *GFSDK_Aftermath_GpuCrashDump_BaseInfo) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_GpuCrashDump_GetDescriptionSize :: (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, key: u32, pValueSize: *u32) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_GpuCrashDump_GetDescription :: (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, key: u32, valueBufferSize: u32, pValue: *u8) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_GpuCrashDump_GetDeviceInfo :: (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, pDeviceInfo: *GFSDK_Aftermath_GpuCrashDump_DeviceInfo) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_GpuCrashDump_GetSystemInfo :: (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, pSystemInfo: *GFSDK_Aftermath_GpuCrashDump_SystemInfo) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_GpuCrashDump_GetGpuInfoCount :: (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, pGpuCount: *u32) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_GpuCrashDump_GetGpuInfo :: (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, gpuInfoBufferCount: u32, pGpuInfo: *GFSDK_Aftermath_GpuCrashDump_GpuInfo) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_GpuCrashDump_GetPageFaultInfo :: (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, pPageFaultInfo: *GFSDK_Aftermath_GpuCrashDump_PageFaultInfo) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_GpuCrashDump_GetPageFaultResourceInfo :: (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, resourceInfoCount: u32, pResourceInfo: *GFSDK_Aftermath_GpuCrashDump_ResourceInfo) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_GpuCrashDump_GetActiveShadersInfoCount :: (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, pShaderCount: *u32) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_GpuCrashDump_GetActiveShadersInfo :: (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, shaderInfoBufferCount: u32, pShaderInfo: *GFSDK_Aftermath_GpuCrashDump_ShaderInfo) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_GpuCrashDump_GetEventMarkersInfoCount :: (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, pMarkerCount: *u32) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_GpuCrashDump_GetEventMarkersInfo :: (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, markerInfoBufferCount: u32, pMarkerInfo: *GFSDK_Aftermath_GpuCrashDump_EventMarkerInfo) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_GpuCrashDump_GenerateJSON :: (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, decoderFlags: u32, formatFlags: u32, shaderDebugInfoLookupCb: PFN_GFSDK_Aftermath_ShaderDebugInfoLookupCb, shaderLookupCb: PFN_GFSDK_Aftermath_ShaderLookupCb, shaderSourceDebugInfoLookupCb: PFN_GFSDK_Aftermath_ShaderSourceDebugInfoLookupCb, pUserData: *void, pJsonSize: *u32) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_GpuCrashDump_GetJSON :: (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, jsonBufferSize: u32, pJson: *u8) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_GetShaderDebugInfoIdentifier :: (apiVersion: GFSDK_Aftermath_Version, pShaderDebugInfo: *void, shaderDebugInfoSize: u32, pIdentifier: *GFSDK_Aftermath_ShaderDebugInfoIdentifier) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_GetShaderHash :: (apiVersion: GFSDK_Aftermath_Version, pShader: *D3D12_SHADER_BYTECODE, pShaderHash: *GFSDK_Aftermath_ShaderBinaryHash) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_GetShaderDebugName :: (apiVersion: GFSDK_Aftermath_Version, pShader: *D3D12_SHADER_BYTECODE, pShaderDebugName: *GFSDK_Aftermath_ShaderDebugName) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `GFSDK_Aftermath_GetShaderHashForShaderInfo :: (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, pShaderInfo: *GFSDK_Aftermath_GpuCrashDump_ShaderInfo, pShaderHash: *GFSDK_Aftermath_ShaderBinaryHash) -> GFSDK_Aftermath_Result #foreign nvidia_aftermath`
- `PFN_GFSDK_Aftermath_GpuCrashDump_CreateDecoder :: #type (apiVersion: GFSDK_Aftermath_Version, pGpuCrashDump: *void, gpuCrashDumpSize: u32, pDecoder: *GFSDK_Aftermath_GpuCrashDump_Decoder) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_GpuCrashDump_DestroyDecoder :: #type (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_GpuCrashDump_GetBaseInfo :: #type (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, pBaseInfo: *GFSDK_Aftermath_GpuCrashDump_BaseInfo) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_GpuCrashDump_GetDescriptionSize :: #type (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, key: u32, pValueSize: *u32) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_GpuCrashDump_GetDescription :: #type (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, key: u32, valueBufferSize: u32, pValue: *u8) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_GpuCrashDump_GetDeviceInfo :: #type (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, pDeviceInfo: *GFSDK_Aftermath_GpuCrashDump_DeviceInfo) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_GpuCrashDump_GetSystemInfo :: #type (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, pSystemInfo: *GFSDK_Aftermath_GpuCrashDump_SystemInfo) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_GpuCrashDump_GetGpuInfoCount :: #type (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, pGpuCount: *u32) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_GpuCrashDump_GetGpuInfo :: #type (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, gpuInfoBufferCount: u32, pGpuInfo: *GFSDK_Aftermath_GpuCrashDump_GpuInfo) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_GpuCrashDump_GetPageFaultInfo :: #type (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, pPageFaultInfo: *GFSDK_Aftermath_GpuCrashDump_PageFaultInfo) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_GpuCrashDump_GetPageFaultResourceInfo :: #type (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, resourceInfoCount: u32, pResourceInfo: *GFSDK_Aftermath_GpuCrashDump_ResourceInfo) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_GpuCrashDump_GetActiveShadersInfoCount :: #type (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, pShaderCount: *u32) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_GpuCrashDump_GetActiveShadersInfo :: #type (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, shaderInfoBufferCount: u32, pShaderInfo: *GFSDK_Aftermath_GpuCrashDump_ShaderInfo) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_GpuCrashDump_GetEventMarkersInfoCount :: #type (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, markerInfoBufferCount: u32) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_GpuCrashDump_GetEventMarkersInfo :: #type (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, markerInfoBufferCount: u32, pMarkerInfo: *GFSDK_Aftermath_GpuCrashDump_EventMarkerInfo) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_GpuCrashDump_GenerateJSON :: #type (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, decoderFlags: u32, formatFlags: u32, shaderDebugInfoLookupCb: PFN_GFSDK_Aftermath_ShaderDebugInfoLookupCb, shaderLookupCb: PFN_GFSDK_Aftermath_ShaderLookupCb, shaderSourceDebugInfoLookupCb: PFN_GFSDK_Aftermath_ShaderSourceDebugInfoLookupCb, pUserData: *void, pJsonSize: *u32) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_GpuCrashDump_GetJSON :: #type (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, jsonBufferSize: u32, pJson: *u8) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_GetShaderDebugInfoIdentifier :: #type (apiVersion: GFSDK_Aftermath_Version, pShaderDebugInfo: *void, shaderDebugInfoSize: u32, pIdentifier: *GFSDK_Aftermath_ShaderDebugInfoIdentifier) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_GetShaderHashForShaderInfo :: #type (decoder: GFSDK_Aftermath_GpuCrashDump_Decoder, pShaderInfo: *GFSDK_Aftermath_GpuCrashDump_ShaderInfo, pShaderHash: *GFSDK_Aftermath_ShaderBinaryHash) -> GFSDK_Aftermath_Result #c_call`
- `PFN_GFSDK_Aftermath_GetShaderHash :: #type (apiVersion: GFSDK_Aftermath_Version, pShader: *D3D12_SHADER_BYTECODE, pShaderHash: *GFSDK_Aftermath_ShaderBinaryHash) -> GFSDK_Aftermath_Result #c_call`
- `GPFN_GFSDK_Aftermath_GetShaderDebugName :: #type (apiVersion: GFSDK_Aftermath_Version, pShader: *D3D12_SHADER_BYTECODE, pShaderDebugName: *GFSDK_Aftermath_ShaderDebugName) -> GFSDK_Aftermath_Result #c_call`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `GFSDK_Aftermath_AccessType`

```jai
GFSDK_Aftermath_AccessType :: enum u32 {
    Unknown :: 0;
    Read    :: 1;
    Write   :: 2;
    Atomic  :: 3;

    GFSDK_Aftermath_AccessType_Unknown :: Unknown;
    GFSDK_Aftermath_AccessType_Read    :: Read;
    GFSDK_Aftermath_AccessType_Write   :: Write;
    GFSDK_Aftermath_AccessType_Atomic  :: Atomic;
}
```
### `GFSDK_Aftermath_Client`

```jai
GFSDK_Aftermath_Client :: enum u32 {
    Unknown                   :: 0;
    HostInterface             :: 1;
    FrontEnd                  :: 2;
    PrimitiveDistributor      :: 3;
    GraphicsProcessingCluster :: 4;
    PolymorphEngine           :: 5;
    RasterEngine              :: 6;
    Rasterizer2D              :: 7;
    RenderOutputUnit          :: 8;
    TextureProcessingCluster  :: 9;
    CopyEngine                :: 10;
    VideoDecoder              :: 11;
    VideoEncoder              :: 12;
    Other                     :: 13;

    GFSDK_Aftermath_Client_Unknown                   :: Unknown;
    GFSDK_Aftermath_Client_HostInterface             :: HostInterface;
    GFSDK_Aftermath_Client_FrontEnd                  :: FrontEnd;
    GFSDK_Aftermath_Client_PrimitiveDistributor      :: PrimitiveDistributor;
    GFSDK_Aftermath_Client_GraphicsProcessingCluster :: GraphicsProcessingCluster;
    GFSDK_Aftermath_Client_PolymorphEngine           :: PolymorphEngine;
    GFSDK_Aftermath_Client_RasterEngine              :: RasterEngine;
    GFSDK_Aftermath_Client_Rasterizer2D              :: Rasterizer2D;
    GFSDK_Aftermath_Client_RenderOutputUnit          :: RenderOutputUnit;
    GFSDK_Aftermath_Client_TextureProcessingCluster  :: TextureProcessingCluster;
    GFSDK_Aftermath_Client_CopyEngine                :: CopyEngine;
    GFSDK_Aftermath_Client_VideoDecoder              :: VideoDecoder;
    GFSDK_Aftermath_Client_VideoEncoder              :: VideoEncoder;
    GFSDK_Aftermath_Client_Other                     :: Other;
}
```
### `GFSDK_Aftermath_ContextData`

```jai
GFSDK_Aftermath_ContextData :: struct {
    struct {
        markerData: *void;

        #overlay (markerData) ptr_align_markerData: u64;
    }
    markerSize: u32;
    status:     GFSDK_Aftermath_Context_Status;
}
```
### `GFSDK_Aftermath_ContextHandle__`

```jai
GFSDK_Aftermath_ContextHandle__ :: struct {
    ID: s32;
}
```
### `GFSDK_Aftermath_Context_Status`

```jai
GFSDK_Aftermath_Context_Status :: enum u32 {
    NotStarted :: 0;

    Executing  :: 1;

    Finished   :: 2;

    Invalid    :: 3;

    GFSDK_Aftermath_Context_Status_NotStarted :: NotStarted;

    GFSDK_Aftermath_Context_Status_Executing  :: Executing;

    GFSDK_Aftermath_Context_Status_Finished   :: Finished;

    GFSDK_Aftermath_Context_Status_Invalid    :: Invalid;
}
```
### `GFSDK_Aftermath_Context_Type`

```jai
GFSDK_Aftermath_Context_Type :: enum u32 {
    Invalid      :: 0;
    Immediate    :: 1;
    CommandList  :: 2;
    Bundle       :: 3;
    CommandQueue :: 4;

    GFSDK_Aftermath_Context_Type_Invalid      :: Invalid;
    GFSDK_Aftermath_Context_Type_Immediate    :: Immediate;
    GFSDK_Aftermath_Context_Type_CommandList  :: CommandList;
    GFSDK_Aftermath_Context_Type_Bundle       :: Bundle;
    GFSDK_Aftermath_Context_Type_CommandQueue :: CommandQueue;
}
```
### `GFSDK_Aftermath_CrashDump_Status`

```jai
GFSDK_Aftermath_CrashDump_Status :: enum u32 {
    NotStarted           :: 0;

    CollectingData       :: 1;

    CollectingDataFailed :: 2;

    InvokingCallback     :: 3;

    Finished             :: 4;

    Unknown              :: 5;

    GFSDK_Aftermath_CrashDump_Status_NotStarted           :: NotStarted;

    GFSDK_Aftermath_CrashDump_Status_CollectingData       :: CollectingData;

    GFSDK_Aftermath_CrashDump_Status_CollectingDataFailed :: CollectingDataFailed;

    GFSDK_Aftermath_CrashDump_Status_InvokingCallback     :: InvokingCallback;

    GFSDK_Aftermath_CrashDump_Status_Finished             :: Finished;

    GFSDK_Aftermath_CrashDump_Status_Unknown              :: Unknown;
}
```
### `GFSDK_Aftermath_Device_Status`

```jai
GFSDK_Aftermath_Device_Status :: enum u32 {
    Active                  :: 0;

    Timeout                 :: 1;

    OutOfMemory             :: 2;

    PageFault               :: 3;

    Stopped                 :: 4;

    Reset                   :: 5;

    Unknown                 :: 6;

    DmaFault                :: 7;

    DeviceRemovedNoGpuFault :: 8;

    GFSDK_Aftermath_Device_Status_Active                  :: Active;

    GFSDK_Aftermath_Device_Status_Timeout                 :: Timeout;

    GFSDK_Aftermath_Device_Status_OutOfMemory             :: OutOfMemory;

    GFSDK_Aftermath_Device_Status_PageFault               :: PageFault;

    GFSDK_Aftermath_Device_Status_Stopped                 :: Stopped;

    GFSDK_Aftermath_Device_Status_Reset                   :: Reset;

    GFSDK_Aftermath_Device_Status_Unknown                 :: Unknown;

    GFSDK_Aftermath_Device_Status_DmaFault                :: DmaFault;

    GFSDK_Aftermath_Device_Status_DeviceRemovedNoGpuFault :: DeviceRemovedNoGpuFault;
}
```
### `GFSDK_Aftermath_Engine`

```jai
GFSDK_Aftermath_Engine :: enum u32 {
    Unknown         :: 0;
    Graphics        :: 1;
    GraphicsCompute :: 2;
    Display         :: 3;
    CopyEngine      :: 4;
    VideoDecoder    :: 5;
    VideoEncoder    :: 6;
    Other           :: 7;

    GFSDK_Aftermath_Engine_Unknown         :: Unknown;
    GFSDK_Aftermath_Engine_Graphics        :: Graphics;
    GFSDK_Aftermath_Engine_GraphicsCompute :: GraphicsCompute;
    GFSDK_Aftermath_Engine_Display         :: Display;
    GFSDK_Aftermath_Engine_CopyEngine      :: CopyEngine;
    GFSDK_Aftermath_Engine_VideoDecoder    :: VideoDecoder;
    GFSDK_Aftermath_Engine_VideoEncoder    :: VideoEncoder;
    GFSDK_Aftermath_Engine_Other           :: Other;
}
```
### `GFSDK_Aftermath_EventMarkerDataOwnership`

```jai
GFSDK_Aftermath_EventMarkerDataOwnership :: enum u32 {
    User    :: 0;

    Decoder :: 1;

    GFSDK_Aftermath_EventMarkerDataOwnership_User    :: User;

    GFSDK_Aftermath_EventMarkerDataOwnership_Decoder :: Decoder;
}
```
### `GFSDK_Aftermath_FaultType`

```jai
GFSDK_Aftermath_FaultType :: enum u32 {
    Unknown                 :: 0;
    AddressTranslationError :: 1;
    IllegalAccessError      :: 2;

    GFSDK_Aftermath_FaultType_Unknown                 :: Unknown;
    GFSDK_Aftermath_FaultType_AddressTranslationError :: AddressTranslationError;
    GFSDK_Aftermath_FaultType_IllegalAccessError      :: IllegalAccessError;
}
```
### `GFSDK_Aftermath_FeatureFlags`

```jai
GFSDK_Aftermath_FeatureFlags :: enum u32 {
    Minimum                    :: 0;

    EnableMarkers              :: 1;

    EnableResourceTracking     :: 2;

    CallStackCapturing         :: 1073741824;

    GenerateShaderDebugInfo    :: 8;

    EnableShaderErrorReporting :: 16;

    GFSDK_Aftermath_FeatureFlags_Minimum                    :: Minimum;

    GFSDK_Aftermath_FeatureFlags_EnableMarkers              :: EnableMarkers;

    GFSDK_Aftermath_FeatureFlags_EnableResourceTracking     :: EnableResourceTracking;

    GFSDK_Aftermath_FeatureFlags_CallStackCapturing         :: CallStackCapturing;

    GFSDK_Aftermath_FeatureFlags_GenerateShaderDebugInfo    :: GenerateShaderDebugInfo;

    GFSDK_Aftermath_FeatureFlags_EnableShaderErrorReporting :: EnableShaderErrorReporting;
}
```
### `GFSDK_Aftermath_GpuCrashDumpDecoderFlags`

```jai
GFSDK_Aftermath_GpuCrashDumpDecoderFlags :: enum u32 {
    BASE_INFO           :: 1;

    DEVICE_INFO         :: 2;

    OS_INFO             :: 4;

    DISPLAY_DRIVER_INFO :: 8;

    GPU_INFO            :: 16;

    PAGE_FAULT_INFO     :: 32;

    SHADER_INFO         :: 64;

    WARP_STATE_INFO     :: 128;

    SHADER_MAPPING_INFO :: 256;

    EVENT_MARKER_INFO   :: 512;

    CALL_STACK_INFO     :: 1024;

    DESCRIPTION_INFO    :: 2048;

    FAULTED_WARP_INFO   :: 4096;

    FINGERPRINT_INFO    :: 8192;

    ALL_INFO            :: 16383;

    GFSDK_Aftermath_GpuCrashDumpDecoderFlags_BASE_INFO           :: BASE_INFO;

    GFSDK_Aftermath_GpuCrashDumpDecoderFlags_DEVICE_INFO         :: DEVICE_INFO;

    GFSDK_Aftermath_GpuCrashDumpDecoderFlags_OS_INFO             :: OS_INFO;

    GFSDK_Aftermath_GpuCrashDumpDecoderFlags_DISPLAY_DRIVER_INFO :: DISPLAY_DRIVER_INFO;

    GFSDK_Aftermath_GpuCrashDumpDecoderFlags_GPU_INFO            :: GPU_INFO;

    GFSDK_Aftermath_GpuCrashDumpDecoderFlags_PAGE_FAULT_INFO     :: PAGE_FAULT_INFO;

    GFSDK_Aftermath_GpuCrashDumpDecoderFlags_SHADER_INFO         :: SHADER_INFO;

    GFSDK_Aftermath_GpuCrashDumpDecoderFlags_WARP_STATE_INFO     :: WARP_STATE_INFO;

    GFSDK_Aftermath_GpuCrashDumpDecoderFlags_SHADER_MAPPING_INFO :: SHADER_MAPPING_INFO;

    GFSDK_Aftermath_GpuCrashDumpDecoderFlags_EVENT_MARKER_INFO   :: EVENT_MARKER_INFO;

    GFSDK_Aftermath_GpuCrashDumpDecoderFlags_CALL_STACK_INFO     :: CALL_STACK_INFO;

    GFSDK_Aftermath_GpuCrashDumpDecoderFlags_DESCRIPTION_INFO    :: DESCRIPTION_INFO;

    GFSDK_Aftermath_GpuCrashDumpDecoderFlags_FAULTED_WARP_INFO   :: FAULTED_WARP_INFO;

    GFSDK_Aftermath_GpuCrashDumpDecoderFlags_FINGERPRINT_INFO    :: FINGERPRINT_INFO;

    GFSDK_Aftermath_GpuCrashDumpDecoderFlags_ALL_INFO            :: ALL_INFO;
}
```
### `GFSDK_Aftermath_GpuCrashDumpDescriptionKey`

```jai
GFSDK_Aftermath_GpuCrashDumpDescriptionKey :: enum u32 {
    ApplicationName    :: 1;

    ApplicationVersion :: 2;

    UserDefined        :: 65536;

    GFSDK_Aftermath_GpuCrashDumpDescriptionKey_ApplicationName    :: ApplicationName;

    GFSDK_Aftermath_GpuCrashDumpDescriptionKey_ApplicationVersion :: ApplicationVersion;

    GFSDK_Aftermath_GpuCrashDumpDescriptionKey_UserDefined        :: UserDefined;
}
```
### `GFSDK_Aftermath_GpuCrashDumpFeatureFlags`

```jai
GFSDK_Aftermath_GpuCrashDumpFeatureFlags :: enum u32 {
    Default                 :: 0;

    DeferDebugInfoCallbacks :: 1;

    GFSDK_Aftermath_GpuCrashDumpFeatureFlags_Default                 :: Default;

    GFSDK_Aftermath_GpuCrashDumpFeatureFlags_DeferDebugInfoCallbacks :: DeferDebugInfoCallbacks;
}
```
### `GFSDK_Aftermath_GpuCrashDumpFormatterFlags`

```jai
GFSDK_Aftermath_GpuCrashDumpFormatterFlags :: enum u32 {
    NONE             :: 0;

    CONDENSED_OUTPUT :: 1;

    UTF8_OUTPUT      :: 2;

    GFSDK_Aftermath_GpuCrashDumpFormatterFlags_NONE             :: NONE;

    GFSDK_Aftermath_GpuCrashDumpFormatterFlags_CONDENSED_OUTPUT :: CONDENSED_OUTPUT;

    GFSDK_Aftermath_GpuCrashDumpFormatterFlags_UTF8_OUTPUT      :: UTF8_OUTPUT;
}
```
### `GFSDK_Aftermath_GpuCrashDumpWatchedApiFlags`

```jai
GFSDK_Aftermath_GpuCrashDumpWatchedApiFlags :: enum u32 {
    None   :: 0;

    DX     :: 1;

    Vulkan :: 2;

    GFSDK_Aftermath_GpuCrashDumpWatchedApiFlags_None   :: None;

    GFSDK_Aftermath_GpuCrashDumpWatchedApiFlags_DX     :: DX;

    GFSDK_Aftermath_GpuCrashDumpWatchedApiFlags_Vulkan :: Vulkan;
}
```
### `GFSDK_Aftermath_GpuCrashDump_BaseInfo`

```jai
GFSDK_Aftermath_GpuCrashDump_BaseInfo :: struct {
    applicationName:   [128] u8;
    creationDate:      [128] u8;
    creationTickCount: u32;
    pid:               u32;
    graphicsApi:       GFSDK_Aftermath_GraphicsApi;
}
```
### `GFSDK_Aftermath_GpuCrashDump_Decoder__`

```jai
GFSDK_Aftermath_GpuCrashDump_Decoder__ :: struct {
    ID: s32;
}
```
### `GFSDK_Aftermath_GpuCrashDump_DeviceInfo`

```jai
GFSDK_Aftermath_GpuCrashDump_DeviceInfo :: struct {
    status: GFSDK_Aftermath_Device_Status;
    struct {
        bool_align_adapterReset: u32;

        #overlay (bool_align_adapterReset) adapterReset: bool;
    }
    struct {
        bool_align_engineReset: u32;

        #overlay (bool_align_engineReset) engineReset: bool;
    }
}
```
### `GFSDK_Aftermath_GpuCrashDump_EventMarkerInfo`

```jai
GFSDK_Aftermath_GpuCrashDump_EventMarkerInfo :: struct {
    contextId:           u64;
    contextStatus:       GFSDK_Aftermath_Context_Status;
    contextType:         GFSDK_Aftermath_Context_Type;
    struct {
        markerData: *void;

        #overlay (markerData) ptr_align_markerData: u64;
    }
    markerDataOwnership: GFSDK_Aftermath_EventMarkerDataOwnership;
    markerDataSize:      u32;
}
```
### `GFSDK_Aftermath_GpuCrashDump_GpuInfo`

```jai
GFSDK_Aftermath_GpuCrashDump_GpuInfo :: struct {
    adapterName:    [128] u8;
    generationName: [128] u8;
    adapterLUID:    u64;
}
```
### `GFSDK_Aftermath_GpuCrashDump_PageFaultInfo`

```jai
GFSDK_Aftermath_GpuCrashDump_PageFaultInfo :: struct {
    faultingGpuVA:     u64;
    faultType:         GFSDK_Aftermath_FaultType;
    accessType:        GFSDK_Aftermath_AccessType;
    engine:            GFSDK_Aftermath_Engine;
    client:            GFSDK_Aftermath_Client;
    resourceInfoCount: u32;
}
```
### `GFSDK_Aftermath_GpuCrashDump_ResourceInfo`

```jai
GFSDK_Aftermath_GpuCrashDump_ResourceInfo :: struct {
    gpuVa:                  u64;
    size:                   u64;

    width:                  u32;
    height:                 u32;
    depth:                  u32;
    mipLevels:              u32;

    format:                 u32;

    apiResource:            u64;

    debugName:              [128] u8;

    struct {
        bool_align_bIsBufferHeap: u32;

        #overlay (bool_align_bIsBufferHeap) bIsBufferHeap: bool;
    }
    struct {
        bool_align_bIsStaticTextureHeap: u32;

        #overlay (bool_align_bIsStaticTextureHeap) bIsStaticTextureHeap: bool;
    }
    struct {
        bool_align_bIsRenderTargetOrDepthStencilViewHeap: u32;

        #overlay (bool_align_bIsRenderTargetOrDepthStencilViewHeap) bIsRenderTargetOrDepthStencilViewHeap: bool;
    }
    struct {
        bool_align_bPlacedResource: u32;

        #overlay (bool_align_bPlacedResource) bPlacedResource: bool;
    }

    struct {
        bool_align_bWasDestroyed: u32;

        #overlay (bool_align_bWasDestroyed) bWasDestroyed: bool;
    }
    createDestroyTickCount: u32;
}
```
### `GFSDK_Aftermath_GpuCrashDump_ShaderInfo`

```jai
GFSDK_Aftermath_GpuCrashDump_ShaderInfo :: struct {

    shaderHash:     u64;
    shaderInstance: u64;
    struct {
        bool_align_isInternal: u32;

        #overlay (bool_align_isInternal) isInternal: bool;
    }
    shaderType:     GFSDK_Aftermath_ShaderType;
}
```
### `GFSDK_Aftermath_GpuCrashDump_SystemInfo`

```jai
GFSDK_Aftermath_GpuCrashDump_SystemInfo :: struct {
    osVersion:     [128] u8;
    DisplayDriverVersion :: struct {
        major: u32;
        minor: u32;
    }

    displayDriver: DisplayDriverVersion;
}
```
### `GFSDK_Aftermath_GraphicsApi`

```jai
GFSDK_Aftermath_GraphicsApi :: enum u32 {
    Unknown  :: 0;
    D3D_10_0 :: 1;
    D3D_10_1 :: 2;
    D3D_11_0 :: 3;
    D3D_11_1 :: 4;
    D3D_11_2 :: 5;
    D3D_12_0 :: 6;
    Vulkan   :: 7;

    GFSDK_Aftermath_GraphicsApi_Unknown  :: Unknown;
    GFSDK_Aftermath_GraphicsApi_D3D_10_0 :: D3D_10_0;
    GFSDK_Aftermath_GraphicsApi_D3D_10_1 :: D3D_10_1;
    GFSDK_Aftermath_GraphicsApi_D3D_11_0 :: D3D_11_0;
    GFSDK_Aftermath_GraphicsApi_D3D_11_1 :: D3D_11_1;
    GFSDK_Aftermath_GraphicsApi_D3D_11_2 :: D3D_11_2;
    GFSDK_Aftermath_GraphicsApi_D3D_12_0 :: D3D_12_0;
    GFSDK_Aftermath_GraphicsApi_Vulkan   :: Vulkan;
}
```
### `GFSDK_Aftermath_PageFaultInformation`

```jai
GFSDK_Aftermath_PageFaultInformation :: struct {
    faultingGpuVA: u64;
    resourceDesc:  GFSDK_Aftermath_ResourceDescriptor;
    struct {
        bool_align_bHasPageFaultOccured: u32;

        #overlay (bool_align_bHasPageFaultOccured) bHasPageFaultOccured: bool;
    }
}
```
### `GFSDK_Aftermath_ResourceDescriptor`

```jai
GFSDK_Aftermath_ResourceDescriptor :: struct {
    struct {
        pAppResource: *ID3D12Resource;

        #overlay (pAppResource) ptr_align_pAppResource: u64;
    }

    size:      u64;

    width:     u32;
    height:    u32;
    depth:     u32;

    mipLevels: u32;

    format:    u32;

    struct {
        bool_align_bIsBufferHeap: u32;

        #overlay (bool_align_bIsBufferHeap) bIsBufferHeap: bool;
    }
    struct {
        bool_align_bIsStaticTextureHeap: u32;

        #overlay (bool_align_bIsStaticTextureHeap) bIsStaticTextureHeap: bool;
    }
    struct {
        bool_align_bIsRtvDsvTextureHeap: u32;

        #overlay (bool_align_bIsRtvDsvTextureHeap) bIsRtvDsvTextureHeap: bool;
    }
    struct {
        bool_align_bPlacedResource: u32;

        #overlay (bool_align_bPlacedResource) bPlacedResource: bool;
    }

    struct {
        bool_align_bWasDestroyed: u32;

        #overlay (bool_align_bWasDestroyed) bWasDestroyed: bool;
    }
}
```
### `GFSDK_Aftermath_ResourceHandle__`

```jai
GFSDK_Aftermath_ResourceHandle__ :: struct {
    ID: s32;
}
```
### `GFSDK_Aftermath_Result`

```jai
GFSDK_Aftermath_Result :: enum u32 {
    Success                                   :: 1;

    NotAvailable                              :: 2;

    Fail                                      :: 3134193664;

    FAIL_VersionMismatch                      :: 3134193665;

    FAIL_NotInitialized                       :: 3134193666;

    FAIL_InvalidAdapter                       :: 3134193667;

    FAIL_InvalidParameter                     :: 3134193668;

    FAIL_Unknown                              :: 3134193669;

    FAIL_ApiError                             :: 3134193670;

    FAIL_NvApiIncompatible                    :: 3134193671;

    FAIL_GettingContextDataWithNewCommandList :: 3134193672;

    FAIL_AlreadyInitialized                   :: 3134193673;

    FAIL_D3DDebugLayerNotCompatible           :: 3134193674;

    FAIL_DriverInitFailed                     :: 3134193675;

    FAIL_DriverVersionNotSupported            :: 3134193676;

    FAIL_OutOfMemory                          :: 3134193677;

    FAIL_GetDataOnBundle                      :: 3134193678;

    FAIL_GetDataOnDeferredContext             :: 3134193679;

    FAIL_FeatureNotEnabled                    :: 3134193680;

    FAIL_NoResourcesRegistered                :: 3134193681;

    FAIL_ThisResourceNeverRegistered          :: 3134193682;

    FAIL_NotSupportedInUWP                    :: 3134193683;

    FAIL_D3dDllNotSupported                   :: 3134193684;

    FAIL_D3dDllInterceptionNotSupported       :: 3134193685;

    FAIL_Disabled                             :: 3134193686;

    FAIL_NotSupportedOnContext                :: 3134193687;

    GFSDK_Aftermath_Result_Success                                   :: Success;

    GFSDK_Aftermath_Result_NotAvailable                              :: NotAvailable;

    GFSDK_Aftermath_Result_Fail                                      :: Fail;

    GFSDK_Aftermath_Result_FAIL_VersionMismatch                      :: FAIL_VersionMismatch;

    GFSDK_Aftermath_Result_FAIL_NotInitialized                       :: FAIL_NotInitialized;

    GFSDK_Aftermath_Result_FAIL_InvalidAdapter                       :: FAIL_InvalidAdapter;

    GFSDK_Aftermath_Result_FAIL_InvalidParameter                     :: FAIL_InvalidParameter;

    GFSDK_Aftermath_Result_FAIL_Unknown                              :: FAIL_Unknown;

    GFSDK_Aftermath_Result_FAIL_ApiError                             :: FAIL_ApiError;

    GFSDK_Aftermath_Result_FAIL_NvApiIncompatible                    :: FAIL_NvApiIncompatible;

    GFSDK_Aftermath_Result_FAIL_GettingContextDataWithNewCommandList :: FAIL_GettingContextDataWithNewCommandList;

    GFSDK_Aftermath_Result_FAIL_AlreadyInitialized                   :: FAIL_AlreadyInitialized;

    GFSDK_Aftermath_Result_FAIL_D3DDebugLayerNotCompatible           :: FAIL_D3DDebugLayerNotCompatible;

    GFSDK_Aftermath_Result_FAIL_DriverInitFailed                     :: FAIL_DriverInitFailed;

    GFSDK_Aftermath_Result_FAIL_DriverVersionNotSupported            :: FAIL_DriverVersionNotSupported;

    GFSDK_Aftermath_Result_FAIL_OutOfMemory                          :: FAIL_OutOfMemory;

    GFSDK_Aftermath_Result_FAIL_GetDataOnBundle                      :: FAIL_GetDataOnBundle;

    GFSDK_Aftermath_Result_FAIL_GetDataOnDeferredContext             :: FAIL_GetDataOnDeferredContext;

    GFSDK_Aftermath_Result_FAIL_FeatureNotEnabled                    :: FAIL_FeatureNotEnabled;

    GFSDK_Aftermath_Result_FAIL_NoResourcesRegistered                :: FAIL_NoResourcesRegistered;

    GFSDK_Aftermath_Result_FAIL_ThisResourceNeverRegistered          :: FAIL_ThisResourceNeverRegistered;

    GFSDK_Aftermath_Result_FAIL_NotSupportedInUWP                    :: FAIL_NotSupportedInUWP;

    GFSDK_Aftermath_Result_FAIL_D3dDllNotSupported                   :: FAIL_D3dDllNotSupported;

    GFSDK_Aftermath_Result_FAIL_D3dDllInterceptionNotSupported       :: FAIL_D3dDllInterceptionNotSupported;

    GFSDK_Aftermath_Result_FAIL_Disabled                             :: FAIL_Disabled;

    GFSDK_Aftermath_Result_FAIL_NotSupportedOnContext                :: FAIL_NotSupportedOnContext;
}
```
### `GFSDK_Aftermath_ShaderBinaryHash`

```jai
GFSDK_Aftermath_ShaderBinaryHash :: struct {
    hash: u64;
}
```
### `GFSDK_Aftermath_ShaderDebugInfoIdentifier`

```jai
GFSDK_Aftermath_ShaderDebugInfoIdentifier :: struct {
    id: [2] u64;
}
```
### `GFSDK_Aftermath_ShaderDebugName`

```jai
GFSDK_Aftermath_ShaderDebugName :: struct {
    name: [128] u8;
}
```
### `GFSDK_Aftermath_ShaderType`

```jai
GFSDK_Aftermath_ShaderType :: enum u32 {
    Unknown                  :: 0;
    Vertex                   :: 1;
    Tessellation_Control     :: 2;
    Hull                     :: 2;
    Tessellation_Evaluation  :: 3;
    Domain                   :: 3;
    Geometry                 :: 4;
    Fragment                 :: 5;
    Pixel                    :: 5;
    Compute                  :: 6;
    RayTracing_RayGeneration :: 7;
    RayTracing_Miss          :: 8;
    RayTracing_Intersection  :: 9;
    RayTracing_AnyHit        :: 10;
    RayTracing_ClosestHit    :: 11;
    RayTracing_Callable      :: 12;
    RayTracing_Internal      :: 13;
    Mesh                     :: 14;
    Task                     :: 15;

    GFSDK_Aftermath_ShaderType_Unknown                  :: Unknown;
    GFSDK_Aftermath_ShaderType_Vertex                   :: Vertex;
    GFSDK_Aftermath_ShaderType_Tessellation_Control     :: Tessellation_Control;
    GFSDK_Aftermath_ShaderType_Hull                     :: Hull;
    GFSDK_Aftermath_ShaderType_Tessellation_Evaluation  :: Tessellation_Evaluation;
    GFSDK_Aftermath_ShaderType_Domain                   :: Domain;
    GFSDK_Aftermath_ShaderType_Geometry                 :: Geometry;
    GFSDK_Aftermath_ShaderType_Fragment                 :: Fragment;
    GFSDK_Aftermath_ShaderType_Pixel                    :: Pixel;
    GFSDK_Aftermath_ShaderType_Compute                  :: Compute;
    GFSDK_Aftermath_ShaderType_RayTracing_RayGeneration :: RayTracing_RayGeneration;
    GFSDK_Aftermath_ShaderType_RayTracing_Miss          :: RayTracing_Miss;
    GFSDK_Aftermath_ShaderType_RayTracing_Intersection  :: RayTracing_Intersection;
    GFSDK_Aftermath_ShaderType_RayTracing_AnyHit        :: RayTracing_AnyHit;
    GFSDK_Aftermath_ShaderType_RayTracing_ClosestHit    :: RayTracing_ClosestHit;
    GFSDK_Aftermath_ShaderType_RayTracing_Callable      :: RayTracing_Callable;
    GFSDK_Aftermath_ShaderType_RayTracing_Internal      :: RayTracing_Internal;
    GFSDK_Aftermath_ShaderType_Mesh                     :: Mesh;
    GFSDK_Aftermath_ShaderType_Task                     :: Task;
}
```
### `GFSDK_Aftermath_Version`

```jai
GFSDK_Aftermath_Version :: enum u32 {
    API :: 535;
    GFSDK_Aftermath_Version_API :: API;
}
```
### `anon_enum_3`

```jai
anon_enum_3 :: enum s32 {
    GFSDK_Aftermath_MAX_STRING_LENGTH :: 127;
}
```
## Contracts and constraints

This page is a Windows x64 binding surface for NVIDIA Aftermath GPU crash-dump and shader-debug APIs; it does not wrap device setup or dump decoding into owned Jai abstractions. The library path and available entry points must match the installed NVIDIA SDK/runtime, and callers must follow the vendor API version and handle ownership rules. The binding inventory lists the exposed declarations; SDK-dependent structure layout and callback lifetime need validation against the matching SDK headers.

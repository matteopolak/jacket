# Metal

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `init_metal :: ()`
- `CAMetalDrawable :: struct`
- `CAMetalLayer :: struct`
- `_CAMetalLayerPrivate :: struct`
- `MTLOrigin :: struct`
- `MTLOriginMake :: inline (x: NSUInteger, y: NSUInteger, z: NSUInteger) -> MTLOrigin`
- `MTLSize :: struct`
- `MTLSizeMake :: inline (width: NSUInteger, height: NSUInteger, depth: NSUInteger) -> MTLSize`
- `MTLRegion :: struct`
- `MTLRegionMake1D :: inline (x: NSUInteger, width: NSUInteger) -> MTLRegion`
- `MTLRegionMake2D :: inline (x: NSUInteger, y: NSUInteger, width: NSUInteger, height: NSUInteger) -> MTLRegion`
- `MTLRegionMake3D :: inline (x: NSUInteger, y: NSUInteger, z: NSUInteger, width: NSUInteger, height: NSUInteger, depth: NSUInteger) -> MTLRegion`
- `MTLSamplePosition :: struct`
- `MTLSamplePositionMake :: inline (x: float, y: float) -> MTLSamplePosition`
- `MTLCommandEncoder :: struct`
- `MTLResource :: struct`
- `MTLBuffer :: struct`
- `MTLSharedTextureHandle :: struct`
- `MTLSharedTextureHandlePrivate :: struct`
- `MTLTextureDescriptor :: struct`
- `MTLTexture :: struct`
- `MTLType :: struct`
- `MTLStructMember :: struct`
- `MTLStructType :: struct`
- `MTLArrayType :: struct`
- `MTLPointerType :: struct`
- `MTLTextureReferenceType :: struct`
- `MTLArgument :: struct`
- `MTLAutoreleasedArgument :: *MTLArgument`
- `MTLVertexAttribute :: struct`
- `MTLAttribute :: struct`
- `MTLFunctionConstant :: struct`
- `MTLFunction :: struct`
- `MTLCompileOptions :: struct`
- `MTLLibrary :: struct`
- `MTLCreateSystemDefaultDevice :: () -> id #foreign Metal`
- `MTLCopyAllDevices :: () -> *NSArray(id) #foreign Metal`
- `MTLDeviceNotificationName :: *NSString`
- `MTLRemoveDeviceObserver :: (observer: id) -> () #foreign Metal`
- `MTLSizeAndAlign :: struct`
- `MTLAutoreleasedRenderPipelineReflection :: *MTLRenderPipelineReflection`
- `MTLAutoreleasedComputePipelineReflection :: *MTLComputePipelineReflection`
- `MTLArgumentDescriptor :: struct`
- `MTLDevice :: struct`
- `MTLFence :: struct`
- `MTLBlitCommandEncoder :: struct`
- `MTLCommandBuffer :: struct`
- `MTLDispatchThreadgroupsIndirectArguments :: struct`
- `MTLStageInRegionIndirectArguments :: struct`
- `MTLComputeCommandEncoder :: struct`
- `MTLCommandQueue :: struct`
- `MTLStencilDescriptor :: struct`
- `MTLDepthStencilDescriptor :: struct`
- `MTLDepthStencilState :: struct`
- `MTLDrawable :: struct`
- `MTLClearColor :: struct`
- `MTLRenderPassAttachmentDescriptor :: struct`
- `MTLRenderPassColorAttachmentDescriptor :: struct`
- `MTLRenderPassDepthAttachmentDescriptor :: struct`
- `MTLRenderPassStencilAttachmentDescriptor :: struct`
- `MTLRenderPassColorAttachmentDescriptorArray :: struct`
- `MTLRenderPassDescriptor :: struct`
- `MTLClearColorMake :: (red: float64, green: float64, blue: float64, alpha: float64) -> MTLClearColor`
- `MTLBufferLayoutDescriptor :: struct`
- `MTLBufferLayoutDescriptorArray :: struct`
- `MTLAttributeDescriptor :: struct`
- `MTLAttributeDescriptorArray :: struct`
- `MTLStageInputOutputDescriptor :: struct`
- `MTLPipelineBufferDescriptor :: struct`
- `MTLPipelineBufferDescriptorArray :: struct`
- `MTLComputePipelineReflection :: struct`
- `MTLComputePipelineDescriptor :: struct`
- `MTLComputePipelineState :: struct`
- `MTLScissorRect :: struct`
- `MTLViewport :: struct`
- `MTLDrawPrimitivesIndirectArguments :: struct`
- `MTLDrawIndexedPrimitivesIndirectArguments :: struct`
- `MTLDrawPatchIndirectArguments :: struct`
- `MTLQuadTessellationFactorsHalf :: struct`
- `MTLTriangleTessellationFactorsHalf :: struct`
- `MTLRenderCommandEncoder :: struct`
- `MTLFunctionConstantValues :: struct`
- `MTLRenderPipelineColorAttachmentDescriptor :: struct`
- `MTLRenderPipelineReflection :: struct`
- `MTLRenderPipelineDescriptor :: struct`
- `MTLRenderPipelineState :: struct`
- `MTLRenderPipelineColorAttachmentDescriptorArray :: struct`
- `MTLVertexBufferLayoutDescriptor :: struct`
- `MTLVertexBufferLayoutDescriptorArray :: struct`
- `MTLVertexAttributeDescriptor :: struct`
- `MTLVertexAttributeDescriptorArray :: struct`
- `MTLVertexDescriptor :: struct`
- `MTLParallelRenderCommandEncoder :: struct`
- `MTLSamplerDescriptor :: struct`
- `MTLSamplerState :: struct`
- `MTLHeapDescriptor :: struct`
- `MTLHeap :: struct`
- `MTLArgumentEncoder :: struct`
- `MTLCaptureManager :: struct`
- `MTLCaptureScope :: struct`
- `MTLIndirectRenderCommand :: struct`
- `MTLIndirectCommandBufferDescriptor :: struct`
- `MTLIndirectCommandBuffer :: struct`
- `MTLEvent :: struct`
- `MTLSharedEventListener :: struct`
- `MTLSharedEvent :: struct`
- `MTLSharedEventHandle :: struct`
- `MTLSharedEventHandlePrivate :: struct`

- `using MTLResourceUsage :: enum NSUInteger`
- `using MTLBarrierScope :: enum NSUInteger`
- `using MTLPixelFormat :: enum NSUInteger`
- `using MTLPurgeableState :: enum NSUInteger`
- `using MTLCPUCacheMode :: enum NSUInteger`
- `using MTLStorageMode :: enum NSUInteger`
- `using MTLResourceOptions :: enum NSUInteger`
- `using MTLTextureType :: enum NSUInteger`
- `using MTLTextureUsage :: enum NSUInteger`
- `using MTLDataType :: enum NSUInteger`
- `using MTLArgumentType :: enum NSUInteger`
- `using MTLArgumentAccess :: enum NSUInteger`
- `using MTLPatchType :: enum NSUInteger`
- `using MTLFunctionType :: enum NSUInteger`
- `using MTLLanguageVersion :: enum NSUInteger`
- `using MTLLibraryError :: enum NSUInteger`
- `using MTLFeatureSet :: enum NSUInteger`
- `using MTLPipelineOption :: enum NSUInteger`
- `using MTLReadWriteTextureTier :: enum NSUInteger`
- `using MTLArgumentBuffersTier :: enum NSUInteger`
- `using MTLBlitOption :: enum NSUInteger`
- `using MTLCommandBufferStatus :: enum NSUInteger`
- `using MTLCommandBufferError :: enum NSUInteger`
- `using MTLDispatchType :: enum NSUInteger`
- `using MTLCompareFunction :: enum NSUInteger`
- `using MTLStencilOperation :: enum NSUInteger`
- `using MTLLoadAction :: enum NSUInteger`
- `using MTLStoreAction :: enum NSUInteger`
- `using MTLStoreActionOptions :: enum NSUInteger`
- `using MTLMultisampleDepthResolveFilter :: enum NSUInteger`
- `using MTLMultisampleStencilResolveFilter :: enum NSUInteger`
- `using MTLAttributeFormat :: enum NSUInteger`
- `using MTLIndexType :: enum NSUInteger`
- `using MTLStepFunction :: enum NSUInteger`
- `using MTLMutability :: enum NSUInteger`
- `using MTLPrimitiveType :: enum NSUInteger`
- `using MTLVisibilityResultMode :: enum NSUInteger`
- `using MTLCullMode :: enum NSUInteger`
- `using MTLWinding :: enum NSUInteger`
- `using MTLDepthClipMode :: enum NSUInteger`
- `using MTLTriangleFillMode :: enum NSUInteger`
- `using MTLRenderStages :: enum NSUInteger`
- `using MTLBlendFactor :: enum NSUInteger`
- `using MTLBlendOperation :: enum NSUInteger`
- `using MTLColorWriteMask :: enum NSUInteger`
- `using MTLPrimitiveTopologyClass :: enum NSUInteger`
- `using MTLTessellationPartitionMode :: enum NSUInteger`
- `using MTLTessellationFactorStepFunction :: enum NSUInteger`
- `using MTLTessellationFactorFormat :: enum NSUInteger`
- `using MTLTessellationControlPointIndexType :: enum NSUInteger`
- `using MTLVertexFormat :: enum NSUInteger`
- `using MTLVertexStepFunction :: enum NSUInteger`
- `using MTLSamplerMinMagFilter :: enum NSUInteger`
- `using MTLSamplerMipFilter :: enum NSUInteger`
- `using MTLSamplerAddressMode :: enum NSUInteger`
- `using MTLSamplerBorderColor :: enum NSUInteger`
- `using MTLIndirectCommandType :: enum NSUInteger`

- `MTLCommandBufferErrorDomain: *NSString`
- `MTLDeviceRemovalRequestedNotification: MTLDeviceNotificationName`
- `MTLDeviceWasAddedNotification: MTLDeviceNotificationName`
- `MTLDeviceWasRemovedNotification: MTLDeviceNotificationName`
- `MTLLibraryErrorDomain: *NSString`

## Public type members

Metal enum declarations are imported as `NSUInteger` enums on the macOS target. Their named members and values are listed here.

### Encoder wrapper field relationships

These declarations describe Jai's public wrapper-member surface, not native
Objective-C instance storage. `MTLCommandEncoder` declares seven method
members and no stored fields, `using` base, or `NSObject` embedding. Of the
four derived encoder wrappers, the module declares exactly one base field:

```jai
MTLRenderCommandEncoder :: struct {
    using #as mtlcommandencder: MTLCommandEncoder;
    // Additional method members are listed in this page.
}
```

The field name is exactly `mtlcommandencder` (as spelled in the declaration);
it is a `using #as` embedding of `MTLCommandEncoder`, with no explicit offset
or padding directive. `MTLBlitCommandEncoder`, `MTLComputeCommandEncoder`, and
`MTLParallelRenderCommandEncoder` declare no `MTLCommandEncoder` base field,
`NSObject` field, or other stored fields. They contain their listed message
members only. `MTLCommandEncoder` likewise declares no `NSObject` field.
Although the native SDK protocols inherit from `NSObject` and the derived
encoder protocols inherit from `MTLCommandEncoder`, that native protocol
relationship does not create additional Jai `using` fields. The Jai wrapper
expresses the inheritance edge only for `MTLRenderCommandEncoder`; do not
silently add that field to the other wrappers when reproducing this public
surface.

The three private placeholder records below are declared with empty bodies; they have no public fields.

#### `_CAMetalLayerPrivate` (empty struct)

#### `MTLSharedTextureHandlePrivate` (empty struct)

#### `MTLSharedEventHandlePrivate` (empty struct)

#### `MTLResourceUsage` (enum `NSUInteger`)

- `MTLResourceUsageRead :: 1`
- `MTLResourceUsageWrite :: 2`
- `MTLResourceUsageSample :: 4`

#### `MTLBarrierScope` (enum `NSUInteger`)

- `MTLBarrierScopeBuffers :: 1`
- `MTLBarrierScopeTextures :: 2`
- `MTLBarrierScopeRenderTargets :: 4`

#### `MTLPixelFormat` (enum `NSUInteger`)

- `MTLPixelFormatInvalid :: 0`
- `MTLPixelFormatA8Unorm :: 1`
- `MTLPixelFormatR8Unorm :: 10`
- `MTLPixelFormatR8Unorm_sRGB :: 11`
- `MTLPixelFormatR8Snorm :: 12`
- `MTLPixelFormatR8Uint :: 13`
- `MTLPixelFormatR8Sint :: 14`
- `MTLPixelFormatR16Unorm :: 20`
- `MTLPixelFormatR16Snorm :: 22`
- `MTLPixelFormatR16Uint :: 23`
- `MTLPixelFormatR16Sint :: 24`
- `MTLPixelFormatR16Float :: 25`
- `MTLPixelFormatRG8Unorm :: 30`
- `MTLPixelFormatRG8Unorm_sRGB :: 31`
- `MTLPixelFormatRG8Snorm :: 32`
- `MTLPixelFormatRG8Uint :: 33`
- `MTLPixelFormatRG8Sint :: 34`
- `MTLPixelFormatB5G6R5Unorm :: 40`
- `MTLPixelFormatA1BGR5Unorm :: 41`
- `MTLPixelFormatABGR4Unorm :: 42`
- `MTLPixelFormatBGR5A1Unorm :: 43`
- `MTLPixelFormatR32Uint :: 53`
- `MTLPixelFormatR32Sint :: 54`
- `MTLPixelFormatR32Float :: 55`
- `MTLPixelFormatRG16Unorm :: 60`
- `MTLPixelFormatRG16Snorm :: 62`
- `MTLPixelFormatRG16Uint :: 63`
- `MTLPixelFormatRG16Sint :: 64`
- `MTLPixelFormatRG16Float :: 65`
- `MTLPixelFormatRGBA8Unorm :: 70`
- `MTLPixelFormatRGBA8Unorm_sRGB :: 71`
- `MTLPixelFormatRGBA8Snorm :: 72`
- `MTLPixelFormatRGBA8Uint :: 73`
- `MTLPixelFormatRGBA8Sint :: 74`
- `MTLPixelFormatBGRA8Unorm :: 80`
- `MTLPixelFormatBGRA8Unorm_sRGB :: 81`
- `MTLPixelFormatRGB10A2Unorm :: 90`
- `MTLPixelFormatRGB10A2Uint :: 91`
- `MTLPixelFormatRG11B10Float :: 92`
- `MTLPixelFormatRGB9E5Float :: 93`
- `MTLPixelFormatBGR10A2Unorm :: 94`
- `MTLPixelFormatBGR10_XR :: 554`
- `MTLPixelFormatBGR10_XR_sRGB :: 555`
- `MTLPixelFormatRG32Uint :: 103`
- `MTLPixelFormatRG32Sint :: 104`
- `MTLPixelFormatRG32Float :: 105`
- `MTLPixelFormatRGBA16Unorm :: 110`
- `MTLPixelFormatRGBA16Snorm :: 112`
- `MTLPixelFormatRGBA16Uint :: 113`
- `MTLPixelFormatRGBA16Sint :: 114`
- `MTLPixelFormatRGBA16Float :: 115`
- `MTLPixelFormatBGRA10_XR :: 552`
- `MTLPixelFormatBGRA10_XR_sRGB :: 553`
- `MTLPixelFormatRGBA32Uint :: 123`
- `MTLPixelFormatRGBA32Sint :: 124`
- `MTLPixelFormatRGBA32Float :: 125`
- `MTLPixelFormatBC1_RGBA :: 130`
- `MTLPixelFormatBC1_RGBA_sRGB :: 131`
- `MTLPixelFormatBC2_RGBA :: 132`
- `MTLPixelFormatBC2_RGBA_sRGB :: 133`
- `MTLPixelFormatBC3_RGBA :: 134`
- `MTLPixelFormatBC3_RGBA_sRGB :: 135`
- `MTLPixelFormatBC4_RUnorm :: 140`
- `MTLPixelFormatBC4_RSnorm :: 141`
- `MTLPixelFormatBC5_RGUnorm :: 142`
- `MTLPixelFormatBC5_RGSnorm :: 143`
- `MTLPixelFormatBC6H_RGBFloat :: 150`
- `MTLPixelFormatBC6H_RGBUfloat :: 151`
- `MTLPixelFormatBC7_RGBAUnorm :: 152`
- `MTLPixelFormatBC7_RGBAUnorm_sRGB :: 153`
- `MTLPixelFormatPVRTC_RGB_2BPP :: 160`
- `MTLPixelFormatPVRTC_RGB_2BPP_sRGB :: 161`
- `MTLPixelFormatPVRTC_RGB_4BPP :: 162`
- `MTLPixelFormatPVRTC_RGB_4BPP_sRGB :: 163`
- `MTLPixelFormatPVRTC_RGBA_2BPP :: 164`
- `MTLPixelFormatPVRTC_RGBA_2BPP_sRGB :: 165`
- `MTLPixelFormatPVRTC_RGBA_4BPP :: 166`
- `MTLPixelFormatPVRTC_RGBA_4BPP_sRGB :: 167`
- `MTLPixelFormatEAC_R11Unorm :: 170`
- `MTLPixelFormatEAC_R11Snorm :: 172`
- `MTLPixelFormatEAC_RG11Unorm :: 174`
- `MTLPixelFormatEAC_RG11Snorm :: 176`
- `MTLPixelFormatEAC_RGBA8 :: 178`
- `MTLPixelFormatEAC_RGBA8_sRGB :: 179`
- `MTLPixelFormatETC2_RGB8 :: 180`
- `MTLPixelFormatETC2_RGB8_sRGB :: 181`
- `MTLPixelFormatETC2_RGB8A1 :: 182`
- `MTLPixelFormatETC2_RGB8A1_sRGB :: 183`
- `MTLPixelFormatASTC_4x4_sRGB :: 186`
- `MTLPixelFormatASTC_5x4_sRGB :: 187`
- `MTLPixelFormatASTC_5x5_sRGB :: 188`
- `MTLPixelFormatASTC_6x5_sRGB :: 189`
- `MTLPixelFormatASTC_6x6_sRGB :: 190`
- `MTLPixelFormatASTC_8x5_sRGB :: 192`
- `MTLPixelFormatASTC_8x6_sRGB :: 193`
- `MTLPixelFormatASTC_8x8_sRGB :: 194`
- `MTLPixelFormatASTC_10x5_sRGB :: 195`
- `MTLPixelFormatASTC_10x6_sRGB :: 196`
- `MTLPixelFormatASTC_10x8_sRGB :: 197`
- `MTLPixelFormatASTC_10x10_sRGB :: 198`
- `MTLPixelFormatASTC_12x10_sRGB :: 199`
- `MTLPixelFormatASTC_12x12_sRGB :: 200`
- `MTLPixelFormatASTC_4x4_LDR :: 204`
- `MTLPixelFormatASTC_5x4_LDR :: 205`
- `MTLPixelFormatASTC_5x5_LDR :: 206`
- `MTLPixelFormatASTC_6x5_LDR :: 207`
- `MTLPixelFormatASTC_6x6_LDR :: 208`
- `MTLPixelFormatASTC_8x5_LDR :: 210`
- `MTLPixelFormatASTC_8x6_LDR :: 211`
- `MTLPixelFormatASTC_8x8_LDR :: 212`
- `MTLPixelFormatASTC_10x5_LDR :: 213`
- `MTLPixelFormatASTC_10x6_LDR :: 214`
- `MTLPixelFormatASTC_10x8_LDR :: 215`
- `MTLPixelFormatASTC_10x10_LDR :: 216`
- `MTLPixelFormatASTC_12x10_LDR :: 217`
- `MTLPixelFormatASTC_12x12_LDR :: 218`
- `MTLPixelFormatGBGR422 :: 240`
- `MTLPixelFormatBGRG422 :: 241`
- `MTLPixelFormatDepth16Unorm :: 250`
- `MTLPixelFormatDepth32Float :: 252`
- `MTLPixelFormatStencil8 :: 253`
- `MTLPixelFormatDepth24Unorm_Stencil8 :: 255`
- `MTLPixelFormatDepth32Float_Stencil8 :: 260`
- `MTLPixelFormatX32_Stencil8 :: 261`
- `MTLPixelFormatX24_Stencil8 :: 262`

#### `MTLPurgeableState` (enum `NSUInteger`)

- `MTLPurgeableStateKeepCurrent :: 1`
- `MTLPurgeableStateNonVolatile :: 2`
- `MTLPurgeableStateVolatile :: 3`
- `MTLPurgeableStateEmpty :: 4`

#### `MTLCPUCacheMode` (enum `NSUInteger`)

- `MTLCPUCacheModeDefaultCache :: 0`
- `MTLCPUCacheModeWriteCombined :: 1`

#### `MTLStorageMode` (enum `NSUInteger`)

- `MTLStorageModeShared :: 0`
- `MTLStorageModeManaged :: 1`
- `MTLStorageModePrivate :: 2`
- `MTLStorageModeMemoryless :: 3`

#### `MTLResourceOptions` (enum `NSUInteger`)

- `MTLResourceCPUCacheModeDefaultCache :: 0`
- `MTLResourceCPUCacheModeWriteCombined :: 1`
- `MTLResourceStorageModeShared :: 0`
- `MTLResourceStorageModeManaged :: 16`
- `MTLResourceStorageModePrivate :: 32`
- `MTLResourceStorageModeMemoryless :: 48`
- `MTLResourceHazardTrackingModeUntracked :: 256`
- `MTLResourceOptionCPUCacheModeDefault :: 0`
- `MTLResourceOptionCPUCacheModeWriteCombined :: 1`

#### `MTLTextureType` (enum `NSUInteger`)

- `MTLTextureType1D :: 0`
- `MTLTextureType1DArray :: 1`
- `MTLTextureType2D :: 2`
- `MTLTextureType2DArray :: 3`
- `MTLTextureType2DMultisample :: 4`
- `MTLTextureTypeCube :: 5`
- `MTLTextureTypeCubeArray :: 6`
- `MTLTextureType3D :: 7`
- `MTLTextureType2DMultisampleArray :: 8`
- `MTLTextureTypeTextureBuffer :: 9`

#### `MTLTextureUsage` (enum `NSUInteger`)

- `MTLTextureUsageUnknown :: 0`
- `MTLTextureUsageShaderRead :: 1`
- `MTLTextureUsageShaderWrite :: 2`
- `MTLTextureUsageRenderTarget :: 4`
- `MTLTextureUsagePixelFormatView :: 16`

#### `MTLDataType` (enum `NSUInteger`)

- `MTLDataTypeNone :: 0`
- `MTLDataTypeStruct :: 1`
- `MTLDataTypeArray :: 2`
- `MTLDataTypeFloat :: 3`
- `MTLDataTypeFloat2 :: 4`
- `MTLDataTypeFloat3 :: 5`
- `MTLDataTypeFloat4 :: 6`
- `MTLDataTypeFloat2x2 :: 7`
- `MTLDataTypeFloat2x3 :: 8`
- `MTLDataTypeFloat2x4 :: 9`
- `MTLDataTypeFloat3x2 :: 10`
- `MTLDataTypeFloat3x3 :: 11`
- `MTLDataTypeFloat3x4 :: 12`
- `MTLDataTypeFloat4x2 :: 13`
- `MTLDataTypeFloat4x3 :: 14`
- `MTLDataTypeFloat4x4 :: 15`
- `MTLDataTypeHalf :: 16`
- `MTLDataTypeHalf2 :: 17`
- `MTLDataTypeHalf3 :: 18`
- `MTLDataTypeHalf4 :: 19`
- `MTLDataTypeHalf2x2 :: 20`
- `MTLDataTypeHalf2x3 :: 21`
- `MTLDataTypeHalf2x4 :: 22`
- `MTLDataTypeHalf3x2 :: 23`
- `MTLDataTypeHalf3x3 :: 24`
- `MTLDataTypeHalf3x4 :: 25`
- `MTLDataTypeHalf4x2 :: 26`
- `MTLDataTypeHalf4x3 :: 27`
- `MTLDataTypeHalf4x4 :: 28`
- `MTLDataTypeInt :: 29`
- `MTLDataTypeInt2 :: 30`
- `MTLDataTypeInt3 :: 31`
- `MTLDataTypeInt4 :: 32`
- `MTLDataTypeUInt :: 33`
- `MTLDataTypeUInt2 :: 34`
- `MTLDataTypeUInt3 :: 35`
- `MTLDataTypeUInt4 :: 36`
- `MTLDataTypeShort :: 37`
- `MTLDataTypeShort2 :: 38`
- `MTLDataTypeShort3 :: 39`
- `MTLDataTypeShort4 :: 40`
- `MTLDataTypeUShort :: 41`
- `MTLDataTypeUShort2 :: 42`
- `MTLDataTypeUShort3 :: 43`
- `MTLDataTypeUShort4 :: 44`
- `MTLDataTypeChar :: 45`
- `MTLDataTypeChar2 :: 46`
- `MTLDataTypeChar3 :: 47`
- `MTLDataTypeChar4 :: 48`
- `MTLDataTypeUChar :: 49`
- `MTLDataTypeUChar2 :: 50`
- `MTLDataTypeUChar3 :: 51`
- `MTLDataTypeUChar4 :: 52`
- `MTLDataTypeBool :: 53`
- `MTLDataTypeBool2 :: 54`
- `MTLDataTypeBool3 :: 55`
- `MTLDataTypeBool4 :: 56`
- `MTLDataTypeTexture :: 58`
- `MTLDataTypeSampler :: 59`
- `MTLDataTypePointer :: 60`
- `MTLDataTypeRenderPipeline :: 78`
- `MTLDataTypeIndirectCommandBuffer :: 80`

#### `MTLArgumentType` (enum `NSUInteger`)

- `MTLArgumentTypeBuffer :: 0`
- `MTLArgumentTypeThreadgroupMemory :: 1`
- `MTLArgumentTypeTexture :: 2`
- `MTLArgumentTypeSampler :: 3`

#### `MTLArgumentAccess` (enum `NSUInteger`)

- `MTLArgumentAccessReadOnly :: 0`
- `MTLArgumentAccessReadWrite :: 1`
- `MTLArgumentAccessWriteOnly :: 2`

#### `MTLPatchType` (enum `NSUInteger`)

- `MTLPatchTypeNone :: 0`
- `MTLPatchTypeTriangle :: 1`
- `MTLPatchTypeQuad :: 2`

#### `MTLFunctionType` (enum `NSUInteger`)

- `MTLFunctionTypeVertex :: 1`
- `MTLFunctionTypeFragment :: 2`
- `MTLFunctionTypeKernel :: 3`

#### `MTLLanguageVersion` (enum `NSUInteger`)

- `MTLLanguageVersion1_0 :: 65536`
- `MTLLanguageVersion1_1 :: 65537`
- `MTLLanguageVersion1_2 :: 65538`
- `MTLLanguageVersion2_0 :: 131072`
- `MTLLanguageVersion2_1 :: 131073`

#### `MTLLibraryError` (enum `NSUInteger`)

- `MTLLibraryErrorUnsupported :: 1`
- `MTLLibraryErrorInternal :: 2`
- `MTLLibraryErrorCompileFailure :: 3`
- `MTLLibraryErrorCompileWarning :: 4`
- `MTLLibraryErrorFunctionNotFound :: 5`
- `MTLLibraryErrorFileNotFound :: 6`

#### `MTLFeatureSet` (enum `NSUInteger`)

- `MTLFeatureSet_iOS_GPUFamily1_v1 :: 0`
- `MTLFeatureSet_iOS_GPUFamily2_v1 :: 1`
- `MTLFeatureSet_iOS_GPUFamily1_v2 :: 2`
- `MTLFeatureSet_iOS_GPUFamily2_v2 :: 3`
- `MTLFeatureSet_iOS_GPUFamily3_v1 :: 4`
- `MTLFeatureSet_iOS_GPUFamily1_v3 :: 5`
- `MTLFeatureSet_iOS_GPUFamily2_v3 :: 6`
- `MTLFeatureSet_iOS_GPUFamily3_v2 :: 7`
- `MTLFeatureSet_iOS_GPUFamily1_v4 :: 8`
- `MTLFeatureSet_iOS_GPUFamily2_v4 :: 9`
- `MTLFeatureSet_iOS_GPUFamily3_v3 :: 10`
- `MTLFeatureSet_iOS_GPUFamily4_v1 :: 11`
- `MTLFeatureSet_iOS_GPUFamily1_v5 :: 12`
- `MTLFeatureSet_iOS_GPUFamily2_v5 :: 13`
- `MTLFeatureSet_iOS_GPUFamily3_v4 :: 14`
- `MTLFeatureSet_iOS_GPUFamily4_v2 :: 15`
- `MTLFeatureSet_macOS_GPUFamily1_v1 :: 10000`
- `MTLFeatureSet_OSX_GPUFamily1_v1 :: 10000`
- `MTLFeatureSet_macOS_GPUFamily1_v2 :: 10001`
- `MTLFeatureSet_OSX_GPUFamily1_v2 :: 10001`
- `MTLFeatureSet_macOS_ReadWriteTextureTier2 :: 10002`
- `MTLFeatureSet_OSX_ReadWriteTextureTier2 :: 10002`
- `MTLFeatureSet_macOS_GPUFamily1_v3 :: 10003`
- `MTLFeatureSet_macOS_GPUFamily1_v4 :: 10004`
- `MTLFeatureSet_macOS_GPUFamily2_v1 :: 10005`
- `MTLFeatureSet_tvOS_GPUFamily1_v1 :: 30000`
- `MTLFeatureSet_TVOS_GPUFamily1_v1 :: 30000`
- `MTLFeatureSet_tvOS_GPUFamily1_v2 :: 30001`
- `MTLFeatureSet_tvOS_GPUFamily1_v3 :: 30002`
- `MTLFeatureSet_tvOS_GPUFamily1_v4 :: 30004`

#### `MTLPipelineOption` (enum `NSUInteger`)

- `MTLPipelineOptionNone :: 0`
- `MTLPipelineOptionArgumentInfo :: 1`
- `MTLPipelineOptionBufferTypeInfo :: 2`

#### `MTLReadWriteTextureTier` (enum `NSUInteger`)

- `MTLReadWriteTextureTierNone :: 0`
- `MTLReadWriteTextureTier1 :: 1`
- `MTLReadWriteTextureTier2 :: 2`

#### `MTLArgumentBuffersTier` (enum `NSUInteger`)

- `MTLArgumentBuffersTier1 :: 0`
- `MTLArgumentBuffersTier2 :: 1`

#### `MTLBlitOption` (enum `NSUInteger`)

- `MTLBlitOptionNone :: 0`
- `MTLBlitOptionDepthFromDepthStencil :: 1`
- `MTLBlitOptionStencilFromDepthStencil :: 2`
- `MTLBlitOptionRowLinearPVRTC :: 4`

#### `MTLCommandBufferStatus` (enum `NSUInteger`)

- `MTLCommandBufferStatusNotEnqueued :: 0`
- `MTLCommandBufferStatusEnqueued :: 1`
- `MTLCommandBufferStatusCommitted :: 2`
- `MTLCommandBufferStatusScheduled :: 3`
- `MTLCommandBufferStatusCompleted :: 4`
- `MTLCommandBufferStatusError :: 5`

#### `MTLCommandBufferError` (enum `NSUInteger`)

- `MTLCommandBufferErrorNone :: 0`
- `MTLCommandBufferErrorInternal :: 1`
- `MTLCommandBufferErrorTimeout :: 2`
- `MTLCommandBufferErrorPageFault :: 3`
- `MTLCommandBufferErrorBlacklisted :: 4`
- `MTLCommandBufferErrorNotPermitted :: 7`
- `MTLCommandBufferErrorOutOfMemory :: 8`
- `MTLCommandBufferErrorInvalidResource :: 9`
- `MTLCommandBufferErrorMemoryless :: 10`
- `MTLCommandBufferErrorDeviceRemoved :: 11`

#### `MTLDispatchType` (enum `NSUInteger`)

- `MTLDispatchTypeSerial :: 0`
- `MTLDispatchTypeConcurrent :: 1`

#### `MTLCompareFunction` (enum `NSUInteger`)

- `MTLCompareFunctionNever :: 0`
- `MTLCompareFunctionLess :: 1`
- `MTLCompareFunctionEqual :: 2`
- `MTLCompareFunctionLessEqual :: 3`
- `MTLCompareFunctionGreater :: 4`
- `MTLCompareFunctionNotEqual :: 5`
- `MTLCompareFunctionGreaterEqual :: 6`
- `MTLCompareFunctionAlways :: 7`

#### `MTLStencilOperation` (enum `NSUInteger`)

- `MTLStencilOperationKeep :: 0`
- `MTLStencilOperationZero :: 1`
- `MTLStencilOperationReplace :: 2`
- `MTLStencilOperationIncrementClamp :: 3`
- `MTLStencilOperationDecrementClamp :: 4`
- `MTLStencilOperationInvert :: 5`
- `MTLStencilOperationIncrementWrap :: 6`
- `MTLStencilOperationDecrementWrap :: 7`

#### `MTLLoadAction` (enum `NSUInteger`)

- `MTLLoadActionDontCare :: 0`
- `MTLLoadActionLoad :: 1`
- `MTLLoadActionClear :: 2`

#### `MTLStoreAction` (enum `NSUInteger`)

- `MTLStoreActionDontCare :: 0`
- `MTLStoreActionStore :: 1`
- `MTLStoreActionMultisampleResolve :: 2`
- `MTLStoreActionStoreAndMultisampleResolve :: 3`
- `MTLStoreActionUnknown :: 4`
- `MTLStoreActionCustomSampleDepthStore :: 5`

#### `MTLStoreActionOptions` (enum `NSUInteger`)

- `MTLStoreActionOptionNone :: 0`
- `MTLStoreActionOptionCustomSamplePositions :: 1`

#### `MTLMultisampleDepthResolveFilter` (enum `NSUInteger`)

- `MTLMultisampleDepthResolveFilterSample0 :: 0`
- `MTLMultisampleDepthResolveFilterMin :: 1`
- `MTLMultisampleDepthResolveFilterMax :: 2`

#### `MTLMultisampleStencilResolveFilter` (enum `NSUInteger`)

- `MTLMultisampleStencilResolveFilterSample0 :: 0`
- `MTLMultisampleStencilResolveFilterDepthResolvedSample :: 1`

#### `MTLAttributeFormat` (enum `NSUInteger`)

- `MTLAttributeFormatInvalid :: 0`
- `MTLAttributeFormatUChar2 :: 1`
- `MTLAttributeFormatUChar3 :: 2`
- `MTLAttributeFormatUChar4 :: 3`
- `MTLAttributeFormatChar2 :: 4`
- `MTLAttributeFormatChar3 :: 5`
- `MTLAttributeFormatChar4 :: 6`
- `MTLAttributeFormatUChar2Normalized :: 7`
- `MTLAttributeFormatUChar3Normalized :: 8`
- `MTLAttributeFormatUChar4Normalized :: 9`
- `MTLAttributeFormatChar2Normalized :: 10`
- `MTLAttributeFormatChar3Normalized :: 11`
- `MTLAttributeFormatChar4Normalized :: 12`
- `MTLAttributeFormatUShort2 :: 13`
- `MTLAttributeFormatUShort3 :: 14`
- `MTLAttributeFormatUShort4 :: 15`
- `MTLAttributeFormatShort2 :: 16`
- `MTLAttributeFormatShort3 :: 17`
- `MTLAttributeFormatShort4 :: 18`
- `MTLAttributeFormatUShort2Normalized :: 19`
- `MTLAttributeFormatUShort3Normalized :: 20`
- `MTLAttributeFormatUShort4Normalized :: 21`
- `MTLAttributeFormatShort2Normalized :: 22`
- `MTLAttributeFormatShort3Normalized :: 23`
- `MTLAttributeFormatShort4Normalized :: 24`
- `MTLAttributeFormatHalf2 :: 25`
- `MTLAttributeFormatHalf3 :: 26`
- `MTLAttributeFormatHalf4 :: 27`
- `MTLAttributeFormatFloat :: 28`
- `MTLAttributeFormatFloat2 :: 29`
- `MTLAttributeFormatFloat3 :: 30`
- `MTLAttributeFormatFloat4 :: 31`
- `MTLAttributeFormatInt :: 32`
- `MTLAttributeFormatInt2 :: 33`
- `MTLAttributeFormatInt3 :: 34`
- `MTLAttributeFormatInt4 :: 35`
- `MTLAttributeFormatUInt :: 36`
- `MTLAttributeFormatUInt2 :: 37`
- `MTLAttributeFormatUInt3 :: 38`
- `MTLAttributeFormatUInt4 :: 39`
- `MTLAttributeFormatInt1010102Normalized :: 40`
- `MTLAttributeFormatUInt1010102Normalized :: 41`
- `MTLAttributeFormatUChar4Normalized_BGRA :: 42`
- `MTLAttributeFormatUChar :: 45`
- `MTLAttributeFormatChar :: 46`
- `MTLAttributeFormatUCharNormalized :: 47`
- `MTLAttributeFormatCharNormalized :: 48`
- `MTLAttributeFormatUShort :: 49`
- `MTLAttributeFormatShort :: 50`
- `MTLAttributeFormatUShortNormalized :: 51`
- `MTLAttributeFormatShortNormalized :: 52`
- `MTLAttributeFormatHalf :: 53`

#### `MTLIndexType` (enum `NSUInteger`)

- `MTLIndexTypeUInt16 :: 0`
- `MTLIndexTypeUInt32 :: 1`

#### `MTLStepFunction` (enum `NSUInteger`)

- `MTLStepFunctionConstant :: 0`
- `MTLStepFunctionPerVertex :: 1`
- `MTLStepFunctionPerInstance :: 2`
- `MTLStepFunctionPerPatch :: 3`
- `MTLStepFunctionPerPatchControlPoint :: 4`
- `MTLStepFunctionThreadPositionInGridX :: 5`
- `MTLStepFunctionThreadPositionInGridY :: 6`
- `MTLStepFunctionThreadPositionInGridXIndexed :: 7`
- `MTLStepFunctionThreadPositionInGridYIndexed :: 8`

#### `MTLMutability` (enum `NSUInteger`)

- `MTLMutabilityDefault :: 0`
- `MTLMutabilityMutable :: 1`
- `MTLMutabilityImmutable :: 2`

#### `MTLPrimitiveType` (enum `NSUInteger`)

- `MTLPrimitiveTypePoint :: 0`
- `MTLPrimitiveTypeLine :: 1`
- `MTLPrimitiveTypeLineStrip :: 2`
- `MTLPrimitiveTypeTriangle :: 3`
- `MTLPrimitiveTypeTriangleStrip :: 4`

#### `MTLVisibilityResultMode` (enum `NSUInteger`)

- `MTLVisibilityResultModeDisabled :: 0`
- `MTLVisibilityResultModeBoolean :: 1`
- `MTLVisibilityResultModeCounting :: 2`

#### `MTLCullMode` (enum `NSUInteger`)

- `MTLCullModeNone :: 0`
- `MTLCullModeFront :: 1`
- `MTLCullModeBack :: 2`

#### `MTLWinding` (enum `NSUInteger`)

- `MTLWindingClockwise :: 0`
- `MTLWindingCounterClockwise :: 1`

#### `MTLDepthClipMode` (enum `NSUInteger`)

- `MTLDepthClipModeClip :: 0`
- `MTLDepthClipModeClamp :: 1`

#### `MTLTriangleFillMode` (enum `NSUInteger`)

- `MTLTriangleFillModeFill :: 0`
- `MTLTriangleFillModeLines :: 1`

#### `MTLRenderStages` (enum `NSUInteger`)

- `MTLRenderStageVertex :: 1`
- `MTLRenderStageFragment :: 2`

#### `MTLBlendFactor` (enum `NSUInteger`)

- `MTLBlendFactorZero :: 0`
- `MTLBlendFactorOne :: 1`
- `MTLBlendFactorSourceColor :: 2`
- `MTLBlendFactorOneMinusSourceColor :: 3`
- `MTLBlendFactorSourceAlpha :: 4`
- `MTLBlendFactorOneMinusSourceAlpha :: 5`
- `MTLBlendFactorDestinationColor :: 6`
- `MTLBlendFactorOneMinusDestinationColor :: 7`
- `MTLBlendFactorDestinationAlpha :: 8`
- `MTLBlendFactorOneMinusDestinationAlpha :: 9`
- `MTLBlendFactorSourceAlphaSaturated :: 10`
- `MTLBlendFactorBlendColor :: 11`
- `MTLBlendFactorOneMinusBlendColor :: 12`
- `MTLBlendFactorBlendAlpha :: 13`
- `MTLBlendFactorOneMinusBlendAlpha :: 14`
- `MTLBlendFactorSource1Color :: 15`
- `MTLBlendFactorOneMinusSource1Color :: 16`
- `MTLBlendFactorSource1Alpha :: 17`
- `MTLBlendFactorOneMinusSource1Alpha :: 18`

#### `MTLBlendOperation` (enum `NSUInteger`)

- `MTLBlendOperationAdd :: 0`
- `MTLBlendOperationSubtract :: 1`
- `MTLBlendOperationReverseSubtract :: 2`
- `MTLBlendOperationMin :: 3`
- `MTLBlendOperationMax :: 4`

#### `MTLColorWriteMask` (enum `NSUInteger`)

- `MTLColorWriteMaskNone :: 0`
- `MTLColorWriteMaskRed :: 8`
- `MTLColorWriteMaskGreen :: 4`
- `MTLColorWriteMaskBlue :: 2`
- `MTLColorWriteMaskAlpha :: 1`
- `MTLColorWriteMaskAll :: 15`

#### `MTLPrimitiveTopologyClass` (enum `NSUInteger`)

- `MTLPrimitiveTopologyClassUnspecified :: 0`
- `MTLPrimitiveTopologyClassPoint :: 1`
- `MTLPrimitiveTopologyClassLine :: 2`
- `MTLPrimitiveTopologyClassTriangle :: 3`

#### `MTLTessellationPartitionMode` (enum `NSUInteger`)

- `MTLTessellationPartitionModePow2 :: 0`
- `MTLTessellationPartitionModeInteger :: 1`
- `MTLTessellationPartitionModeFractionalOdd :: 2`
- `MTLTessellationPartitionModeFractionalEven :: 3`

#### `MTLTessellationFactorStepFunction` (enum `NSUInteger`)

- `MTLTessellationFactorStepFunctionConstant :: 0`
- `MTLTessellationFactorStepFunctionPerPatch :: 1`
- `MTLTessellationFactorStepFunctionPerInstance :: 2`
- `MTLTessellationFactorStepFunctionPerPatchAndPerInstance :: 3`

#### `MTLTessellationFactorFormat` (enum `NSUInteger`)

- `MTLTessellationFactorFormatHalf :: 0`

#### `MTLTessellationControlPointIndexType` (enum `NSUInteger`)

- `MTLTessellationControlPointIndexTypeNone :: 0`
- `MTLTessellationControlPointIndexTypeUInt16 :: 1`
- `MTLTessellationControlPointIndexTypeUInt32 :: 2`

#### `MTLVertexFormat` (enum `NSUInteger`)

- `MTLVertexFormatInvalid :: 0`
- `MTLVertexFormatUChar2 :: 1`
- `MTLVertexFormatUChar3 :: 2`
- `MTLVertexFormatUChar4 :: 3`
- `MTLVertexFormatChar2 :: 4`
- `MTLVertexFormatChar3 :: 5`
- `MTLVertexFormatChar4 :: 6`
- `MTLVertexFormatUChar2Normalized :: 7`
- `MTLVertexFormatUChar3Normalized :: 8`
- `MTLVertexFormatUChar4Normalized :: 9`
- `MTLVertexFormatChar2Normalized :: 10`
- `MTLVertexFormatChar3Normalized :: 11`
- `MTLVertexFormatChar4Normalized :: 12`
- `MTLVertexFormatUShort2 :: 13`
- `MTLVertexFormatUShort3 :: 14`
- `MTLVertexFormatUShort4 :: 15`
- `MTLVertexFormatShort2 :: 16`
- `MTLVertexFormatShort3 :: 17`
- `MTLVertexFormatShort4 :: 18`
- `MTLVertexFormatUShort2Normalized :: 19`
- `MTLVertexFormatUShort3Normalized :: 20`
- `MTLVertexFormatUShort4Normalized :: 21`
- `MTLVertexFormatShort2Normalized :: 22`
- `MTLVertexFormatShort3Normalized :: 23`
- `MTLVertexFormatShort4Normalized :: 24`
- `MTLVertexFormatHalf2 :: 25`
- `MTLVertexFormatHalf3 :: 26`
- `MTLVertexFormatHalf4 :: 27`
- `MTLVertexFormatFloat :: 28`
- `MTLVertexFormatFloat2 :: 29`
- `MTLVertexFormatFloat3 :: 30`
- `MTLVertexFormatFloat4 :: 31`
- `MTLVertexFormatInt :: 32`
- `MTLVertexFormatInt2 :: 33`
- `MTLVertexFormatInt3 :: 34`
- `MTLVertexFormatInt4 :: 35`
- `MTLVertexFormatUInt :: 36`
- `MTLVertexFormatUInt2 :: 37`
- `MTLVertexFormatUInt3 :: 38`
- `MTLVertexFormatUInt4 :: 39`
- `MTLVertexFormatInt1010102Normalized :: 40`
- `MTLVertexFormatUInt1010102Normalized :: 41`
- `MTLVertexFormatUChar4Normalized_BGRA :: 42`
- `MTLVertexFormatUChar :: 45`
- `MTLVertexFormatChar :: 46`
- `MTLVertexFormatUCharNormalized :: 47`
- `MTLVertexFormatCharNormalized :: 48`
- `MTLVertexFormatUShort :: 49`
- `MTLVertexFormatShort :: 50`
- `MTLVertexFormatUShortNormalized :: 51`
- `MTLVertexFormatShortNormalized :: 52`
- `MTLVertexFormatHalf :: 53`

#### `MTLVertexStepFunction` (enum `NSUInteger`)

- `MTLVertexStepFunctionConstant :: 0`
- `MTLVertexStepFunctionPerVertex :: 1`
- `MTLVertexStepFunctionPerInstance :: 2`
- `MTLVertexStepFunctionPerPatch :: 3`
- `MTLVertexStepFunctionPerPatchControlPoint :: 4`

#### `MTLSamplerMinMagFilter` (enum `NSUInteger`)

- `MTLSamplerMinMagFilterNearest :: 0`
- `MTLSamplerMinMagFilterLinear :: 1`

#### `MTLSamplerMipFilter` (enum `NSUInteger`)

- `MTLSamplerMipFilterNotMipmapped :: 0`
- `MTLSamplerMipFilterNearest :: 1`
- `MTLSamplerMipFilterLinear :: 2`

#### `MTLSamplerAddressMode` (enum `NSUInteger`)

- `MTLSamplerAddressModeClampToEdge :: 0`
- `MTLSamplerAddressModeMirrorClampToEdge :: 1`
- `MTLSamplerAddressModeRepeat :: 2`
- `MTLSamplerAddressModeMirrorRepeat :: 3`
- `MTLSamplerAddressModeClampToZero :: 4`
- `MTLSamplerAddressModeClampToBorderColor :: 5`

#### `MTLSamplerBorderColor` (enum `NSUInteger`)

- `MTLSamplerBorderColorTransparentBlack :: 0`
- `MTLSamplerBorderColorOpaqueBlack :: 1`
- `MTLSamplerBorderColorOpaqueWhite :: 2`

#### `MTLIndirectCommandType` (enum `NSUInteger`)

- `MTLIndirectCommandTypeDraw :: 1`
- `MTLIndirectCommandTypeDrawIndexed :: 2`
- `MTLIndirectCommandTypeDrawPatches :: 4`
- `MTLIndirectCommandTypeDrawIndexedPatches :: 8`


#### `CAMetalDrawable` (struct)

- `texture :: (self: *CAMetalDrawable) -> id {`
- `layer :: (self: *CAMetalDrawable) -> *CAMetalLayer {`

#### `CAMetalLayer` (struct)

- `using calayer: CALayer;`

- `_priv: *_CAMetalLayerPrivate`
- `nextDrawable :: (self: *CAMetalLayer) -> *CAMetalDrawable {`
- `device :: (self: *CAMetalLayer) -> *MTLDevice {`
- `setDevice :: (self: *CAMetalLayer, device: *MTLDevice) -> () {`
- `pixelFormat :: (self: *CAMetalLayer) -> MTLPixelFormat {`
- `setPixelFormat :: (self: *CAMetalLayer, pixelFormat: MTLPixelFormat) -> () {`
- `framebufferOnly :: (self: *CAMetalLayer) -> BOOL {`
- `setFramebufferOnly :: (self: *CAMetalLayer, framebufferOnly: BOOL) -> () {`
- `drawableSize :: (self: *CAMetalLayer) -> CGSize {`
- `setDrawableSize :: (self: *CAMetalLayer, drawableSize: CGSize) -> () {`
- `maximumDrawableCount :: (self: *CAMetalLayer) -> NSUInteger {`
- `setMaximumDrawableCount :: (self: *CAMetalLayer, maximumDrawableCount: NSUInteger) -> () {`
- `presentsWithTransaction :: (self: *CAMetalLayer) -> BOOL {`
- `setPresentsWithTransaction :: (self: *CAMetalLayer, presentsWithTransaction: BOOL) -> () {`
- `colorspace :: (self: *CAMetalLayer) -> CGColorSpaceRef {`
- `setColorspace :: (self: *CAMetalLayer, colorspace: CGColorSpaceRef) -> () {`
- `wantsExtendedDynamicRangeContent :: (self: *CAMetalLayer) -> BOOL {`
- `setWantsExtendedDynamicRangeContent :: (self: *CAMetalLayer, wantsExtendedDynamicRangeContent: BOOL) -> () {`
- `displaySyncEnabled :: (self: *CAMetalLayer) -> BOOL {`
- `setDisplaySyncEnabled :: (self: *CAMetalLayer, displaySyncEnabled: BOOL) -> () {`
- `allowsNextDrawableTimeout :: (self: *CAMetalLayer) -> BOOL {`
- `setAllowsNextDrawableTimeout :: (self: *CAMetalLayer, allowsNextDrawableTimeout: BOOL) -> () {`

#### `MTLOrigin` (struct)

- `x: NSUInteger`
- `y: NSUInteger`
- `z: NSUInteger`

#### `MTLSize` (struct)

- `width: NSUInteger`
- `height: NSUInteger`
- `depth: NSUInteger`

#### `MTLRegion` (struct)

- `origin: MTLOrigin`
- `size: MTLSize`

#### `MTLSamplePosition` (struct)

- `x: float32`
- `y: float32`

#### `MTLCommandEncoder` (struct)

- `endEncoding :: (self: *MTLCommandEncoder) -> () {`
- `insertDebugSignpost :: (self: *MTLCommandEncoder, _string: *NSString) -> () {`
- `pushDebugGroup :: (self: *MTLCommandEncoder, _string: *NSString) -> () {`
- `popDebugGroup :: (self: *MTLCommandEncoder) -> () {`
- `device :: (self: *MTLCommandEncoder) -> id {`
- `label :: (self: *MTLCommandEncoder) -> *NSString {`
- `setLabel :: (self: *MTLCommandEncoder, label: *NSString) -> () {`

#### `MTLResource` (struct)

- `setPurgeableState :: (self: *MTLResource, state: MTLPurgeableState) -> MTLPurgeableState {`
- `makeAliasable :: (self: *MTLResource) -> () {`
- `isAliasable :: (self: *MTLResource) -> BOOL {`
- `label :: (self: *MTLResource) -> *NSString {`
- `setLabel :: (self: *MTLResource, label: *NSString) -> () {`
- `device :: (self: *MTLResource) -> id {`
- `cpuCacheMode :: (self: *MTLResource) -> MTLCPUCacheMode {`
- `storageMode :: (self: *MTLResource) -> MTLStorageMode {`
- `heap :: (self: *MTLResource) -> id {`
- `allocatedSize :: (self: *MTLResource) -> NSUInteger {`

#### `MTLBuffer` (struct)

- `contents :: (self: *MTLBuffer) -> *void {`
- `didModifyRange :: (self: *MTLBuffer, range: NSRange) -> () {`
- `newTextureWithDescriptor_offset_bytesPerRow :: (self: *MTLBuffer, descriptor: *MTLTextureDescriptor, offset: NSUInteger, bytesPerRow: NSUInteger) -> id {`
- `addDebugMarker_range :: (self: *MTLBuffer, marker: *NSString, range: NSRange) -> () {`
- `removeAllDebugMarkers :: (self: *MTLBuffer) -> () {`
- `length :: (self: *MTLBuffer) -> NSUInteger {`

#### `MTLSharedTextureHandle` (struct)

- `nsobject: NSObject`
- `_priv: *MTLSharedTextureHandlePrivate`
- `device :: (self: *MTLSharedTextureHandle) -> id {`
- `label :: (self: *MTLSharedTextureHandle) -> *NSString {`

#### `MTLTextureDescriptor` (struct)

- `nsobject: NSObject`
- `texture2DDescriptorWithPixelFormat_width_height_mipmapped :: (pixelFormat: MTLPixelFormat, width: NSUInteger, height: NSUInteger, mipmapped: BOOL) -> *MTLTextureDescriptor {`
- `textureCubeDescriptorWithPixelFormat_size_mipmapped :: (pixelFormat: MTLPixelFormat, size: NSUInteger, mipmapped: BOOL) -> *MTLTextureDescriptor {`
- `textureBufferDescriptorWithPixelFormat_width_resourceOptions_usage :: (pixelFormat: MTLPixelFormat, width: NSUInteger, resourceOptions: MTLResourceOptions, usage: MTLTextureUsage) -> *MTLTextureDescriptor {`
- `textureType :: (self: *MTLTextureDescriptor) -> MTLTextureType {`
- `setTextureType :: (self: *MTLTextureDescriptor, textureType: MTLTextureType) -> () {`
- `pixelFormat :: (self: *MTLTextureDescriptor) -> MTLPixelFormat {`
- `setPixelFormat :: (self: *MTLTextureDescriptor, pixelFormat: MTLPixelFormat) -> () {`
- `width :: (self: *MTLTextureDescriptor) -> NSUInteger {`
- `setWidth :: (self: *MTLTextureDescriptor, width: NSUInteger) -> () {`
- `height :: (self: *MTLTextureDescriptor) -> NSUInteger {`
- `setHeight :: (self: *MTLTextureDescriptor, height: NSUInteger) -> () {`
- `depth :: (self: *MTLTextureDescriptor) -> NSUInteger {`
- `setDepth :: (self: *MTLTextureDescriptor, depth: NSUInteger) -> () {`
- `mipmapLevelCount :: (self: *MTLTextureDescriptor) -> NSUInteger {`
- `setMipmapLevelCount :: (self: *MTLTextureDescriptor, mipmapLevelCount: NSUInteger) -> () {`
- `sampleCount :: (self: *MTLTextureDescriptor) -> NSUInteger {`
- `setSampleCount :: (self: *MTLTextureDescriptor, sampleCount: NSUInteger) -> () {`
- `arrayLength :: (self: *MTLTextureDescriptor) -> NSUInteger {`
- `setArrayLength :: (self: *MTLTextureDescriptor, arrayLength: NSUInteger) -> () {`
- `resourceOptions :: (self: *MTLTextureDescriptor) -> MTLResourceOptions {`
- `setResourceOptions :: (self: *MTLTextureDescriptor, resourceOptions: MTLResourceOptions) -> () {`
- `cpuCacheMode :: (self: *MTLTextureDescriptor) -> MTLCPUCacheMode {`
- `setCpuCacheMode :: (self: *MTLTextureDescriptor, cpuCacheMode: MTLCPUCacheMode) -> () {`
- `storageMode :: (self: *MTLTextureDescriptor) -> MTLStorageMode {`
- `setStorageMode :: (self: *MTLTextureDescriptor, storageMode: MTLStorageMode) -> () {`
- `usage :: (self: *MTLTextureDescriptor) -> MTLTextureUsage {`
- `setUsage :: (self: *MTLTextureDescriptor, usage: MTLTextureUsage) -> () {`
- `allowGPUOptimizedContents :: (self: *MTLTextureDescriptor) -> BOOL {`
- `setAllowGPUOptimizedContents :: (self: *MTLTextureDescriptor, allowGPUOptimizedContents: BOOL) -> () {`

#### `MTLTexture` (struct)

- `using MTLResource;`
- `getBytes_bytesPerRow_bytesPerImage_fromRegion_mipmapLevel_slice :: (self: *MTLTexture, pixelBytes: *void, bytesPerRow: NSUInteger, bytesPerImage: NSUInteger, region: MTLRegion, level: NSUInteger, slice: NSUInteger) -> () {`
- `replaceRegion_mipmapLevel_slice_withBytes_bytesPerRow_bytesPerImage :: (self: *MTLTexture, region: MTLRegion, level: NSUInteger, slice: NSUInteger, pixelBytes: *void, bytesPerRow: NSUInteger, bytesPerImage: NSUInteger) -> () {`
- `getBytes_bytesPerRow_fromRegion_mipmapLevel :: (self: *MTLTexture, pixelBytes: *void, bytesPerRow: NSUInteger, region: MTLRegion, level: NSUInteger) -> () {`
- `replaceRegion_mipmapLevel_withBytes_bytesPerRow :: (self: *MTLTexture, region: MTLRegion, level: NSUInteger, pixelBytes: *void, bytesPerRow: NSUInteger) -> () {`
- `newTextureViewWithPixelFormat :: (self: *MTLTexture, pixelFormat: MTLPixelFormat) -> id {`
- `newTextureViewWithPixelFormat_textureType_levels_slices :: (self: *MTLTexture, pixelFormat: MTLPixelFormat, textureType: MTLTextureType, levelRange: NSRange, sliceRange: NSRange) -> id {`
- `newSharedTextureHandle :: (self: *MTLTexture) -> *MTLSharedTextureHandle {`
- `rootResource :: (self: *MTLTexture) -> id {`
- `parentTexture :: (self: *MTLTexture) -> id {`
- `parentRelativeLevel :: (self: *MTLTexture) -> NSUInteger {`
- `parentRelativeSlice :: (self: *MTLTexture) -> NSUInteger {`
- `buffer :: (self: *MTLTexture) -> id {`
- `bufferOffset :: (self: *MTLTexture) -> NSUInteger {`
- `bufferBytesPerRow :: (self: *MTLTexture) -> NSUInteger {`
- `iosurface :: (self: *MTLTexture) -> IOSurfaceRef {`
- `iosurfacePlane :: (self: *MTLTexture) -> NSUInteger {`
- `textureType :: (self: *MTLTexture) -> MTLTextureType {`
- `pixelFormat :: (self: *MTLTexture) -> MTLPixelFormat {`
- `width :: (self: *MTLTexture) -> NSUInteger {`
- `height :: (self: *MTLTexture) -> NSUInteger {`
- `depth :: (self: *MTLTexture) -> NSUInteger {`
- `mipmapLevelCount :: (self: *MTLTexture) -> NSUInteger {`
- `sampleCount :: (self: *MTLTexture) -> NSUInteger {`
- `arrayLength :: (self: *MTLTexture) -> NSUInteger {`
- `usage :: (self: *MTLTexture) -> MTLTextureUsage {`
- `isShareable :: (self: *MTLTexture) -> BOOL {`
- `isFramebufferOnly :: (self: *MTLTexture) -> BOOL {`
- `allowGPUOptimizedContents :: (self: *MTLTexture) -> BOOL {`

#### `MTLType` (struct)

- `nsobject: NSObject`
- `dataType :: (self: *MTLType) -> MTLDataType {`

#### `MTLStructMember` (struct)

- `nsobject: NSObject`
- `structType :: (self: *MTLStructMember) -> *MTLStructType {`
- `arrayType :: (self: *MTLStructMember) -> *MTLArrayType {`
- `textureReferenceType :: (self: *MTLStructMember) -> *MTLTextureReferenceType {`
- `pointerType :: (self: *MTLStructMember) -> *MTLPointerType {`
- `name :: (self: *MTLStructMember) -> *NSString {`
- `offset :: (self: *MTLStructMember) -> NSUInteger {`
- `dataType :: (self: *MTLStructMember) -> MTLDataType {`
- `argumentIndex :: (self: *MTLStructMember) -> NSUInteger {`

#### `MTLStructType` (struct)

- `mtltype: MTLType`
- `memberByName :: (self: *MTLStructType, name: *NSString) -> *MTLStructMember {`
- `members :: (self: *MTLStructType) -> *NSArray(*MTLStructMember) {`

#### `MTLArrayType` (struct)

- `mtltype: MTLType`
- `elementStructType :: (self: *MTLArrayType) -> *MTLStructType {`
- `elementArrayType :: (self: *MTLArrayType) -> *MTLArrayType {`
- `elementTextureReferenceType :: (self: *MTLArrayType) -> *MTLTextureReferenceType {`
- `elementPointerType :: (self: *MTLArrayType) -> *MTLPointerType {`
- `elementType :: (self: *MTLArrayType) -> MTLDataType {`
- `arrayLength :: (self: *MTLArrayType) -> NSUInteger {`
- `stride :: (self: *MTLArrayType) -> NSUInteger {`
- `argumentIndexStride :: (self: *MTLArrayType) -> NSUInteger {`

#### `MTLPointerType` (struct)

- `mtltype: MTLType`
- `elementStructType :: (self: *MTLPointerType) -> *MTLStructType {`
- `elementArrayType :: (self: *MTLPointerType) -> *MTLArrayType {`
- `elementType :: (self: *MTLPointerType) -> MTLDataType {`
- `access :: (self: *MTLPointerType) -> MTLArgumentAccess {`
- `alignment :: (self: *MTLPointerType) -> NSUInteger {`
- `dataSize :: (self: *MTLPointerType) -> NSUInteger {`
- `elementIsArgumentBuffer :: (self: *MTLPointerType) -> BOOL {`

#### `MTLTextureReferenceType` (struct)

- `mtltype: MTLType`
- `textureDataType :: (self: *MTLTextureReferenceType) -> MTLDataType {`
- `textureType :: (self: *MTLTextureReferenceType) -> MTLTextureType {`
- `access :: (self: *MTLTextureReferenceType) -> MTLArgumentAccess {`
- `isDepthTexture :: (self: *MTLTextureReferenceType) -> BOOL {`

#### `MTLArgument` (struct)

- `nsobject: NSObject`
- `name :: (self: *MTLArgument) -> *NSString {`
- `type :: (self: *MTLArgument) -> MTLArgumentType {`
- `access :: (self: *MTLArgument) -> MTLArgumentAccess {`
- `index :: (self: *MTLArgument) -> NSUInteger {`
- `isActive :: (self: *MTLArgument) -> BOOL {`
- `bufferAlignment :: (self: *MTLArgument) -> NSUInteger {`
- `bufferDataSize :: (self: *MTLArgument) -> NSUInteger {`
- `bufferDataType :: (self: *MTLArgument) -> MTLDataType {`
- `bufferStructType :: (self: *MTLArgument) -> *MTLStructType {`
- `bufferPointerType :: (self: *MTLArgument) -> *MTLPointerType {`
- `threadgroupMemoryAlignment :: (self: *MTLArgument) -> NSUInteger {`
- `threadgroupMemoryDataSize :: (self: *MTLArgument) -> NSUInteger {`
- `textureType :: (self: *MTLArgument) -> MTLTextureType {`
- `textureDataType :: (self: *MTLArgument) -> MTLDataType {`
- `isDepthTexture :: (self: *MTLArgument) -> BOOL {`
- `arrayLength :: (self: *MTLArgument) -> NSUInteger {`

#### `MTLVertexAttribute` (struct)

- `nsobject: NSObject`
- `name :: (self: *MTLVertexAttribute) -> *NSString {`
- `attributeIndex :: (self: *MTLVertexAttribute) -> NSUInteger {`
- `attributeType :: (self: *MTLVertexAttribute) -> MTLDataType {`
- `isActive :: (self: *MTLVertexAttribute) -> BOOL {`
- `isPatchData :: (self: *MTLVertexAttribute) -> BOOL {`
- `isPatchControlPointData :: (self: *MTLVertexAttribute) -> BOOL {`

#### `MTLAttribute` (struct)

- `nsobject: NSObject`
- `name :: (self: *MTLAttribute) -> *NSString {`
- `attributeIndex :: (self: *MTLAttribute) -> NSUInteger {`
- `attributeType :: (self: *MTLAttribute) -> MTLDataType {`
- `isActive :: (self: *MTLAttribute) -> BOOL {`
- `isPatchData :: (self: *MTLAttribute) -> BOOL {`
- `isPatchControlPointData :: (self: *MTLAttribute) -> BOOL {`

#### `MTLFunctionConstant` (struct)

- `nsobject: NSObject`
- `name :: (self: *MTLFunctionConstant) -> *NSString {`
- `type :: (self: *MTLFunctionConstant) -> MTLDataType {`
- `index :: (self: *MTLFunctionConstant) -> NSUInteger {`
- `required :: (self: *MTLFunctionConstant) -> BOOL {`

#### `MTLFunction` (struct)

- `newArgumentEncoderWithBufferIndex :: (self: *MTLFunction, bufferIndex: NSUInteger) -> id {`
- `newArgumentEncoderWithBufferIndex_reflection :: (self: *MTLFunction, bufferIndex: NSUInteger, reflection: **MTLArgument) -> id {`
- `label :: (self: *MTLFunction) -> *NSString {`
- `setLabel :: (self: *MTLFunction, label: *NSString) -> () {`
- `device :: (self: *MTLFunction) -> id {`
- `functionType :: (self: *MTLFunction) -> MTLFunctionType {`
- `patchType :: (self: *MTLFunction) -> MTLPatchType {`
- `patchControlPointCount :: (self: *MTLFunction) -> NSInteger {`
- `vertexAttributes :: (self: *MTLFunction) -> *NSArray(*MTLVertexAttribute) {`
- `stageInputAttributes :: (self: *MTLFunction) -> *NSArray(*MTLAttribute) {`
- `name :: (self: *MTLFunction) -> *NSString {`
- `functionConstantsDictionary :: (self: *MTLFunction) -> *NSDictionary {`

#### `MTLCompileOptions` (struct)

- `nsobject: NSObject`
- `preprocessorMacros :: (self: *MTLCompileOptions) -> *NSDictionary {`
- `setPreprocessorMacros :: (self: *MTLCompileOptions, preprocessorMacros: *NSDictionary) -> () {`
- `fastMathEnabled :: (self: *MTLCompileOptions) -> BOOL {`
- `setFastMathEnabled :: (self: *MTLCompileOptions, fastMathEnabled: BOOL) -> () {`
- `languageVersion :: (self: *MTLCompileOptions) -> MTLLanguageVersion {`
- `setLanguageVersion :: (self: *MTLCompileOptions, languageVersion: MTLLanguageVersion) -> () {`

#### `MTLLibrary` (struct)

- `newFunctionWithName :: (self: *MTLLibrary, functionName: *NSString) -> id {`
- `newFunctionWithName_constantValues_error :: (self: *MTLLibrary, name: *NSString, constantValues: *MTLFunctionConstantValues, error: **NSError) -> id {`
- `label :: (self: *MTLLibrary) -> *NSString {`
- `setLabel :: (self: *MTLLibrary, label: *NSString) -> () {`
- `device :: (self: *MTLLibrary) -> id {`
- `functionNames :: (self: *MTLLibrary) -> *NSArray(*NSString) {`

#### `MTLSizeAndAlign` (struct)

- `size: NSUInteger`
- `align: NSUInteger`

#### `MTLArgumentDescriptor` (struct)

- `nsobject: NSObject`
- `argumentDescriptor :: () -> *MTLArgumentDescriptor {`
- `dataType :: (self: *MTLArgumentDescriptor) -> MTLDataType {`
- `setDataType :: (self: *MTLArgumentDescriptor, dataType: MTLDataType) -> () {`
- `index :: (self: *MTLArgumentDescriptor) -> NSUInteger {`
- `setIndex :: (self: *MTLArgumentDescriptor, index: NSUInteger) -> () {`
- `arrayLength :: (self: *MTLArgumentDescriptor) -> NSUInteger {`
- `setArrayLength :: (self: *MTLArgumentDescriptor, arrayLength: NSUInteger) -> () {`
- `access :: (self: *MTLArgumentDescriptor) -> MTLArgumentAccess {`
- `setAccess :: (self: *MTLArgumentDescriptor, access: MTLArgumentAccess) -> () {`
- `textureType :: (self: *MTLArgumentDescriptor) -> MTLTextureType {`
- `setTextureType :: (self: *MTLArgumentDescriptor, textureType: MTLTextureType) -> () {`
- `constantBlockAlignment :: (self: *MTLArgumentDescriptor) -> NSUInteger {`
- `setConstantBlockAlignment :: (self: *MTLArgumentDescriptor, constantBlockAlignment: NSUInteger) -> () {`

#### `MTLDevice` (struct)

- `newCommandQueue :: (self: *MTLDevice) -> *MTLCommandQueue {`
- `newCommandQueueWithMaxCommandBufferCount :: (self: *MTLDevice, maxCommandBufferCount: NSUInteger) -> *MTLCommandQueue {`
- `heapTextureSizeAndAlignWithDescriptor :: (self: *MTLDevice, desc: *MTLTextureDescriptor) -> MTLSizeAndAlign {`
- `heapBufferSizeAndAlignWithLength_options :: (self: *MTLDevice, length: NSUInteger, options: MTLResourceOptions) -> MTLSizeAndAlign {`
- `newHeapWithDescriptor :: (self: *MTLDevice, descriptor: *MTLHeapDescriptor) -> id {`
- `newBufferWithLength_options :: (self: *MTLDevice, length: NSUInteger, options: MTLResourceOptions) -> *MTLBuffer {`
- `newBufferWithBytes_length_options :: (self: *MTLDevice, pointer: *void, length: NSUInteger, options: MTLResourceOptions) -> *MTLBuffer {`
- `newDepthStencilStateWithDescriptor :: (self: *MTLDevice, descriptor: *MTLDepthStencilDescriptor) -> id {`
- `newTextureWithDescriptor :: (self: *MTLDevice, descriptor: *MTLTextureDescriptor) -> id {`
- `newTextureWithDescriptor_iosurface_plane :: (self: *MTLDevice, descriptor: *MTLTextureDescriptor, iosurface: IOSurfaceRef, plane: NSUInteger) -> id {`
- `newSharedTextureWithDescriptor :: (self: *MTLDevice, descriptor: *MTLTextureDescriptor) -> id {`
- `newSharedTextureWithHandle :: (self: *MTLDevice, sharedHandle: *MTLSharedTextureHandle) -> id {`
- `newSamplerStateWithDescriptor :: (self: *MTLDevice, descriptor: *MTLSamplerDescriptor) -> id {`
- `newDefaultLibrary :: (self: *MTLDevice) -> *MTLLibrary {`
- `newDefaultLibraryWithBundle_error :: (self: *MTLDevice, bundle: *NSBundle, error: **NSError) -> *MTLLibrary {`
- `newLibraryWithFile_error :: (self: *MTLDevice, filepath: *NSString, error: **NSError) -> *MTLLibrary {`
- `newLibraryWithURL_error :: (self: *MTLDevice, url: *NSURL, error: **NSError) -> *MTLLibrary {`
- `newLibraryWithData_error :: (self: *MTLDevice, data: dispatch_data_t, error: **NSError) -> *MTLLibrary {`
- `newLibraryWithSource_options_error :: (self: *MTLDevice, source: *NSString, options: *MTLCompileOptions, error: **NSError) -> *MTLLibrary {`
- `newRenderPipelineStateWithDescriptor_error :: (self: *MTLDevice, descriptor: *MTLRenderPipelineDescriptor, error: **NSError) -> *MTLRenderPipelineState {`
- `newRenderPipelineStateWithDescriptor_options_reflection_error :: (self: *MTLDevice, descriptor: *MTLRenderPipelineDescriptor, options: MTLPipelineOption, reflection: **MTLRenderPipelineReflection, error: **NSError) -> id {`
- `newComputePipelineStateWithFunction_error :: (self: *MTLDevice, computeFunction: id, error: **NSError) -> *MTLComputePipelineState {`
- `newComputePipelineStateWithFunction_options_reflection_error :: (self: *MTLDevice, computeFunction: id, options: MTLPipelineOption, reflection: **MTLComputePipelineReflection, error: **NSError) -> *MTLComputePipelineState {`
- `newComputePipelineStateWithDescriptor_options_reflection_error :: (self: *MTLDevice, descriptor: *MTLComputePipelineDescriptor, options: MTLPipelineOption, reflection: **MTLComputePipelineReflection, error: **NSError) -> id {`
- `newFence :: (self: *MTLDevice) -> id {`
- `supportsFeatureSet :: (self: *MTLDevice, featureSet: MTLFeatureSet) -> BOOL {`
- `supportsTextureSampleCount :: (self: *MTLDevice, sampleCount: NSUInteger) -> BOOL {`
- `minimumLinearTextureAlignmentForPixelFormat :: (self: *MTLDevice, format: MTLPixelFormat) -> NSUInteger {`
- `minimumTextureBufferAlignmentForPixelFormat :: (self: *MTLDevice, format: MTLPixelFormat) -> NSUInteger {`
- `getDefaultSamplePositions_count :: (self: *MTLDevice, positions: *MTLSamplePosition, count: NSUInteger) -> () {`
- `newArgumentEncoderWithArguments :: (self: *MTLDevice, arguments: *NSArray(*MTLArgumentDescriptor)) -> id {`
- `newIndirectCommandBufferWithDescriptor_maxCommandCount_options :: (self: *MTLDevice, descriptor: *MTLIndirectCommandBufferDescriptor, maxCount: NSUInteger, options: MTLResourceOptions) -> id {`
- `newEvent :: (self: *MTLDevice) -> id {`
- `newSharedEvent :: (self: *MTLDevice) -> id {`
- `newSharedEventWithHandle :: (self: *MTLDevice, sharedEventHandle: *MTLSharedEventHandle) -> id {`
- `name :: (self: *MTLDevice) -> *NSString {`
- `registryID :: (self: *MTLDevice) -> uint64_t {`
- `maxThreadsPerThreadgroup :: (self: *MTLDevice) -> MTLSize {`
- `isLowPower :: (self: *MTLDevice) -> BOOL {`
- `isHeadless :: (self: *MTLDevice) -> BOOL {`
- `isRemovable :: (self: *MTLDevice) -> BOOL {`
- `recommendedMaxWorkingSetSize :: (self: *MTLDevice) -> uint64_t {`
- `isDepth24Stencil8PixelFormatSupported :: (self: *MTLDevice) -> BOOL {`
- `readWriteTextureSupport :: (self: *MTLDevice) -> MTLReadWriteTextureTier {`
- `argumentBuffersSupport :: (self: *MTLDevice) -> MTLArgumentBuffersTier {`
- `areRasterOrderGroupsSupported :: (self: *MTLDevice) -> BOOL {`
- `currentAllocatedSize :: (self: *MTLDevice) -> NSUInteger {`
- `maxThreadgroupMemoryLength :: (self: *MTLDevice) -> NSUInteger {`
- `maxArgumentBufferSamplerCount :: (self: *MTLDevice) -> NSUInteger {`
- `areProgrammableSamplePositionsSupported :: (self: *MTLDevice) -> BOOL {`
- `maxBufferLength :: (self: *MTLDevice) -> NSUInteger {`

#### `MTLFence` (struct)

- `device :: (self: *MTLFence) -> id {`
- `label :: (self: *MTLFence) -> *NSString {`
- `setLabel :: (self: *MTLFence, label: *NSString) -> () {`

#### `MTLBlitCommandEncoder` (struct)

- `synchronizeResource :: (self: *MTLBlitCommandEncoder, resource: id) -> () {`
- `synchronizeTexture_slice_level :: (self: *MTLBlitCommandEncoder, texture: id, slice: NSUInteger, level: NSUInteger) -> () {`
- `copyFromTexture_sourceSlice_sourceLevel_sourceOrigin_sourceSize_toTexture_destinationSlice_destinationLevel_destinationOrigin :: (self: *MTLBlitCommandEncoder, sourceTexture: id, sourceSlice: NSUInteger, sourceLevel: NSUInteger, sourceOrigin: MTLOrigin, sourceSize: MTLSize, destinationTexture: id, destinationSlice: NSUInteger, destinationLevel: NSUInteger, destinationOrigin: MTLOrigin) -> () {`
- `copyFromBuffer_sourceOffset_sourceBytesPerRow_sourceBytesPerImage_sourceSize_toTexture_destinationSlice_destinationLevel_destinationOrigin :: (self: *MTLBlitCommandEncoder, sourceBuffer: id, sourceOffset: NSUInteger, sourceBytesPerRow: NSUInteger, sourceBytesPerImage: NSUInteger, sourceSize: MTLSize, destinationTexture: id, destinationSlice: NSUInteger, destinationLevel: NSUInteger, destinationOrigin: MTLOrigin) -> () {`
- `copyFromBuffer_sourceOffset_sourceBytesPerRow_sourceBytesPerImage_sourceSize_toTexture_destinationSlice_destinationLevel_destinationOrigin_options :: (self: *MTLBlitCommandEncoder, sourceBuffer: id, sourceOffset: NSUInteger, sourceBytesPerRow: NSUInteger, sourceBytesPerImage: NSUInteger, sourceSize: MTLSize, destinationTexture: id, destinationSlice: NSUInteger, destinationLevel: NSUInteger, destinationOrigin: MTLOrigin, options: MTLBlitOption) -> () {`
- `copyFromTexture_sourceSlice_sourceLevel_sourceOrigin_sourceSize_toBuffer_destinationOffset_destinationBytesPerRow_destinationBytesPerImage :: (self: *MTLBlitCommandEncoder, sourceTexture: id, sourceSlice: NSUInteger, sourceLevel: NSUInteger, sourceOrigin: MTLOrigin, sourceSize: MTLSize, destinationBuffer: id, destinationOffset: NSUInteger, destinationBytesPerRow: NSUInteger, destinationBytesPerImage: NSUInteger) -> () {`
- `copyFromTexture_sourceSlice_sourceLevel_sourceOrigin_sourceSize_toBuffer_destinationOffset_destinationBytesPerRow_destinationBytesPerImage_options :: (self: *MTLBlitCommandEncoder, sourceTexture: id, sourceSlice: NSUInteger, sourceLevel: NSUInteger, sourceOrigin: MTLOrigin, sourceSize: MTLSize, destinationBuffer: id, destinationOffset: NSUInteger, destinationBytesPerRow: NSUInteger, destinationBytesPerImage: NSUInteger, options: MTLBlitOption) -> () {`
- `generateMipmapsForTexture :: (self: *MTLBlitCommandEncoder, texture: id) -> () {`
- `fillBuffer_range_value :: (self: *MTLBlitCommandEncoder, buffer: id, range: NSRange, value: uint8_t) -> () {`
- `copyFromBuffer_sourceOffset_toBuffer_destinationOffset_size :: (self: *MTLBlitCommandEncoder, sourceBuffer: id, sourceOffset: NSUInteger, destinationBuffer: id, destinationOffset: NSUInteger, size: NSUInteger) -> () {`
- `updateFence :: (self: *MTLBlitCommandEncoder, fence: id) -> () {`
- `waitForFence :: (self: *MTLBlitCommandEncoder, fence: id) -> () {`
- `optimizeContentsForGPUAccess :: (self: *MTLBlitCommandEncoder, texture: id) -> () {`
- `optimizeContentsForGPUAccess_slice_level :: (self: *MTLBlitCommandEncoder, texture: id, slice: NSUInteger, level: NSUInteger) -> () {`
- `optimizeContentsForCPUAccess :: (self: *MTLBlitCommandEncoder, texture: id) -> () {`
- `optimizeContentsForCPUAccess_slice_level :: (self: *MTLBlitCommandEncoder, texture: id, slice: NSUInteger, level: NSUInteger) -> () {`
- `resetCommandsInBuffer_withRange :: (self: *MTLBlitCommandEncoder, buffer: id, range: NSRange) -> () {`
- `copyIndirectCommandBuffer_sourceRange_destination_destinationIndex :: (self: *MTLBlitCommandEncoder, source: id, sourceRange: NSRange, destination: id, destinationIndex: NSUInteger) -> () {`
- `optimizeIndirectCommandBuffer_withRange :: (self: *MTLBlitCommandEncoder, indirectCommandBuffer: id, range: NSRange) -> () {`

#### `MTLCommandBuffer` (struct)

- `enqueue :: (self: *MTLCommandBuffer) -> () {`
- `commit :: (self: *MTLCommandBuffer) -> () {`
- `presentDrawable :: (self: *MTLCommandBuffer, drawable: id) -> () {`
- `presentDrawable_atTime :: (self: *MTLCommandBuffer, drawable: id, presentationTime: CFTimeInterval) -> () {`
- `waitUntilScheduled :: (self: *MTLCommandBuffer) -> () {`
- `waitUntilCompleted :: (self: *MTLCommandBuffer) -> () {`
- `blitCommandEncoder :: (self: *MTLCommandBuffer) -> *MTLBlitCommandEncoder {`
- `renderCommandEncoderWithDescriptor :: (self: *MTLCommandBuffer, renderPassDescriptor: *MTLRenderPassDescriptor) -> *MTLRenderCommandEncoder {`
- `computeCommandEncoder :: (self: *MTLCommandBuffer) -> *MTLComputeCommandEncoder {`
- `computeCommandEncoderWithDispatchType :: (self: *MTLCommandBuffer, dispatchType: MTLDispatchType) -> *MTLComputeCommandEncoder {`
- `encodeWaitForEvent_value :: (self: *MTLCommandBuffer, event: id, value: uint64_t) -> () {`
- `encodeSignalEvent_value :: (self: *MTLCommandBuffer, event: id, value: uint64_t) -> () {`
- `parallelRenderCommandEncoderWithDescriptor :: (self: *MTLCommandBuffer, renderPassDescriptor: *MTLRenderPassDescriptor) -> id {`
- `pushDebugGroup :: (self: *MTLCommandBuffer, _string: *NSString) -> () {`
- `popDebugGroup :: (self: *MTLCommandBuffer) -> () {`
- `device :: (self: *MTLCommandBuffer) -> id {`
- `commandQueue :: (self: *MTLCommandBuffer) -> id {`
- `retainedReferences :: (self: *MTLCommandBuffer) -> BOOL {`
- `label :: (self: *MTLCommandBuffer) -> *NSString {`
- `setLabel :: (self: *MTLCommandBuffer, label: *NSString) -> () {`
- `kernelStartTime :: (self: *MTLCommandBuffer) -> CFTimeInterval {`
- `kernelEndTime :: (self: *MTLCommandBuffer) -> CFTimeInterval {`
- `GPUStartTime :: (self: *MTLCommandBuffer) -> CFTimeInterval {`
- `GPUEndTime :: (self: *MTLCommandBuffer) -> CFTimeInterval {`
- `status :: (self: *MTLCommandBuffer) -> MTLCommandBufferStatus {`
- `error :: (self: *MTLCommandBuffer) -> *NSError {`

#### `MTLDispatchThreadgroupsIndirectArguments` (struct)

- `threadgroupsPerGrid: [3] uint32_t`

#### `MTLStageInRegionIndirectArguments` (struct)

- `stageInOrigin: [3] uint32_t`
- `stageInSize: [3] uint32_t`

#### `MTLComputeCommandEncoder` (struct)

- `setComputePipelineState :: (self: *MTLComputeCommandEncoder, state: id) -> () {`
- `setBytes_length_atIndex :: (self: *MTLComputeCommandEncoder, bytes: *void, length: NSUInteger, index: NSUInteger) -> () {`
- `setBuffer_offset_atIndex :: (self: *MTLComputeCommandEncoder, buffer: id, offset: NSUInteger, index: NSUInteger) -> () {`
- `setBufferOffset_atIndex :: (self: *MTLComputeCommandEncoder, offset: NSUInteger, index: NSUInteger) -> () {`
- `setBuffers_offsets_withRange :: (self: *MTLComputeCommandEncoder, buffers: *id, offsets: *NSUInteger, range: NSRange) -> () {`
- `setTexture_atIndex :: (self: *MTLComputeCommandEncoder, texture: id, index: NSUInteger) -> () {`
- `setTextures_withRange :: (self: *MTLComputeCommandEncoder, textures: *id, range: NSRange) -> () {`
- `setSamplerState_atIndex :: (self: *MTLComputeCommandEncoder, sampler: id, index: NSUInteger) -> () {`
- `setSamplerStates_withRange :: (self: *MTLComputeCommandEncoder, samplers: *id, range: NSRange) -> () {`
- `setSamplerState_lodMinClamp_lodMaxClamp_atIndex :: (self: *MTLComputeCommandEncoder, sampler: id, lodMinClamp: float32, lodMaxClamp: float32, index: NSUInteger) -> () {`
- `setSamplerStates_lodMinClamps_lodMaxClamps_withRange :: (self: *MTLComputeCommandEncoder, samplers: *id, lodMinClamps: *float32, lodMaxClamps: *float32, range: NSRange) -> () {`
- `setThreadgroupMemoryLength_atIndex :: (self: *MTLComputeCommandEncoder, length: NSUInteger, index: NSUInteger) -> () {`
- `setStageInRegion :: (self: *MTLComputeCommandEncoder, region: MTLRegion) -> () {`
- `setStageInRegionWithIndirectBuffer_indirectBufferOffset :: (self: *MTLComputeCommandEncoder, indirectBuffer: id, indirectBufferOffset: NSUInteger) -> () {`
- `dispatchThreadgroups_threadsPerThreadgroup :: (self: *MTLComputeCommandEncoder, threadgroupsPerGrid: MTLSize, threadsPerThreadgroup: MTLSize) -> () {`
- `dispatchThreadgroupsWithIndirectBuffer_indirectBufferOffset_threadsPerThreadgroup :: (self: *MTLComputeCommandEncoder, indirectBuffer: id, indirectBufferOffset: NSUInteger, threadsPerThreadgroup: MTLSize) -> () {`
- `dispatchThreads_threadsPerThreadgroup :: (self: *MTLComputeCommandEncoder, threadsPerGrid: MTLSize, threadsPerThreadgroup: MTLSize) -> () {`
- `updateFence :: (self: *MTLComputeCommandEncoder, fence: id) -> () {`
- `waitForFence :: (self: *MTLComputeCommandEncoder, fence: id) -> () {`
- `useResource_usage :: (self: *MTLComputeCommandEncoder, resource: id, usage: MTLResourceUsage) -> () {`
- `useResources_count_usage :: (self: *MTLComputeCommandEncoder, resources: *id, count: NSUInteger, usage: MTLResourceUsage) -> () {`
- `useHeap :: (self: *MTLComputeCommandEncoder, heap: id) -> () {`
- `useHeaps_count :: (self: *MTLComputeCommandEncoder, heaps: *id, count: NSUInteger) -> () {`
- `memoryBarrierWithScope :: (self: *MTLComputeCommandEncoder, scope: MTLBarrierScope) -> () {`
- `memoryBarrierWithResources_count :: (self: *MTLComputeCommandEncoder, resources: *id, count: NSUInteger) -> () {`
- `dispatchType :: (self: *MTLComputeCommandEncoder) -> MTLDispatchType {`

#### `MTLCommandQueue` (struct)

- `commandBuffer :: (self: *MTLCommandQueue) -> *MTLCommandBuffer {`
- `commandBufferWithUnretainedReferences :: (self: *MTLCommandQueue) -> *MTLCommandBuffer {`
- `insertDebugCaptureBoundary :: (self: *MTLCommandQueue) -> () {`
- `label :: (self: *MTLCommandQueue) -> *NSString {`
- `setLabel :: (self: *MTLCommandQueue, label: *NSString) -> () {`
- `device :: (self: *MTLCommandQueue) -> id {`

#### `MTLStencilDescriptor` (struct)

- `nsobject: NSObject`
- `stencilCompareFunction :: (self: *MTLStencilDescriptor) -> MTLCompareFunction {`
- `setStencilCompareFunction :: (self: *MTLStencilDescriptor, stencilCompareFunction: MTLCompareFunction) -> () {`
- `stencilFailureOperation :: (self: *MTLStencilDescriptor) -> MTLStencilOperation {`
- `setStencilFailureOperation :: (self: *MTLStencilDescriptor, stencilFailureOperation: MTLStencilOperation) -> () {`
- `depthFailureOperation :: (self: *MTLStencilDescriptor) -> MTLStencilOperation {`
- `setDepthFailureOperation :: (self: *MTLStencilDescriptor, depthFailureOperation: MTLStencilOperation) -> () {`
- `depthStencilPassOperation :: (self: *MTLStencilDescriptor) -> MTLStencilOperation {`
- `setDepthStencilPassOperation :: (self: *MTLStencilDescriptor, depthStencilPassOperation: MTLStencilOperation) -> () {`
- `readMask :: (self: *MTLStencilDescriptor) -> uint32_t {`
- `setReadMask :: (self: *MTLStencilDescriptor, readMask: uint32_t) -> () {`
- `writeMask :: (self: *MTLStencilDescriptor) -> uint32_t {`
- `setWriteMask :: (self: *MTLStencilDescriptor, writeMask: uint32_t) -> () {`

#### `MTLDepthStencilDescriptor` (struct)

- `nsobject: NSObject`
- `depthCompareFunction :: (self: *MTLDepthStencilDescriptor) -> MTLCompareFunction {`
- `setDepthCompareFunction :: (self: *MTLDepthStencilDescriptor, depthCompareFunction: MTLCompareFunction) -> () {`
- `isDepthWriteEnabled :: (self: *MTLDepthStencilDescriptor) -> BOOL {`
- `setDepthWriteEnabled :: (self: *MTLDepthStencilDescriptor, depthWriteEnabled: BOOL) -> () {`
- `frontFaceStencil :: (self: *MTLDepthStencilDescriptor) -> *MTLStencilDescriptor {`
- `setFrontFaceStencil :: (self: *MTLDepthStencilDescriptor, frontFaceStencil: *MTLStencilDescriptor) -> () {`
- `backFaceStencil :: (self: *MTLDepthStencilDescriptor) -> *MTLStencilDescriptor {`
- `setBackFaceStencil :: (self: *MTLDepthStencilDescriptor, backFaceStencil: *MTLStencilDescriptor) -> () {`
- `label :: (self: *MTLDepthStencilDescriptor) -> *NSString {`
- `setLabel :: (self: *MTLDepthStencilDescriptor, label: *NSString) -> () {`

#### `MTLDepthStencilState` (struct)

- `label :: (self: *MTLDepthStencilState) -> *NSString {`
- `device :: (self: *MTLDepthStencilState) -> id {`

#### `MTLDrawable` (struct)

- `present :: (self: *MTLDrawable) -> () {`
- `presentAtTime :: (self: *MTLDrawable, presentationTime: CFTimeInterval) -> () {`

#### `MTLClearColor` (struct)

- `red: float64`
- `green: float64`
- `blue: float64`
- `alpha: float64`

#### `MTLRenderPassAttachmentDescriptor` (struct)

- `nsobject: NSObject`
- `texture :: (self: *MTLRenderPassAttachmentDescriptor) -> id {`
- `setTexture :: (self: *MTLRenderPassAttachmentDescriptor, texture: id) -> () {`
- `level :: (self: *MTLRenderPassAttachmentDescriptor) -> NSUInteger {`
- `setLevel :: (self: *MTLRenderPassAttachmentDescriptor, level: NSUInteger) -> () {`
- `slice :: (self: *MTLRenderPassAttachmentDescriptor) -> NSUInteger {`
- `setSlice :: (self: *MTLRenderPassAttachmentDescriptor, slice: NSUInteger) -> () {`
- `depthPlane :: (self: *MTLRenderPassAttachmentDescriptor) -> NSUInteger {`
- `setDepthPlane :: (self: *MTLRenderPassAttachmentDescriptor, depthPlane: NSUInteger) -> () {`
- `resolveTexture :: (self: *MTLRenderPassAttachmentDescriptor) -> id {`
- `setResolveTexture :: (self: *MTLRenderPassAttachmentDescriptor, resolveTexture: id) -> () {`
- `resolveLevel :: (self: *MTLRenderPassAttachmentDescriptor) -> NSUInteger {`
- `setResolveLevel :: (self: *MTLRenderPassAttachmentDescriptor, resolveLevel: NSUInteger) -> () {`
- `resolveSlice :: (self: *MTLRenderPassAttachmentDescriptor) -> NSUInteger {`
- `setResolveSlice :: (self: *MTLRenderPassAttachmentDescriptor, resolveSlice: NSUInteger) -> () {`
- `resolveDepthPlane :: (self: *MTLRenderPassAttachmentDescriptor) -> NSUInteger {`
- `setResolveDepthPlane :: (self: *MTLRenderPassAttachmentDescriptor, resolveDepthPlane: NSUInteger) -> () {`
- `loadAction :: (self: *MTLRenderPassAttachmentDescriptor) -> MTLLoadAction {`
- `setLoadAction :: (self: *MTLRenderPassAttachmentDescriptor, loadAction: MTLLoadAction) -> () {`
- `storeAction :: (self: *MTLRenderPassAttachmentDescriptor) -> MTLStoreAction {`
- `setStoreAction :: (self: *MTLRenderPassAttachmentDescriptor, storeAction: MTLStoreAction) -> () {`
- `storeActionOptions :: (self: *MTLRenderPassAttachmentDescriptor) -> MTLStoreActionOptions {`
- `setStoreActionOptions :: (self: *MTLRenderPassAttachmentDescriptor, storeActionOptions: MTLStoreActionOptions) -> () {`

#### `MTLRenderPassColorAttachmentDescriptor` (struct)

- `using #as mtlrenderpassattachmentdescriptor: MTLRenderPassAttachmentDescriptor;`
- `clearColor :: (self: *MTLRenderPassColorAttachmentDescriptor) -> MTLClearColor {`
- `setClearColor :: (self: *MTLRenderPassColorAttachmentDescriptor, clearColor: MTLClearColor) -> () {`

The color attachment descriptor embeds its base descriptor through the `using #as` member above. Its public inherited texture, resolve, and load/store accessors are consequently available through that using relationship. The Metal SDK likewise declares this class as a subclass of `MTLRenderPassAttachmentDescriptor`.

#### `MTLRenderPassDepthAttachmentDescriptor` (struct)

- `mtlrenderpassattachmentdescriptor: MTLRenderPassAttachmentDescriptor`
- `clearDepth :: (self: *MTLRenderPassDepthAttachmentDescriptor) -> float64 {`
- `setClearDepth :: (self: *MTLRenderPassDepthAttachmentDescriptor, clearDepth: float64) -> () {`
- `depthResolveFilter :: (self: *MTLRenderPassDepthAttachmentDescriptor) -> MTLMultisampleDepthResolveFilter {`
- `setDepthResolveFilter :: (self: *MTLRenderPassDepthAttachmentDescriptor, depthResolveFilter: MTLMultisampleDepthResolveFilter) -> () {`

#### `MTLRenderPassStencilAttachmentDescriptor` (struct)

- `mtlrenderpassattachmentdescriptor: MTLRenderPassAttachmentDescriptor`
- `clearStencil :: (self: *MTLRenderPassStencilAttachmentDescriptor) -> uint32_t {`
- `setClearStencil :: (self: *MTLRenderPassStencilAttachmentDescriptor, clearStencil: uint32_t) -> () {`
- `stencilResolveFilter :: (self: *MTLRenderPassStencilAttachmentDescriptor) -> MTLMultisampleStencilResolveFilter {`
- `setStencilResolveFilter :: (self: *MTLRenderPassStencilAttachmentDescriptor, stencilResolveFilter: MTLMultisampleStencilResolveFilter) -> () {`

Depth and stencil wrappers expose the base descriptor as the ordinary named member shown above; they do not declare it with `using`. The SDK also declares both classes as subclasses, but this Jai surface does not add using-based inherited member lookup for those two wrappers.

#### `MTLRenderPassColorAttachmentDescriptorArray` (struct)

- `nsobject: NSObject`
- `objectAtIndexedSubscript :: (self: *MTLRenderPassColorAttachmentDescriptorArray, attachmentIndex: NSUInteger) -> *MTLRenderPassColorAttachmentDescriptor {`
- `setObject_atIndexedSubscript :: (self: *MTLRenderPassColorAttachmentDescriptorArray, attachment: *MTLRenderPassColorAttachmentDescriptor, attachmentIndex: NSUInteger) -> () {`

#### `MTLRenderPassDescriptor` (struct)

- `nsobject: NSObject`
- `renderPassDescriptor :: () -> *MTLRenderPassDescriptor {`
- `setSamplePositions_count :: (self: *MTLRenderPassDescriptor, positions: *MTLSamplePosition, count: NSUInteger) -> () {`
- `getSamplePositions_count :: (self: *MTLRenderPassDescriptor, positions: *MTLSamplePosition, count: NSUInteger) -> NSUInteger {`
- `colorAttachments :: (self: *MTLRenderPassDescriptor) -> *MTLRenderPassColorAttachmentDescriptorArray {`
- `depthAttachment :: (self: *MTLRenderPassDescriptor) -> *MTLRenderPassDepthAttachmentDescriptor {`
- `setDepthAttachment :: (self: *MTLRenderPassDescriptor, depthAttachment: *MTLRenderPassDepthAttachmentDescriptor) -> () {`
- `stencilAttachment :: (self: *MTLRenderPassDescriptor) -> *MTLRenderPassStencilAttachmentDescriptor {`
- `setStencilAttachment :: (self: *MTLRenderPassDescriptor, stencilAttachment: *MTLRenderPassStencilAttachmentDescriptor) -> () {`
- `visibilityResultBuffer :: (self: *MTLRenderPassDescriptor) -> id {`
- `setVisibilityResultBuffer :: (self: *MTLRenderPassDescriptor, visibilityResultBuffer: id) -> () {`
- `renderTargetArrayLength :: (self: *MTLRenderPassDescriptor) -> NSUInteger {`
- `setRenderTargetArrayLength :: (self: *MTLRenderPassDescriptor, renderTargetArrayLength: NSUInteger) -> () {`

#### `MTLBufferLayoutDescriptor` (struct)

- `nsobject: NSObject`
- `stride :: (self: *MTLBufferLayoutDescriptor) -> NSUInteger {`
- `setStride :: (self: *MTLBufferLayoutDescriptor, stride: NSUInteger) -> () {`
- `stepFunction :: (self: *MTLBufferLayoutDescriptor) -> MTLStepFunction {`
- `setStepFunction :: (self: *MTLBufferLayoutDescriptor, stepFunction: MTLStepFunction) -> () {`
- `stepRate :: (self: *MTLBufferLayoutDescriptor) -> NSUInteger {`
- `setStepRate :: (self: *MTLBufferLayoutDescriptor, stepRate: NSUInteger) -> () {`

#### `MTLBufferLayoutDescriptorArray` (struct)

- `nsobject: NSObject`
- `objectAtIndexedSubscript :: (self: *MTLBufferLayoutDescriptorArray, index: NSUInteger) -> *MTLBufferLayoutDescriptor {`
- `setObject_atIndexedSubscript :: (self: *MTLBufferLayoutDescriptorArray, bufferDesc: *MTLBufferLayoutDescriptor, index: NSUInteger) -> () {`

#### `MTLAttributeDescriptor` (struct)

- `nsobject: NSObject`
- `format :: (self: *MTLAttributeDescriptor) -> MTLAttributeFormat {`
- `setFormat :: (self: *MTLAttributeDescriptor, format: MTLAttributeFormat) -> () {`
- `offset :: (self: *MTLAttributeDescriptor) -> NSUInteger {`
- `setOffset :: (self: *MTLAttributeDescriptor, offset: NSUInteger) -> () {`
- `bufferIndex :: (self: *MTLAttributeDescriptor) -> NSUInteger {`
- `setBufferIndex :: (self: *MTLAttributeDescriptor, bufferIndex: NSUInteger) -> () {`

#### `MTLAttributeDescriptorArray` (struct)

- `nsobject: NSObject`
- `objectAtIndexedSubscript :: (self: *MTLAttributeDescriptorArray, index: NSUInteger) -> *MTLAttributeDescriptor {`
- `setObject_atIndexedSubscript :: (self: *MTLAttributeDescriptorArray, attributeDesc: *MTLAttributeDescriptor, index: NSUInteger) -> () {`

#### `MTLStageInputOutputDescriptor` (struct)

- `nsobject: NSObject`
- `stageInputOutputDescriptor :: () -> *MTLStageInputOutputDescriptor {`
- `reset :: (self: *MTLStageInputOutputDescriptor) -> () {`
- `layouts :: (self: *MTLStageInputOutputDescriptor) -> *MTLBufferLayoutDescriptorArray {`
- `attributes :: (self: *MTLStageInputOutputDescriptor) -> *MTLAttributeDescriptorArray {`
- `indexType :: (self: *MTLStageInputOutputDescriptor) -> MTLIndexType {`
- `setIndexType :: (self: *MTLStageInputOutputDescriptor, indexType: MTLIndexType) -> () {`
- `indexBufferIndex :: (self: *MTLStageInputOutputDescriptor) -> NSUInteger {`
- `setIndexBufferIndex :: (self: *MTLStageInputOutputDescriptor, indexBufferIndex: NSUInteger) -> () {`

#### `MTLPipelineBufferDescriptor` (struct)

- `nsobject: NSObject`
- `mutability :: (self: *MTLPipelineBufferDescriptor) -> MTLMutability {`
- `setMutability :: (self: *MTLPipelineBufferDescriptor, mutability: MTLMutability) -> () {`

#### `MTLPipelineBufferDescriptorArray` (struct)

- `nsobject: NSObject`
- `objectAtIndexedSubscript :: (self: *MTLPipelineBufferDescriptorArray, bufferIndex: NSUInteger) -> *MTLPipelineBufferDescriptor {`
- `setObject_atIndexedSubscript :: (self: *MTLPipelineBufferDescriptorArray, buffer: *MTLPipelineBufferDescriptor, bufferIndex: NSUInteger) -> () {`

#### `MTLComputePipelineReflection` (struct)

- `nsobject: NSObject`
- `arguments :: (self: *MTLComputePipelineReflection) -> *NSArray(*MTLArgument) {`

#### `MTLComputePipelineDescriptor` (struct)

- `nsobject: NSObject`
- `reset :: (self: *MTLComputePipelineDescriptor) -> () {`
- `label :: (self: *MTLComputePipelineDescriptor) -> *NSString {`
- `setLabel :: (self: *MTLComputePipelineDescriptor, label: *NSString) -> () {`
- `computeFunction :: (self: *MTLComputePipelineDescriptor) -> id {`
- `setComputeFunction :: (self: *MTLComputePipelineDescriptor, computeFunction: id) -> () {`
- `threadGroupSizeIsMultipleOfThreadExecutionWidth :: (self: *MTLComputePipelineDescriptor) -> BOOL {`
- `setThreadGroupSizeIsMultipleOfThreadExecutionWidth :: (self: *MTLComputePipelineDescriptor, threadGroupSizeIsMultipleOfThreadExecutionWidth: BOOL) -> () {`
- `maxTotalThreadsPerThreadgroup :: (self: *MTLComputePipelineDescriptor) -> NSUInteger {`
- `setMaxTotalThreadsPerThreadgroup :: (self: *MTLComputePipelineDescriptor, maxTotalThreadsPerThreadgroup: NSUInteger) -> () {`
- `stageInputDescriptor :: (self: *MTLComputePipelineDescriptor) -> *MTLStageInputOutputDescriptor {`
- `setStageInputDescriptor :: (self: *MTLComputePipelineDescriptor, stageInputDescriptor: *MTLStageInputOutputDescriptor) -> () {`
- `buffers :: (self: *MTLComputePipelineDescriptor) -> *MTLPipelineBufferDescriptorArray {`

#### `MTLComputePipelineState` (struct)

- `label :: (self: *MTLComputePipelineState) -> *NSString {`
- `device :: (self: *MTLComputePipelineState) -> id {`
- `maxTotalThreadsPerThreadgroup :: (self: *MTLComputePipelineState) -> NSUInteger {`
- `threadExecutionWidth :: (self: *MTLComputePipelineState) -> NSUInteger {`
- `staticThreadgroupMemoryLength :: (self: *MTLComputePipelineState) -> NSUInteger {`

#### `MTLScissorRect` (struct)

- `x: NSUInteger`
- `y: NSUInteger`
- `width: NSUInteger`
- `height: NSUInteger`

#### `MTLViewport` (struct)

- `originX: float64`
- `originY: float64`
- `width: float64`
- `height: float64`
- `znear: float64`
- `zfar: float64`

#### `MTLDrawPrimitivesIndirectArguments` (struct)

- `vertexCount: uint32_t`
- `instanceCount: uint32_t`
- `vertexStart: uint32_t`
- `baseInstance: uint32_t`

#### `MTLDrawIndexedPrimitivesIndirectArguments` (struct)

- `indexCount: uint32_t`
- `instanceCount: uint32_t`
- `indexStart: uint32_t`
- `baseVertex: int32_t`
- `baseInstance: uint32_t`

#### `MTLDrawPatchIndirectArguments` (struct)

- `patchCount: uint32_t`
- `instanceCount: uint32_t`
- `patchStart: uint32_t`
- `baseInstance: uint32_t`

#### `MTLQuadTessellationFactorsHalf` (struct)

- `edgeTessellationFactor: [4] uint16_t`
- `insideTessellationFactor: [2] uint16_t`

#### `MTLTriangleTessellationFactorsHalf` (struct)

- `edgeTessellationFactor: [3] uint16_t`
- `insideTessellationFactor: uint16_t`

#### `MTLRenderCommandEncoder` (struct)

- `setRenderPipelineState :: (self: *MTLRenderCommandEncoder, pipelineState: id) -> () {`
- `setVertexBytes_length_atIndex :: (self: *MTLRenderCommandEncoder, bytes: *void, length: NSUInteger, index: NSUInteger) -> () {`
- `setVertexBuffer_offset_atIndex :: (self: *MTLRenderCommandEncoder, buffer: id, offset: NSUInteger, index: NSUInteger) -> () {`
- `setVertexBufferOffset_atIndex :: (self: *MTLRenderCommandEncoder, offset: NSUInteger, index: NSUInteger) -> () {`
- `setVertexBuffers_offsets_withRange :: (self: *MTLRenderCommandEncoder, buffers: *id, offsets: *NSUInteger, range: NSRange) -> () {`
- `setVertexTexture_atIndex :: (self: *MTLRenderCommandEncoder, texture: id, index: NSUInteger) -> () {`
- `setVertexTextures_withRange :: (self: *MTLRenderCommandEncoder, textures: *id, range: NSRange) -> () {`
- `setVertexSamplerState_atIndex :: (self: *MTLRenderCommandEncoder, sampler: id, index: NSUInteger) -> () {`
- `setVertexSamplerStates_withRange :: (self: *MTLRenderCommandEncoder, samplers: *id, range: NSRange) -> () {`
- `setVertexSamplerState_lodMinClamp_lodMaxClamp_atIndex :: (self: *MTLRenderCommandEncoder, sampler: id, lodMinClamp: float32, lodMaxClamp: float32, index: NSUInteger) -> () {`
- `setVertexSamplerStates_lodMinClamps_lodMaxClamps_withRange :: (self: *MTLRenderCommandEncoder, samplers: *id, lodMinClamps: *float32, lodMaxClamps: *float32, range: NSRange) -> () {`
- `setViewport :: (self: *MTLRenderCommandEncoder, viewport: MTLViewport) -> () {`
- `setViewports_count :: (self: *MTLRenderCommandEncoder, viewports: *MTLViewport, count: NSUInteger) -> () {`
- `setFrontFacingWinding :: (self: *MTLRenderCommandEncoder, frontFacingWinding: MTLWinding) -> () {`
- `setCullMode :: (self: *MTLRenderCommandEncoder, cullMode: MTLCullMode) -> () {`
- `setDepthClipMode :: (self: *MTLRenderCommandEncoder, depthClipMode: MTLDepthClipMode) -> () {`
- `setDepthBias_slopeScale_clamp :: (self: *MTLRenderCommandEncoder, depthBias: float32, slopeScale: float32, clamp: float32) -> () {`
- `setScissorRect :: (self: *MTLRenderCommandEncoder, rect: MTLScissorRect) -> () {`
- `setScissorRects_count :: (self: *MTLRenderCommandEncoder, scissorRects: *MTLScissorRect, count: NSUInteger) -> () {`
- `setTriangleFillMode :: (self: *MTLRenderCommandEncoder, fillMode: MTLTriangleFillMode) -> () {`
- `setFragmentBytes_length_atIndex :: (self: *MTLRenderCommandEncoder, bytes: *void, length: NSUInteger, index: NSUInteger) -> () {`
- `setFragmentBuffer_offset_atIndex :: (self: *MTLRenderCommandEncoder, buffer: id, offset: NSUInteger, index: NSUInteger) -> () {`
- `setFragmentBufferOffset_atIndex :: (self: *MTLRenderCommandEncoder, offset: NSUInteger, index: NSUInteger) -> () {`
- `setFragmentBuffers_offsets_withRange :: (self: *MTLRenderCommandEncoder, buffers: *id, offsets: *NSUInteger, range: NSRange) -> () {`
- `setFragmentTexture_atIndex :: (self: *MTLRenderCommandEncoder, texture: id, index: NSUInteger) -> () {`
- `setFragmentTextures_withRange :: (self: *MTLRenderCommandEncoder, textures: *id, range: NSRange) -> () {`
- `setFragmentSamplerState_atIndex :: (self: *MTLRenderCommandEncoder, sampler: id, index: NSUInteger) -> () {`
- `setFragmentSamplerStates_withRange :: (self: *MTLRenderCommandEncoder, samplers: *id, range: NSRange) -> () {`
- `setFragmentSamplerState_lodMinClamp_lodMaxClamp_atIndex :: (self: *MTLRenderCommandEncoder, sampler: id, lodMinClamp: float32, lodMaxClamp: float32, index: NSUInteger) -> () {`
- `setFragmentSamplerStates_lodMinClamps_lodMaxClamps_withRange :: (self: *MTLRenderCommandEncoder, samplers: *id, lodMinClamps: *float32, lodMaxClamps: *float32, range: NSRange) -> () {`
- `setBlendColorRed_green_blue_alpha :: (self: *MTLRenderCommandEncoder, red: float32, green: float32, blue: float32, alpha: float32) -> () {`
- `setDepthStencilState :: (self: *MTLRenderCommandEncoder, depthStencilState: id) -> () {`
- `setStencilReferenceValue :: (self: *MTLRenderCommandEncoder, referenceValue: uint32_t) -> () {`
- `setStencilFrontReferenceValue_backReferenceValue :: (self: *MTLRenderCommandEncoder, frontReferenceValue: uint32_t, backReferenceValue: uint32_t) -> () {`
- `setVisibilityResultMode_offset :: (self: *MTLRenderCommandEncoder, mode: MTLVisibilityResultMode, offset: NSUInteger) -> () {`
- `setColorStoreAction_atIndex :: (self: *MTLRenderCommandEncoder, storeAction: MTLStoreAction, colorAttachmentIndex: NSUInteger) -> () {`
- `setDepthStoreAction :: (self: *MTLRenderCommandEncoder, storeAction: MTLStoreAction) -> () {`
- `setStencilStoreAction :: (self: *MTLRenderCommandEncoder, storeAction: MTLStoreAction) -> () {`
- `setColorStoreActionOptions_atIndex :: (self: *MTLRenderCommandEncoder, storeActionOptions: MTLStoreActionOptions, colorAttachmentIndex: NSUInteger) -> () {`
- `setDepthStoreActionOptions :: (self: *MTLRenderCommandEncoder, storeActionOptions: MTLStoreActionOptions) -> () {`
- `setStencilStoreActionOptions :: (self: *MTLRenderCommandEncoder, storeActionOptions: MTLStoreActionOptions) -> () {`
- `drawPrimitives_vertexStart_vertexCount_instanceCount :: (self: *MTLRenderCommandEncoder, primitiveType: MTLPrimitiveType, vertexStart: NSUInteger, vertexCount: NSUInteger, instanceCount: NSUInteger) -> () {`
- `drawPrimitives_vertexStart_vertexCount :: (self: *MTLRenderCommandEncoder, primitiveType: MTLPrimitiveType, vertexStart: NSUInteger, vertexCount: NSUInteger) -> () {`
- `drawIndexedPrimitives_indexCount_indexType_indexBuffer_indexBufferOffset_instanceCount :: (self: *MTLRenderCommandEncoder, primitiveType: MTLPrimitiveType, indexCount: NSUInteger, indexType: MTLIndexType, indexBuffer: id, indexBufferOffset: NSUInteger, instanceCount: NSUInteger) -> () {`
- `drawIndexedPrimitives_indexCount_indexType_indexBuffer_indexBufferOffset :: (self: *MTLRenderCommandEncoder, primitiveType: MTLPrimitiveType, indexCount: NSUInteger, indexType: MTLIndexType, indexBuffer: id, indexBufferOffset: NSUInteger) -> () {`
- `drawPrimitives_vertexStart_vertexCount_instanceCount_baseInstance :: (self: *MTLRenderCommandEncoder, primitiveType: MTLPrimitiveType, vertexStart: NSUInteger, vertexCount: NSUInteger, instanceCount: NSUInteger, baseInstance: NSUInteger) -> () {`
- `drawIndexedPrimitives_indexCount_indexType_indexBuffer_indexBufferOffset_instanceCount_baseVertex_baseInstance :: (self: *MTLRenderCommandEncoder, primitiveType: MTLPrimitiveType, indexCount: NSUInteger, indexType: MTLIndexType, indexBuffer: id, indexBufferOffset: NSUInteger, instanceCount: NSUInteger, baseVertex: NSInteger, baseInstance: NSUInteger) -> () {`
- `drawPrimitives_indirectBuffer_indirectBufferOffset :: (self: *MTLRenderCommandEncoder, primitiveType: MTLPrimitiveType, indirectBuffer: id, indirectBufferOffset: NSUInteger) -> () {`
- `drawIndexedPrimitives_indexType_indexBuffer_indexBufferOffset_indirectBuffer_indirectBufferOffset :: (self: *MTLRenderCommandEncoder, primitiveType: MTLPrimitiveType, indexType: MTLIndexType, indexBuffer: id, indexBufferOffset: NSUInteger, indirectBuffer: id, indirectBufferOffset: NSUInteger) -> () {`
- `textureBarrier :: (self: *MTLRenderCommandEncoder) -> () {`
- `updateFence_afterStages :: (self: *MTLRenderCommandEncoder, fence: id, stages: MTLRenderStages) -> () {`
- `waitForFence_beforeStages :: (self: *MTLRenderCommandEncoder, fence: id, stages: MTLRenderStages) -> () {`
- `setTessellationFactorBuffer_offset_instanceStride :: (self: *MTLRenderCommandEncoder, buffer: id, offset: NSUInteger, instanceStride: NSUInteger) -> () {`
- `setTessellationFactorScale :: (self: *MTLRenderCommandEncoder, scale: float32) -> () {`
- `drawPatches_patchStart_patchCount_patchIndexBuffer_patchIndexBufferOffset_instanceCount_baseInstance :: (self: *MTLRenderCommandEncoder, numberOfPatchControlPoints: NSUInteger, patchStart: NSUInteger, patchCount: NSUInteger, patchIndexBuffer: id, patchIndexBufferOffset: NSUInteger, instanceCount: NSUInteger, baseInstance: NSUInteger) -> () {`
- `drawPatches_patchIndexBuffer_patchIndexBufferOffset_indirectBuffer_indirectBufferOffset :: (self: *MTLRenderCommandEncoder, numberOfPatchControlPoints: NSUInteger, patchIndexBuffer: id, patchIndexBufferOffset: NSUInteger, indirectBuffer: id, indirectBufferOffset: NSUInteger) -> () {`
- `drawIndexedPatches_patchStart_patchCount_patchIndexBuffer_patchIndexBufferOffset_controlPointIndexBuffer_controlPointIndexBufferOffset_instanceCount_baseInstance :: (self: *MTLRenderCommandEncoder, numberOfPatchControlPoints: NSUInteger, patchStart: NSUInteger, patchCount: NSUInteger, patchIndexBuffer: id, patchIndexBufferOffset: NSUInteger, controlPointIndexBuffer: id, controlPointIndexBufferOffset: NSUInteger, instanceCount: NSUInteger, baseInstance: NSUInteger) -> () {`
- `drawIndexedPatches_patchIndexBuffer_patchIndexBufferOffset_controlPointIndexBuffer_controlPointIndexBufferOffset_indirectBuffer_indirectBufferOffset :: (self: *MTLRenderCommandEncoder, numberOfPatchControlPoints: NSUInteger, patchIndexBuffer: id, patchIndexBufferOffset: NSUInteger, controlPointIndexBuffer: id, controlPointIndexBufferOffset: NSUInteger, indirectBuffer: id, indirectBufferOffset: NSUInteger) -> () {`
- `useResource_usage :: (self: *MTLRenderCommandEncoder, resource: id, usage: MTLResourceUsage) -> () {`
- `useResources_count_usage :: (self: *MTLRenderCommandEncoder, resources: *id, count: NSUInteger, usage: MTLResourceUsage) -> () {`
- `useHeap :: (self: *MTLRenderCommandEncoder, heap: id) -> () {`
- `useHeaps_count :: (self: *MTLRenderCommandEncoder, heaps: *id, count: NSUInteger) -> () {`
- `executeCommandsInBuffer_withRange :: (self: *MTLRenderCommandEncoder, indirectCommandBuffer: id, executionRange: NSRange) -> () {`
- `executeCommandsInBuffer_indirectBuffer_indirectBufferOffset :: (self: *MTLRenderCommandEncoder, indirectCommandbuffer: id, indirectRangeBuffer: id, indirectBufferOffset: NSUInteger) -> () {`
- `memoryBarrierWithScope_afterStages_beforeStages :: (self: *MTLRenderCommandEncoder, scope: MTLBarrierScope, after: MTLRenderStages, before: MTLRenderStages) -> () {`
- `memoryBarrierWithResources_count_afterStages_beforeStages :: (self: *MTLRenderCommandEncoder, resources: *id, count: NSUInteger, after: MTLRenderStages, before: MTLRenderStages) -> () {`

#### `MTLFunctionConstantValues` (struct)

- `nsobject: NSObject`
- `setConstantValue_type_atIndex :: (self: *MTLFunctionConstantValues, value: *void, type: MTLDataType, index: NSUInteger) -> () {`
- `setConstantValues_type_withRange :: (self: *MTLFunctionConstantValues, values: *void, type: MTLDataType, range: NSRange) -> () {`
- `setConstantValue_type_withName :: (self: *MTLFunctionConstantValues, value: *void, type: MTLDataType, name: *NSString) -> () {`
- `reset :: (self: *MTLFunctionConstantValues) -> () {`

#### `MTLRenderPipelineColorAttachmentDescriptor` (struct)

- `nsobject: NSObject`
- `pixelFormat :: (self: *MTLRenderPipelineColorAttachmentDescriptor) -> MTLPixelFormat {`
- `setPixelFormat :: (self: *MTLRenderPipelineColorAttachmentDescriptor, pixelFormat: MTLPixelFormat) -> () {`
- `isBlendingEnabled :: (self: *MTLRenderPipelineColorAttachmentDescriptor) -> BOOL {`
- `setBlendingEnabled :: (self: *MTLRenderPipelineColorAttachmentDescriptor, blendingEnabled: BOOL) -> () {`
- `sourceRGBBlendFactor :: (self: *MTLRenderPipelineColorAttachmentDescriptor) -> MTLBlendFactor {`
- `setSourceRGBBlendFactor :: (self: *MTLRenderPipelineColorAttachmentDescriptor, sourceRGBBlendFactor: MTLBlendFactor) -> () {`
- `destinationRGBBlendFactor :: (self: *MTLRenderPipelineColorAttachmentDescriptor) -> MTLBlendFactor {`
- `setDestinationRGBBlendFactor :: (self: *MTLRenderPipelineColorAttachmentDescriptor, destinationRGBBlendFactor: MTLBlendFactor) -> () {`
- `rgbBlendOperation :: (self: *MTLRenderPipelineColorAttachmentDescriptor) -> MTLBlendOperation {`
- `setRgbBlendOperation :: (self: *MTLRenderPipelineColorAttachmentDescriptor, rgbBlendOperation: MTLBlendOperation) -> () {`
- `sourceAlphaBlendFactor :: (self: *MTLRenderPipelineColorAttachmentDescriptor) -> MTLBlendFactor {`
- `setSourceAlphaBlendFactor :: (self: *MTLRenderPipelineColorAttachmentDescriptor, sourceAlphaBlendFactor: MTLBlendFactor) -> () {`
- `destinationAlphaBlendFactor :: (self: *MTLRenderPipelineColorAttachmentDescriptor) -> MTLBlendFactor {`
- `setDestinationAlphaBlendFactor :: (self: *MTLRenderPipelineColorAttachmentDescriptor, destinationAlphaBlendFactor: MTLBlendFactor) -> () {`
- `alphaBlendOperation :: (self: *MTLRenderPipelineColorAttachmentDescriptor) -> MTLBlendOperation {`
- `setAlphaBlendOperation :: (self: *MTLRenderPipelineColorAttachmentDescriptor, alphaBlendOperation: MTLBlendOperation) -> () {`
- `writeMask :: (self: *MTLRenderPipelineColorAttachmentDescriptor) -> MTLColorWriteMask {`
- `setWriteMask :: (self: *MTLRenderPipelineColorAttachmentDescriptor, writeMask: MTLColorWriteMask) -> () {`

#### `MTLRenderPipelineReflection` (struct)

- `nsobject: NSObject`
- `vertexArguments :: (self: *MTLRenderPipelineReflection) -> *NSArray(*MTLArgument) {`
- `fragmentArguments :: (self: *MTLRenderPipelineReflection) -> *NSArray(*MTLArgument) {`

#### `MTLRenderPipelineDescriptor` (struct)

- `nsobject: NSObject`
- `reset :: (self: *MTLRenderPipelineDescriptor) -> () {`
- `label :: (self: *MTLRenderPipelineDescriptor) -> *NSString {`
- `setLabel :: (self: *MTLRenderPipelineDescriptor, label: *NSString) -> () {`
- `vertexFunction :: (self: *MTLRenderPipelineDescriptor) -> id {`
- `setVertexFunction :: (self: *MTLRenderPipelineDescriptor, vertexFunction: id) -> () {`
- `fragmentFunction :: (self: *MTLRenderPipelineDescriptor) -> id {`
- `setFragmentFunction :: (self: *MTLRenderPipelineDescriptor, fragmentFunction: id) -> () {`
- `vertexDescriptor :: (self: *MTLRenderPipelineDescriptor) -> *MTLVertexDescriptor {`
- `setVertexDescriptor :: (self: *MTLRenderPipelineDescriptor, vertexDescriptor: *MTLVertexDescriptor) -> () {`
- `sampleCount :: (self: *MTLRenderPipelineDescriptor) -> NSUInteger {`
- `setSampleCount :: (self: *MTLRenderPipelineDescriptor, sampleCount: NSUInteger) -> () {`
- `rasterSampleCount :: (self: *MTLRenderPipelineDescriptor) -> NSUInteger {`
- `setRasterSampleCount :: (self: *MTLRenderPipelineDescriptor, rasterSampleCount: NSUInteger) -> () {`
- `isAlphaToCoverageEnabled :: (self: *MTLRenderPipelineDescriptor) -> BOOL {`
- `setAlphaToCoverageEnabled :: (self: *MTLRenderPipelineDescriptor, alphaToCoverageEnabled: BOOL) -> () {`
- `isAlphaToOneEnabled :: (self: *MTLRenderPipelineDescriptor) -> BOOL {`
- `setAlphaToOneEnabled :: (self: *MTLRenderPipelineDescriptor, alphaToOneEnabled: BOOL) -> () {`
- `isRasterizationEnabled :: (self: *MTLRenderPipelineDescriptor) -> BOOL {`
- `setRasterizationEnabled :: (self: *MTLRenderPipelineDescriptor, rasterizationEnabled: BOOL) -> () {`
- `colorAttachments :: (self: *MTLRenderPipelineDescriptor) -> *MTLRenderPipelineColorAttachmentDescriptorArray {`
- `depthAttachmentPixelFormat :: (self: *MTLRenderPipelineDescriptor) -> MTLPixelFormat {`
- `setDepthAttachmentPixelFormat :: (self: *MTLRenderPipelineDescriptor, depthAttachmentPixelFormat: MTLPixelFormat) -> () {`
- `stencilAttachmentPixelFormat :: (self: *MTLRenderPipelineDescriptor) -> MTLPixelFormat {`
- `setStencilAttachmentPixelFormat :: (self: *MTLRenderPipelineDescriptor, stencilAttachmentPixelFormat: MTLPixelFormat) -> () {`
- `inputPrimitiveTopology :: (self: *MTLRenderPipelineDescriptor) -> MTLPrimitiveTopologyClass {`
- `setInputPrimitiveTopology :: (self: *MTLRenderPipelineDescriptor, inputPrimitiveTopology: MTLPrimitiveTopologyClass) -> () {`
- `tessellationPartitionMode :: (self: *MTLRenderPipelineDescriptor) -> MTLTessellationPartitionMode {`
- `setTessellationPartitionMode :: (self: *MTLRenderPipelineDescriptor, tessellationPartitionMode: MTLTessellationPartitionMode) -> () {`
- `maxTessellationFactor :: (self: *MTLRenderPipelineDescriptor) -> NSUInteger {`
- `setMaxTessellationFactor :: (self: *MTLRenderPipelineDescriptor, maxTessellationFactor: NSUInteger) -> () {`
- `isTessellationFactorScaleEnabled :: (self: *MTLRenderPipelineDescriptor) -> BOOL {`
- `setTessellationFactorScaleEnabled :: (self: *MTLRenderPipelineDescriptor, tessellationFactorScaleEnabled: BOOL) -> () {`
- `tessellationFactorFormat :: (self: *MTLRenderPipelineDescriptor) -> MTLTessellationFactorFormat {`
- `setTessellationFactorFormat :: (self: *MTLRenderPipelineDescriptor, tessellationFactorFormat: MTLTessellationFactorFormat) -> () {`
- `tessellationControlPointIndexType :: (self: *MTLRenderPipelineDescriptor) -> MTLTessellationControlPointIndexType {`
- `setTessellationControlPointIndexType :: (self: *MTLRenderPipelineDescriptor, tessellationControlPointIndexType: MTLTessellationControlPointIndexType) -> () {`
- `tessellationFactorStepFunction :: (self: *MTLRenderPipelineDescriptor) -> MTLTessellationFactorStepFunction {`
- `setTessellationFactorStepFunction :: (self: *MTLRenderPipelineDescriptor, tessellationFactorStepFunction: MTLTessellationFactorStepFunction) -> () {`
- `tessellationOutputWindingOrder :: (self: *MTLRenderPipelineDescriptor) -> MTLWinding {`
- `setTessellationOutputWindingOrder :: (self: *MTLRenderPipelineDescriptor, tessellationOutputWindingOrder: MTLWinding) -> () {`
- `vertexBuffers :: (self: *MTLRenderPipelineDescriptor) -> *MTLPipelineBufferDescriptorArray {`
- `fragmentBuffers :: (self: *MTLRenderPipelineDescriptor) -> *MTLPipelineBufferDescriptorArray {`
- `supportIndirectCommandBuffers :: (self: *MTLRenderPipelineDescriptor) -> BOOL {`
- `setSupportIndirectCommandBuffers :: (self: *MTLRenderPipelineDescriptor, supportIndirectCommandBuffers: BOOL) -> () {`

#### `MTLRenderPipelineState` (struct)

- `label :: (self: *MTLRenderPipelineState) -> *NSString {`
- `device :: (self: *MTLRenderPipelineState) -> id {`
- `supportIndirectCommandBuffers :: (self: *MTLRenderPipelineState) -> BOOL {`

#### `MTLRenderPipelineColorAttachmentDescriptorArray` (struct)

- `nsobject: NSObject`
- `objectAtIndexedSubscript :: (self: *MTLRenderPipelineColorAttachmentDescriptorArray, attachmentIndex: NSUInteger) -> *MTLRenderPipelineColorAttachmentDescriptor {`
- `setObject_atIndexedSubscript :: (self: *MTLRenderPipelineColorAttachmentDescriptorArray, attachment: *MTLRenderPipelineColorAttachmentDescriptor, attachmentIndex: NSUInteger) -> () {`

#### `MTLVertexBufferLayoutDescriptor` (struct)

- `nsobject: NSObject`
- `stride :: (self: *MTLVertexBufferLayoutDescriptor) -> NSUInteger {`
- `setStride :: (self: *MTLVertexBufferLayoutDescriptor, stride: NSUInteger) -> () {`
- `stepFunction :: (self: *MTLVertexBufferLayoutDescriptor) -> MTLVertexStepFunction {`
- `setStepFunction :: (self: *MTLVertexBufferLayoutDescriptor, stepFunction: MTLVertexStepFunction) -> () {`
- `stepRate :: (self: *MTLVertexBufferLayoutDescriptor) -> NSUInteger {`
- `setStepRate :: (self: *MTLVertexBufferLayoutDescriptor, stepRate: NSUInteger) -> () {`

#### `MTLVertexBufferLayoutDescriptorArray` (struct)

- `nsobject: NSObject`
- `objectAtIndexedSubscript :: (self: *MTLVertexBufferLayoutDescriptorArray, index: NSUInteger) -> *MTLVertexBufferLayoutDescriptor {`
- `setObject_atIndexedSubscript :: (self: *MTLVertexBufferLayoutDescriptorArray, bufferDesc: *MTLVertexBufferLayoutDescriptor, index: NSUInteger) -> () {`

#### `MTLVertexAttributeDescriptor` (struct)

- `nsobject: NSObject`
- `format :: (self: *MTLVertexAttributeDescriptor) -> MTLVertexFormat {`
- `setFormat :: (self: *MTLVertexAttributeDescriptor, format: MTLVertexFormat) -> () {`
- `offset :: (self: *MTLVertexAttributeDescriptor) -> NSUInteger {`
- `setOffset :: (self: *MTLVertexAttributeDescriptor, offset: NSUInteger) -> () {`
- `bufferIndex :: (self: *MTLVertexAttributeDescriptor) -> NSUInteger {`
- `setBufferIndex :: (self: *MTLVertexAttributeDescriptor, bufferIndex: NSUInteger) -> () {`

#### `MTLVertexAttributeDescriptorArray` (struct)

- `nsobject: NSObject`
- `objectAtIndexedSubscript :: (self: *MTLVertexAttributeDescriptorArray, index: NSUInteger) -> *MTLVertexAttributeDescriptor {`
- `setObject_atIndexedSubscript :: (self: *MTLVertexAttributeDescriptorArray, attributeDesc: *MTLVertexAttributeDescriptor, index: NSUInteger) -> () {`

#### `MTLVertexDescriptor` (struct)

- `nsobject: NSObject`
- `vertexDescriptor :: () -> *MTLVertexDescriptor {`
- `reset :: (self: *MTLVertexDescriptor) -> () {`
- `layouts :: (self: *MTLVertexDescriptor) -> *MTLVertexBufferLayoutDescriptorArray {`
- `attributes :: (self: *MTLVertexDescriptor) -> *MTLVertexAttributeDescriptorArray {`

#### `MTLParallelRenderCommandEncoder` (struct)

- `renderCommandEncoder :: (self: *MTLParallelRenderCommandEncoder) -> id {`
- `setColorStoreAction_atIndex :: (self: *MTLParallelRenderCommandEncoder, storeAction: MTLStoreAction, colorAttachmentIndex: NSUInteger) -> () {`
- `setDepthStoreAction :: (self: *MTLParallelRenderCommandEncoder, storeAction: MTLStoreAction) -> () {`
- `setStencilStoreAction :: (self: *MTLParallelRenderCommandEncoder, storeAction: MTLStoreAction) -> () {`
- `setColorStoreActionOptions_atIndex :: (self: *MTLParallelRenderCommandEncoder, storeActionOptions: MTLStoreActionOptions, colorAttachmentIndex: NSUInteger) -> () {`
- `setDepthStoreActionOptions :: (self: *MTLParallelRenderCommandEncoder, storeActionOptions: MTLStoreActionOptions) -> () {`
- `setStencilStoreActionOptions :: (self: *MTLParallelRenderCommandEncoder, storeActionOptions: MTLStoreActionOptions) -> () {`

#### `MTLSamplerDescriptor` (struct)

- `nsobject: NSObject`
- `minFilter :: (self: *MTLSamplerDescriptor) -> MTLSamplerMinMagFilter {`
- `setMinFilter :: (self: *MTLSamplerDescriptor, minFilter: MTLSamplerMinMagFilter) -> () {`
- `magFilter :: (self: *MTLSamplerDescriptor) -> MTLSamplerMinMagFilter {`
- `setMagFilter :: (self: *MTLSamplerDescriptor, magFilter: MTLSamplerMinMagFilter) -> () {`
- `mipFilter :: (self: *MTLSamplerDescriptor) -> MTLSamplerMipFilter {`
- `setMipFilter :: (self: *MTLSamplerDescriptor, mipFilter: MTLSamplerMipFilter) -> () {`
- `maxAnisotropy :: (self: *MTLSamplerDescriptor) -> NSUInteger {`
- `setMaxAnisotropy :: (self: *MTLSamplerDescriptor, maxAnisotropy: NSUInteger) -> () {`
- `sAddressMode :: (self: *MTLSamplerDescriptor) -> MTLSamplerAddressMode {`
- `setSAddressMode :: (self: *MTLSamplerDescriptor, sAddressMode: MTLSamplerAddressMode) -> () {`
- `tAddressMode :: (self: *MTLSamplerDescriptor) -> MTLSamplerAddressMode {`
- `setTAddressMode :: (self: *MTLSamplerDescriptor, tAddressMode: MTLSamplerAddressMode) -> () {`
- `rAddressMode :: (self: *MTLSamplerDescriptor) -> MTLSamplerAddressMode {`
- `setRAddressMode :: (self: *MTLSamplerDescriptor, rAddressMode: MTLSamplerAddressMode) -> () {`
- `borderColor :: (self: *MTLSamplerDescriptor) -> MTLSamplerBorderColor {`
- `setBorderColor :: (self: *MTLSamplerDescriptor, borderColor: MTLSamplerBorderColor) -> () {`
- `normalizedCoordinates :: (self: *MTLSamplerDescriptor) -> BOOL {`
- `setNormalizedCoordinates :: (self: *MTLSamplerDescriptor, normalizedCoordinates: BOOL) -> () {`
- `lodMinClamp :: (self: *MTLSamplerDescriptor) -> float32 {`
- `setLodMinClamp :: (self: *MTLSamplerDescriptor, lodMinClamp: float32) -> () {`
- `lodMaxClamp :: (self: *MTLSamplerDescriptor) -> float32 {`
- `setLodMaxClamp :: (self: *MTLSamplerDescriptor, lodMaxClamp: float32) -> () {`
- `lodAverage :: (self: *MTLSamplerDescriptor) -> BOOL {`
- `setLodAverage :: (self: *MTLSamplerDescriptor, lodAverage: BOOL) -> () {`
- `compareFunction :: (self: *MTLSamplerDescriptor) -> MTLCompareFunction {`
- `setCompareFunction :: (self: *MTLSamplerDescriptor, compareFunction: MTLCompareFunction) -> () {`
- `supportArgumentBuffers :: (self: *MTLSamplerDescriptor) -> BOOL {`
- `setSupportArgumentBuffers :: (self: *MTLSamplerDescriptor, supportArgumentBuffers: BOOL) -> () {`
- `label :: (self: *MTLSamplerDescriptor) -> *NSString {`
- `setLabel :: (self: *MTLSamplerDescriptor, label: *NSString) -> () {`

#### `MTLSamplerState` (struct)

- `label :: (self: *MTLSamplerState) -> *NSString {`
- `device :: (self: *MTLSamplerState) -> id {`

#### `MTLHeapDescriptor` (struct)

- `nsobject: NSObject`
- `size :: (self: *MTLHeapDescriptor) -> NSUInteger {`
- `setSize :: (self: *MTLHeapDescriptor, size: NSUInteger) -> () {`
- `storageMode :: (self: *MTLHeapDescriptor) -> MTLStorageMode {`
- `setStorageMode :: (self: *MTLHeapDescriptor, storageMode: MTLStorageMode) -> () {`
- `cpuCacheMode :: (self: *MTLHeapDescriptor) -> MTLCPUCacheMode {`
- `setCpuCacheMode :: (self: *MTLHeapDescriptor, cpuCacheMode: MTLCPUCacheMode) -> () {`

#### `MTLHeap` (struct)

- `maxAvailableSizeWithAlignment :: (self: *MTLHeap, alignment: NSUInteger) -> NSUInteger {`
- `newBufferWithLength_options :: (self: *MTLHeap, length: NSUInteger, options: MTLResourceOptions) -> *MTLBuffer {`
- `newTextureWithDescriptor :: (self: *MTLHeap, desc: *MTLTextureDescriptor) -> id {`
- `setPurgeableState :: (self: *MTLHeap, state: MTLPurgeableState) -> MTLPurgeableState {`
- `label :: (self: *MTLHeap) -> *NSString {`
- `setLabel :: (self: *MTLHeap, label: *NSString) -> () {`
- `device :: (self: *MTLHeap) -> id {`
- `storageMode :: (self: *MTLHeap) -> MTLStorageMode {`
- `cpuCacheMode :: (self: *MTLHeap) -> MTLCPUCacheMode {`
- `size :: (self: *MTLHeap) -> NSUInteger {`
- `usedSize :: (self: *MTLHeap) -> NSUInteger {`
- `currentAllocatedSize :: (self: *MTLHeap) -> NSUInteger {`

#### `MTLArgumentEncoder` (struct)

- `setArgumentBuffer_offset :: (self: *MTLArgumentEncoder, argumentBuffer: id, offset: NSUInteger) -> () {`
- `setArgumentBuffer_startOffset_arrayElement :: (self: *MTLArgumentEncoder, argumentBuffer: id, startOffset: NSUInteger, arrayElement: NSUInteger) -> () {`
- `setBuffer_offset_atIndex :: (self: *MTLArgumentEncoder, buffer: id, offset: NSUInteger, index: NSUInteger) -> () {`
- `setBuffers_offsets_withRange :: (self: *MTLArgumentEncoder, buffers: *id, offsets: *NSUInteger, range: NSRange) -> () {`
- `setTexture_atIndex :: (self: *MTLArgumentEncoder, texture: id, index: NSUInteger) -> () {`
- `setTextures_withRange :: (self: *MTLArgumentEncoder, textures: *id, range: NSRange) -> () {`
- `setSamplerState_atIndex :: (self: *MTLArgumentEncoder, sampler: id, index: NSUInteger) -> () {`
- `setSamplerStates_withRange :: (self: *MTLArgumentEncoder, samplers: *id, range: NSRange) -> () {`
- `constantDataAtIndex :: (self: *MTLArgumentEncoder, index: NSUInteger) -> *void {`
- `setRenderPipelineState_atIndex :: (self: *MTLArgumentEncoder, pipeline: id, index: NSUInteger) -> () {`
- `setRenderPipelineStates_withRange :: (self: *MTLArgumentEncoder, pipelines: *id, range: NSRange) -> () {`
- `setIndirectCommandBuffer_atIndex :: (self: *MTLArgumentEncoder, indirectCommandBuffer: id, index: NSUInteger) -> () {`
- `setIndirectCommandBuffers_withRange :: (self: *MTLArgumentEncoder, buffers: *id, range: NSRange) -> () {`
- `newArgumentEncoderForBufferAtIndex :: (self: *MTLArgumentEncoder, index: NSUInteger) -> id {`
- `device :: (self: *MTLArgumentEncoder) -> id {`
- `label :: (self: *MTLArgumentEncoder) -> *NSString {`
- `setLabel :: (self: *MTLArgumentEncoder, label: *NSString) -> () {`
- `encodedLength :: (self: *MTLArgumentEncoder) -> NSUInteger {`
- `alignment :: (self: *MTLArgumentEncoder) -> NSUInteger {`

#### `MTLCaptureManager` (struct)

- `nsobject: NSObject`
- `_isCapturing: BOOL`
- `_defaultCaptureScope: id`
- `sharedCaptureManager :: () -> *MTLCaptureManager {`
- `init :: (self: *MTLCaptureManager) -> *MTLCaptureManager {`
- `newCaptureScopeWithDevice :: (self: *MTLCaptureManager, device: id) -> id {`
- `newCaptureScopeWithCommandQueue :: (self: *MTLCaptureManager, commandQueue: id) -> id {`
- `startCaptureWithDevice :: (self: *MTLCaptureManager, device: id) -> () {`
- `startCaptureWithCommandQueue :: (self: *MTLCaptureManager, commandQueue: id) -> () {`
- `startCaptureWithScope :: (self: *MTLCaptureManager, captureScope: id) -> () {`
- `stopCapture :: (self: *MTLCaptureManager) -> () {`
- `defaultCaptureScope :: (self: *MTLCaptureManager) -> id {`
- `setDefaultCaptureScope :: (self: *MTLCaptureManager, defaultCaptureScope: id) -> () {`
- `isCapturing :: (self: *MTLCaptureManager) -> BOOL {`

#### `MTLCaptureScope` (struct)

- `beginScope :: (self: *MTLCaptureScope) -> () {`
- `endScope :: (self: *MTLCaptureScope) -> () {`
- `label :: (self: *MTLCaptureScope) -> *NSString {`
- `setLabel :: (self: *MTLCaptureScope, label: *NSString) -> () {`
- `device :: (self: *MTLCaptureScope) -> id {`
- `commandQueue :: (self: *MTLCaptureScope) -> id {`

#### `MTLIndirectRenderCommand` (struct)

- `setRenderPipelineState :: (self: *MTLIndirectRenderCommand, pipelineState: id) -> () {`
- `setVertexBuffer_offset_atIndex :: (self: *MTLIndirectRenderCommand, buffer: id, offset: NSUInteger, index: NSUInteger) -> () {`
- `setFragmentBuffer_offset_atIndex :: (self: *MTLIndirectRenderCommand, buffer: id, offset: NSUInteger, index: NSUInteger) -> () {`
- `drawPatches_patchStart_patchCount_patchIndexBuffer_patchIndexBufferOffset_instanceCount_baseInstance_tessellationFactorBuffer_tessellationFactorBufferOffset_tessellationFactorBufferInstanceStride :: (self: *MTLIndirectRenderCommand, numberOfPatchControlPoints: NSUInteger, patchStart: NSUInteger, patchCount: NSUInteger, patchIndexBuffer: id, patchIndexBufferOffset: NSUInteger, instanceCount: NSUInteger, baseInstance: NSUInteger, buffer: id, offset: NSUInteger, instanceStride: NSUInteger) -> () {`
- `drawIndexedPatches_patchStart_patchCount_patchIndexBuffer_patchIndexBufferOffset_controlPointIndexBuffer_controlPointIndexBufferOffset_instanceCount_baseInstance_tessellationFactorBuffer_tessellationFactorBufferOffset_tessellationFactorBufferInstanceStride :: (self: *MTLIndirectRenderCommand, numberOfPatchControlPoints: NSUInteger, patchStart: NSUInteger, patchCount: NSUInteger, patchIndexBuffer: id, patchIndexBufferOffset: NSUInteger, controlPointIndexBuffer: id, controlPointIndexBufferOffset: NSUInteger, instanceCount: NSUInteger, baseInstance: NSUInteger, buffer: id, offset: NSUInteger, instanceStride: NSUInteger) -> () {`
- `drawPrimitives_vertexStart_vertexCount_instanceCount_baseInstance :: (self: *MTLIndirectRenderCommand, primitiveType: MTLPrimitiveType, vertexStart: NSUInteger, vertexCount: NSUInteger, instanceCount: NSUInteger, baseInstance: NSUInteger) -> () {`
- `drawIndexedPrimitives_indexCount_indexType_indexBuffer_indexBufferOffset_instanceCount_baseVertex_baseInstance :: (self: *MTLIndirectRenderCommand, primitiveType: MTLPrimitiveType, indexCount: NSUInteger, indexType: MTLIndexType, indexBuffer: id, indexBufferOffset: NSUInteger, instanceCount: NSUInteger, baseVertex: NSInteger, baseInstance: NSUInteger) -> () {`
- `reset :: (self: *MTLIndirectRenderCommand) -> () {`

#### `MTLIndirectCommandBufferDescriptor` (struct)

- `nsobject: NSObject`
- `commandTypes :: (self: *MTLIndirectCommandBufferDescriptor) -> MTLIndirectCommandType {`
- `setCommandTypes :: (self: *MTLIndirectCommandBufferDescriptor, commandTypes: MTLIndirectCommandType) -> () {`
- `inheritPipelineState :: (self: *MTLIndirectCommandBufferDescriptor) -> BOOL {`
- `setInheritPipelineState :: (self: *MTLIndirectCommandBufferDescriptor, inheritPipelineState: BOOL) -> () {`
- `inheritBuffers :: (self: *MTLIndirectCommandBufferDescriptor) -> BOOL {`
- `setInheritBuffers :: (self: *MTLIndirectCommandBufferDescriptor, inheritBuffers: BOOL) -> () {`
- `maxVertexBufferBindCount :: (self: *MTLIndirectCommandBufferDescriptor) -> NSUInteger {`
- `setMaxVertexBufferBindCount :: (self: *MTLIndirectCommandBufferDescriptor, maxVertexBufferBindCount: NSUInteger) -> () {`
- `maxFragmentBufferBindCount :: (self: *MTLIndirectCommandBufferDescriptor) -> NSUInteger {`
- `setMaxFragmentBufferBindCount :: (self: *MTLIndirectCommandBufferDescriptor, maxFragmentBufferBindCount: NSUInteger) -> () {`

#### `MTLIndirectCommandBuffer` (struct)

- `resetWithRange :: (self: *MTLIndirectCommandBuffer, range: NSRange) -> () {`
- `indirectRenderCommandAtIndex :: (self: *MTLIndirectCommandBuffer, commandIndex: NSUInteger) -> id {`
- `size :: (self: *MTLIndirectCommandBuffer) -> NSUInteger {`

#### `MTLEvent` (struct)

- `device :: (self: *MTLEvent) -> id {`
- `label :: (self: *MTLEvent) -> *NSString {`
- `setLabel :: (self: *MTLEvent, label: *NSString) -> () {`

#### `MTLSharedEventListener` (struct)

- `nsobject: NSObject`
- `init :: (self: *MTLSharedEventListener) -> *MTLSharedEventListener {`
- `initWithDispatchQueue :: (self: *MTLSharedEventListener, dispatchQueue: dispatch_queue_t) -> *MTLSharedEventListener {`
- `dispatchQueue :: (self: *MTLSharedEventListener) -> dispatch_queue_t {`

#### `MTLSharedEvent` (struct)

- `newSharedEventHandle :: (self: *MTLSharedEvent) -> *MTLSharedEventHandle {`
- `signaledValue :: (self: *MTLSharedEvent) -> uint64_t {`
- `setSignaledValue :: (self: *MTLSharedEvent, signaledValue: uint64_t) -> () {`

#### `MTLSharedEventHandle` (struct)

- `nsobject: NSObject`
- `_priv: *MTLSharedEventHandlePrivate`
- `label :: (self: *MTLSharedEventHandle) -> *NSString {`

## Behavioral contract and constraints

This package exposes Objective-C Metal framework calls and native object types. Callers must create and release/use Objective-C objects according to Metal/ARC ownership rules; this binding does not provide a uniform ownership wrapper. The module warns that some `objc_msgSend` call forms may be incorrect for floating-point or structure-returning methods, so affected signatures are not reliable. macOS/iOS availability follows the framework.

The public surface is a direct native binding: declared records, constants, and function signatures preserve the represented external ABI; this wrapper does not itself define a different ownership or error model. Native status/error values and handle lifetimes follow the external API. ABI compatibility across external-library versions is not specified unless a version note appears above.

External libraries/frameworks named in source: no explicit `#library` name identified.

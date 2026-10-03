# nvtt


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- [OS == .WINDOWS] `NVTT_VERSION :: 20100`
- [OS == .WINDOWS] `Format :: enum s32`
- [OS == .WINDOWS] `PixelType :: enum s32`
- [OS == .WINDOWS] `Quality :: enum s32`
- [OS == .WINDOWS] `Decoder :: enum s32`
- [OS == .WINDOWS] `CompressionOptions :: struct`
- [OS == .WINDOWS] `WrapMode :: enum s32`
- [OS == .WINDOWS] `TextureType :: enum s32`
- [OS == .WINDOWS] `InputFormat :: enum s32`
- [OS == .WINDOWS] `MipmapFilter :: enum s32`
- [OS == .WINDOWS] `ResizeFilter :: enum s32`
- [OS == .WINDOWS] `RoundMode :: enum s32`
- [OS == .WINDOWS] `AlphaMode :: enum s32`
- [OS == .WINDOWS] `ShapeRestriction :: enum s32`
- [OS == .WINDOWS] `InputOptions :: struct`
- [OS == .WINDOWS] `OutputHandler :: struct`
- [OS == .WINDOWS] `OutputHandler_VTable :: struct #type_info_none`
- [OS == .WINDOWS] `OutputHandler_Destructor :: inline (this: *OutputHandler, flags: Msvc_Destructor_Flags = .NONE)`
- [OS == .WINDOWS] `OutputHandler_beginImage :: inline (this: *OutputHandler, size: s32, width: s32, height: s32, depth: s32, face: s32, miplevel: s32)`
- [OS == .WINDOWS] `OutputHandler_writeData :: inline (this: *OutputHandler, data: *void, size: s32) -> bool`
- [OS == .WINDOWS] `OutputHandler_endImage :: inline (this: *OutputHandler)`
- [OS == .WINDOWS] `vtable :: (obj: *OutputHandler) -> *OutputHandler_VTable`
- [OS == .WINDOWS] `Error :: enum s32`
- [OS == .WINDOWS] `ErrorHandler :: struct`
- [OS == .WINDOWS] `ErrorHandler_VTable :: struct #type_info_none`
- [OS == .WINDOWS] `ErrorHandler_Destructor :: inline (this: *ErrorHandler, flags: Msvc_Destructor_Flags = .NONE)`
- [OS == .WINDOWS] `ErrorHandler_error :: inline (this: *ErrorHandler, e: Error)`
- [OS == .WINDOWS] `vtable :: (obj: *ErrorHandler) -> *ErrorHandler_VTable`
- [OS == .WINDOWS] `Container :: enum s32`
- [OS == .WINDOWS] `OutputOptions :: struct`
- [OS == .WINDOWS] `Task :: #type (_context: *void, id: s32) -> void #c_call`
- [OS == .WINDOWS] `TaskDispatcher :: struct`
- [OS == .WINDOWS] `TaskDispatcher_VTable :: struct #type_info_none`
- [OS == .WINDOWS] `TaskDispatcher_Destructor :: inline (this: *TaskDispatcher, flags: Msvc_Destructor_Flags = .NONE)`
- [OS == .WINDOWS] `TaskDispatcher_dispatch :: inline (this: *TaskDispatcher, task: Task, _context: *void, count: s32)`
- [OS == .WINDOWS] `vtable :: (obj: *TaskDispatcher) -> *TaskDispatcher_VTable`
- [OS == .WINDOWS] `Compressor :: struct #type_info_procedures_are_void_pointers`
- [OS == .WINDOWS] `Context :: Compressor`
- [OS == .WINDOWS] `NormalTransform :: enum s32`
- [OS == .WINDOWS] `ToneMapper :: enum s32`
- [OS == .WINDOWS] `WarpFunction :: #type (x: *float, y: *float, d: *float) -> void #c_call`
- [OS == .WINDOWS] `Surface :: struct #type_info_none`
- [OS == .WINDOWS] `operator_assign :: (this: *Surface, img: *Surface) -> void #cpp_method #foreign libnvtt "??4Surface@nvtt@@QEAAXAEBU01@@Z"`
- [OS == .WINDOWS] `operator_assign :: (this: *Surface, img: Surface) #no_context`
- [OS == .WINDOWS] `CubeLayout :: enum s32`
- [OS == .WINDOWS] `EdgeFixup :: enum s32`
- [OS == .WINDOWS] `CubeSurface :: struct #type_info_procedures_are_void_pointers`
- [OS == .WINDOWS] `operator_assign :: (this: *CubeSurface, img: *CubeSurface) -> void #cpp_method #foreign libnvtt "??4CubeSurface@nvtt@@QEAAXAEBU01@@Z"`
- [OS == .WINDOWS] `operator_assign :: (this: *CubeSurface, img: CubeSurface) #no_context`
- [OS == .WINDOWS] `errorString :: (e: Error) -> *u8 #foreign libnvtt "?errorString@nvtt@@YAPEBDW4Error@1@@Z"`
- [OS == .WINDOWS] `version :: () -> u32 #foreign libnvtt "?version@nvtt@@YAIXZ"`
- [OS == .WINDOWS] `rmsError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "?rmsError@nvtt@@YAMAEBUSurface@1@0@Z"`
- [OS == .WINDOWS] `rmsError :: (reference: Surface, img: Surface) -> float #no_context`
- [OS == .WINDOWS] `rmsAlphaError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "?rmsAlphaError@nvtt@@YAMAEBUSurface@1@0@Z"`
- [OS == .WINDOWS] `rmsAlphaError :: (reference: Surface, img: Surface) -> float #no_context`
- [OS == .WINDOWS] `cieLabError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "?cieLabError@nvtt@@YAMAEBUSurface@1@0@Z"`
- [OS == .WINDOWS] `cieLabError :: (reference: Surface, img: Surface) -> float #no_context`
- [OS == .WINDOWS] `angularError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "?angularError@nvtt@@YAMAEBUSurface@1@0@Z"`
- [OS == .WINDOWS] `angularError :: (reference: Surface, img: Surface) -> float #no_context`
- [OS == .WINDOWS] `diff :: (reference: *Surface, img: *Surface, scale: float) -> Surface #cpp_return_type_is_non_pod #foreign libnvtt "?diff@nvtt@@YA?AUSurface@1@AEBU21@0M@Z"`
- [OS == .WINDOWS] `diff :: (reference: Surface, img: Surface, scale: float) -> Surface #no_context`
- [OS == .WINDOWS] `rmsToneMappedError :: (reference: *Surface, img: *Surface, exposure: float) -> float #foreign libnvtt "?rmsToneMappedError@nvtt@@YAMAEBUSurface@1@0M@Z"`
- [OS == .WINDOWS] `rmsToneMappedError :: (reference: Surface, img: Surface, exposure: float) -> float #no_context`
- [OS == .WINDOWS] `histogram :: (img: *Surface, width: s32, height: s32) -> Surface #cpp_return_type_is_non_pod #foreign libnvtt "?histogram@nvtt@@YA?AUSurface@1@AEBU21@HH@Z"`
- [OS == .WINDOWS] `histogram :: (img: Surface, width: s32, height: s32) -> Surface #no_context`
- [OS == .WINDOWS] `histogram :: (img: *Surface, minRange: float, maxRange: float, width: s32, height: s32) -> Surface #cpp_return_type_is_non_pod #foreign libnvtt "?histogram@nvtt@@YA?AUSurface@1@AEBU21@MMHH@Z"`
- [OS == .WINDOWS] `histogram :: (img: Surface, minRange: float, maxRange: float, width: s32, height: s32) -> Surface #no_context`
- [OS == .LINUX] `NVTT_VERSION :: 20100`
- [OS == .LINUX] `Format :: enum u32`
- [OS == .LINUX] `PixelType :: enum u32`
- [OS == .LINUX] `Quality :: enum u32`
- [OS == .LINUX] `Decoder :: enum u32`
- [OS == .LINUX] `CompressionOptions :: struct`
- [OS == .LINUX] `WrapMode :: enum u32`
- [OS == .LINUX] `TextureType :: enum u32`
- [OS == .LINUX] `InputFormat :: enum u32`
- [OS == .LINUX] `MipmapFilter :: enum u32`
- [OS == .LINUX] `ResizeFilter :: enum u32`
- [OS == .LINUX] `RoundMode :: enum u32`
- [OS == .LINUX] `AlphaMode :: enum u32`
- [OS == .LINUX] `ShapeRestriction :: enum u32`
- [OS == .LINUX] `InputOptions :: struct`
- [OS == .LINUX] `OutputHandler :: struct`
- [OS == .LINUX] `OutputHandler_VTable :: struct #type_info_none`
- [OS == .LINUX] `OutputHandler_Destructor :: inline (this: *OutputHandler)`
- [OS == .LINUX] `OutputHandler_Destructor_Deleting :: inline (this: *OutputHandler)`
- [OS == .LINUX] `OutputHandler_beginImage :: inline (this: *OutputHandler, size: s32, width: s32, height: s32, depth: s32, face: s32, miplevel: s32)`
- [OS == .LINUX] `OutputHandler_writeData :: inline (this: *OutputHandler, data: *void, size: s32) -> bool`
- [OS == .LINUX] `OutputHandler_endImage :: inline (this: *OutputHandler)`
- [OS == .LINUX] `vtable :: (obj: *OutputHandler) -> *OutputHandler_VTable`
- [OS == .LINUX] `Error :: enum u32`
- [OS == .LINUX] `ErrorHandler :: struct`
- [OS == .LINUX] `ErrorHandler_VTable :: struct #type_info_none`
- [OS == .LINUX] `ErrorHandler_Destructor :: inline (this: *ErrorHandler)`
- [OS == .LINUX] `ErrorHandler_Destructor_Deleting :: inline (this: *ErrorHandler)`
- [OS == .LINUX] `ErrorHandler_error :: inline (this: *ErrorHandler, e: Error)`
- [OS == .LINUX] `vtable :: (obj: *ErrorHandler) -> *ErrorHandler_VTable`
- [OS == .LINUX] `Container :: enum u32`
- [OS == .LINUX] `OutputOptions :: struct`
- [OS == .LINUX] `Task :: #type (_context: *void, id: s32) -> void #c_call`
- [OS == .LINUX] `TaskDispatcher :: struct`
- [OS == .LINUX] `TaskDispatcher_VTable :: struct #type_info_none`
- [OS == .LINUX] `TaskDispatcher_Destructor :: inline (this: *TaskDispatcher)`
- [OS == .LINUX] `TaskDispatcher_Destructor_Deleting :: inline (this: *TaskDispatcher)`
- [OS == .LINUX] `TaskDispatcher_dispatch :: inline (this: *TaskDispatcher, task: Task, _context: *void, count: s32)`
- [OS == .LINUX] `vtable :: (obj: *TaskDispatcher) -> *TaskDispatcher_VTable`
- [OS == .LINUX] `Compressor :: struct #type_info_procedures_are_void_pointers`
- [OS == .LINUX] `Context :: Compressor`
- [OS == .LINUX] `NormalTransform :: enum u32`
- [OS == .LINUX] `ToneMapper :: enum u32`
- [OS == .LINUX] `WarpFunction :: #type (x: *float, y: *float, d: *float) -> void #c_call`
- [OS == .LINUX] `Surface :: struct #type_info_none`
- [OS == .LINUX] `operator_assign :: (this: *Surface, img: *Surface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7SurfaceaSERKS0_"`
- [OS == .LINUX] `operator_assign :: (this: *Surface, img: Surface) #no_context`
- [OS == .LINUX] `CubeLayout :: enum u32`
- [OS == .LINUX] `EdgeFixup :: enum u32`
- [OS == .LINUX] `CubeSurface :: struct #type_info_procedures_are_void_pointers`
- [OS == .LINUX] `operator_assign :: (this: *CubeSurface, img: *CubeSurface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt11CubeSurfaceaSERKS0_"`
- [OS == .LINUX] `operator_assign :: (this: *CubeSurface, img: CubeSurface) #no_context`
- [OS == .LINUX] `errorString :: (e: Error) -> *u8 #foreign libnvtt "_ZN4nvtt11errorStringENS_5ErrorE"`
- [OS == .LINUX] `version :: () -> u32 #foreign libnvtt "_ZN4nvtt7versionEv"`
- [OS == .LINUX] `rmsError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "_ZN4nvtt8rmsErrorERKNS_7SurfaceES2_"`
- [OS == .LINUX] `rmsError :: (reference: Surface, img: Surface) -> float #no_context`
- [OS == .LINUX] `rmsAlphaError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "_ZN4nvtt13rmsAlphaErrorERKNS_7SurfaceES2_"`
- [OS == .LINUX] `rmsAlphaError :: (reference: Surface, img: Surface) -> float #no_context`
- [OS == .LINUX] `cieLabError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "_ZN4nvtt11cieLabErrorERKNS_7SurfaceES2_"`
- [OS == .LINUX] `cieLabError :: (reference: Surface, img: Surface) -> float #no_context`
- [OS == .LINUX] `angularError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "_ZN4nvtt12angularErrorERKNS_7SurfaceES2_"`
- [OS == .LINUX] `angularError :: (reference: Surface, img: Surface) -> float #no_context`
- [OS == .LINUX] `diff :: (reference: *Surface, img: *Surface, scale: float) -> Surface #cpp_return_type_is_non_pod #foreign libnvtt "_ZN4nvtt4diffERKNS_7SurfaceES2_f"`
- [OS == .LINUX] `diff :: (reference: Surface, img: Surface, scale: float) -> Surface #no_context`
- [OS == .LINUX] `rmsToneMappedError :: (reference: *Surface, img: *Surface, exposure: float) -> float #foreign libnvtt "_ZN4nvtt18rmsToneMappedErrorERKNS_7SurfaceES2_f"`
- [OS == .LINUX] `rmsToneMappedError :: (reference: Surface, img: Surface, exposure: float) -> float #no_context`
- [OS == .LINUX] `histogram :: (img: *Surface, width: s32, height: s32) -> Surface #cpp_return_type_is_non_pod #foreign libnvtt "_ZN4nvtt9histogramERKNS_7SurfaceEii"`
- [OS == .LINUX] `histogram :: (img: Surface, width: s32, height: s32) -> Surface #no_context`
- [OS == .LINUX] `histogram :: (img: *Surface, minRange: float, maxRange: float, width: s32, height: s32) -> Surface #cpp_return_type_is_non_pod #foreign libnvtt "_ZN4nvtt9histogramERKNS_7SurfaceEffii"`
- [OS == .LINUX] `histogram :: (img: Surface, minRange: float, maxRange: float, width: s32, height: s32) -> Surface #no_context`
- [OS == .MACOS] `NVTT_VERSION :: 20100`
- [OS == .MACOS] `Format :: enum u32`
- [OS == .MACOS] `PixelType :: enum u32`
- [OS == .MACOS] `Quality :: enum u32`
- [OS == .MACOS] `Decoder :: enum u32`
- [OS == .MACOS] `CompressionOptions :: struct`
- [OS == .MACOS] `WrapMode :: enum u32`
- [OS == .MACOS] `TextureType :: enum u32`
- [OS == .MACOS] `InputFormat :: enum u32`
- [OS == .MACOS] `MipmapFilter :: enum u32`
- [OS == .MACOS] `ResizeFilter :: enum u32`
- [OS == .MACOS] `RoundMode :: enum u32`
- [OS == .MACOS] `AlphaMode :: enum u32`
- [OS == .MACOS] `ShapeRestriction :: enum u32`
- [OS == .MACOS] `InputOptions :: struct`
- [OS == .MACOS] `OutputHandler :: struct`
- [OS == .MACOS] `OutputHandler_VTable :: struct #type_info_none`
- [OS == .MACOS] `OutputHandler_Destructor :: inline (this: *OutputHandler)`
- [OS == .MACOS] `OutputHandler_Destructor_Deleting :: inline (this: *OutputHandler)`
- [OS == .MACOS] `OutputHandler_beginImage :: inline (this: *OutputHandler, size: s32, width: s32, height: s32, depth: s32, face: s32, miplevel: s32)`
- [OS == .MACOS] `OutputHandler_writeData :: inline (this: *OutputHandler, data: *void, size: s32) -> bool`
- [OS == .MACOS] `OutputHandler_endImage :: inline (this: *OutputHandler)`
- [OS == .MACOS] `vtable :: (obj: *OutputHandler) -> *OutputHandler_VTable`
- [OS == .MACOS] `Error :: enum u32`
- [OS == .MACOS] `ErrorHandler :: struct`
- [OS == .MACOS] `ErrorHandler_VTable :: struct #type_info_none`
- [OS == .MACOS] `ErrorHandler_Destructor :: inline (this: *ErrorHandler)`
- [OS == .MACOS] `ErrorHandler_Destructor_Deleting :: inline (this: *ErrorHandler)`
- [OS == .MACOS] `ErrorHandler_error :: inline (this: *ErrorHandler, e: Error)`
- [OS == .MACOS] `vtable :: (obj: *ErrorHandler) -> *ErrorHandler_VTable`
- [OS == .MACOS] `Container :: enum u32`
- [OS == .MACOS] `OutputOptions :: struct`
- [OS == .MACOS] `Task :: #type (_context: *void, id: s32) -> void #c_call`
- [OS == .MACOS] `TaskDispatcher :: struct`
- [OS == .MACOS] `TaskDispatcher_VTable :: struct #type_info_none`
- [OS == .MACOS] `TaskDispatcher_Destructor :: inline (this: *TaskDispatcher)`
- [OS == .MACOS] `TaskDispatcher_Destructor_Deleting :: inline (this: *TaskDispatcher)`
- [OS == .MACOS] `TaskDispatcher_dispatch :: inline (this: *TaskDispatcher, task: Task, _context: *void, count: s32)`
- [OS == .MACOS] `vtable :: (obj: *TaskDispatcher) -> *TaskDispatcher_VTable`
- [OS == .MACOS] `Compressor :: struct #type_info_procedures_are_void_pointers`
- [OS == .MACOS] `Context :: Compressor`
- [OS == .MACOS] `NormalTransform :: enum u32`
- [OS == .MACOS] `ToneMapper :: enum u32`
- [OS == .MACOS] `WarpFunction :: #type (x: *float, y: *float, d: *float) -> void #c_call`
- [OS == .MACOS] `Surface :: struct #type_info_none`
- [OS == .MACOS] `operator_assign :: (this: *Surface, img: *Surface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7SurfaceaSERKS0_"`
- [OS == .MACOS] `operator_assign :: (this: *Surface, img: Surface) #no_context`
- [OS == .MACOS] `CubeLayout :: enum u32`
- [OS == .MACOS] `EdgeFixup :: enum u32`
- [OS == .MACOS] `CubeSurface :: struct #type_info_procedures_are_void_pointers`
- [OS == .MACOS] `operator_assign :: (this: *CubeSurface, img: *CubeSurface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt11CubeSurfaceaSERKS0_"`
- [OS == .MACOS] `operator_assign :: (this: *CubeSurface, img: CubeSurface) #no_context`
- [OS == .MACOS] `errorString :: (e: Error) -> *u8 #foreign libnvtt "_ZN4nvtt11errorStringENS_5ErrorE"`
- [OS == .MACOS] `version :: () -> u32 #foreign libnvtt "_ZN4nvtt7versionEv"`
- [OS == .MACOS] `rmsError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "_ZN4nvtt8rmsErrorERKNS_7SurfaceES2_"`
- [OS == .MACOS] `rmsError :: (reference: Surface, img: Surface) -> float #no_context`
- [OS == .MACOS] `rmsAlphaError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "_ZN4nvtt13rmsAlphaErrorERKNS_7SurfaceES2_"`
- [OS == .MACOS] `rmsAlphaError :: (reference: Surface, img: Surface) -> float #no_context`
- [OS == .MACOS] `cieLabError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "_ZN4nvtt11cieLabErrorERKNS_7SurfaceES2_"`
- [OS == .MACOS] `cieLabError :: (reference: Surface, img: Surface) -> float #no_context`
- [OS == .MACOS] `angularError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "_ZN4nvtt12angularErrorERKNS_7SurfaceES2_"`
- [OS == .MACOS] `angularError :: (reference: Surface, img: Surface) -> float #no_context`
- [OS == .MACOS] `diff :: (reference: *Surface, img: *Surface, scale: float) -> Surface #cpp_return_type_is_non_pod #foreign libnvtt "_ZN4nvtt4diffERKNS_7SurfaceES2_f"`
- [OS == .MACOS] `diff :: (reference: Surface, img: Surface, scale: float) -> Surface #no_context`
- [OS == .MACOS] `rmsToneMappedError :: (reference: *Surface, img: *Surface, exposure: float) -> float #foreign libnvtt "_ZN4nvtt18rmsToneMappedErrorERKNS_7SurfaceES2_f"`
- [OS == .MACOS] `rmsToneMappedError :: (reference: Surface, img: Surface, exposure: float) -> float #no_context`
- [OS == .MACOS] `histogram :: (img: *Surface, width: s32, height: s32) -> Surface #cpp_return_type_is_non_pod #foreign libnvtt "_ZN4nvtt9histogramERKNS_7SurfaceEii"`
- [OS == .MACOS] `histogram :: (img: Surface, width: s32, height: s32) -> Surface #no_context`
- [OS == .MACOS] `histogram :: (img: *Surface, minRange: float, maxRange: float, width: s32, height: s32) -> Surface #cpp_return_type_is_non_pod #foreign libnvtt "_ZN4nvtt9histogramERKNS_7SurfaceEffii"`
- [OS == .MACOS] `histogram :: (img: Surface, minRange: float, maxRange: float, width: s32, height: s32) -> Surface #no_context`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `AlphaMode`

Variant 1 (Windows):

```jai
AlphaMode :: enum s32 {
    None          :: 0;
    Transparency  :: 1;
    Premultiplied :: 2;
}
```
Variant 2 (Linux):

```jai
AlphaMode :: enum u32 {
    None          :: 0;
    Transparency  :: 1;
    Premultiplied :: 2;
}
```
### `CompressionOptions`

Variant 1 (Windows):

```jai
CompressionOptions :: struct {
    Private :: struct {}
    m: *Private;

    Constructor :: (this: *CompressionOptions) -> void #cpp_method #foreign libnvtt "??0CompressionOptions@nvtt@@QEAA@XZ";
    Destructor :: (this: *CompressionOptions) -> void #cpp_method #foreign libnvtt "??1CompressionOptions@nvtt@@QEAA@XZ";

    reset :: (this: *CompressionOptions) -> void #cpp_method #foreign libnvtt "?reset@CompressionOptions@nvtt@@QEAAXXZ";

    setFormat :: (this: *CompressionOptions, format: Format) -> void #cpp_method #foreign libnvtt "?setFormat@CompressionOptions@nvtt@@QEAAXW4Format@2@@Z";
    setQuality :: (this: *CompressionOptions, quality: Quality) -> void #cpp_method #foreign libnvtt "?setQuality@CompressionOptions@nvtt@@QEAAXW4Quality@2@@Z";
    setColorWeights :: (this: *CompressionOptions, red: float, green: float, blue: float, alpha: float = 1.0) -> void #cpp_method #foreign libnvtt "?setColorWeights@CompressionOptions@nvtt@@QEAAXMMMM@Z";
    setRGBMThreshold :: (this: *CompressionOptions, min_m: float) -> void #cpp_method #foreign libnvtt "?setRGBMThreshold@CompressionOptions@nvtt@@QEAAXM@Z";

    setExternalCompressor :: (this: *CompressionOptions, name: *u8) -> void #cpp_method #foreign libnvtt "?setExternalCompressor@CompressionOptions@nvtt@@QEAAXPEBD@Z";

    setPixelFormat :: (this: *CompressionOptions, bitcount: u32, rmask: u32, gmask: u32, bmask: u32, amask: u32) -> void #cpp_method #foreign libnvtt "?setPixelFormat@CompressionOptions@nvtt@@QEAAXIIIII@Z";
    setPixelFormat :: (this: *CompressionOptions, rsize: u8, gsize: u8, bsize: u8, asize: u8) -> void #cpp_method #foreign libnvtt "?setPixelFormat@CompressionOptions@nvtt@@QEAAXEEEE@Z";

    setPixelType :: (this: *CompressionOptions, pixelType: PixelType) -> void #cpp_method #foreign libnvtt "?setPixelType@CompressionOptions@nvtt@@QEAAXW4PixelType@2@@Z";

    setPitchAlignment :: (this: *CompressionOptions, pitchAlignment: s32) -> void #cpp_method #foreign libnvtt "?setPitchAlignment@CompressionOptions@nvtt@@QEAAXH@Z";

    setQuantization :: (this: *CompressionOptions, colorDithering: bool, alphaDithering: bool, binaryAlpha: bool, alphaThreshold: s32 = 127) -> void #cpp_method #foreign libnvtt "?setQuantization@CompressionOptions@nvtt@@QEAAX_N00H@Z";

    setTargetDecoder :: (this: *CompressionOptions, decoder: Decoder) -> void #cpp_method #foreign libnvtt "?setTargetDecoder@CompressionOptions@nvtt@@QEAAXW4Decoder@2@@Z";

    format :: (this: *CompressionOptions) -> Format #cpp_method #foreign libnvtt "?format@CompressionOptions@nvtt@@QEBA?AW4Format@2@XZ";
    d3d9Format :: (this: *CompressionOptions) -> u32 #cpp_method #foreign libnvtt "?d3d9Format@CompressionOptions@nvtt@@QEBAIXZ";
    dxgiFormat :: (this: *CompressionOptions) -> u32 #cpp_method #foreign libnvtt "?dxgiFormat@CompressionOptions@nvtt@@QEBAIXZ";
}
```
Variant 2 (Linux):

```jai
CompressionOptions :: struct {
    Private :: struct {}
    m: *Private;

    Constructor :: (this: *CompressionOptions) -> void #cpp_method #foreign libnvtt "_ZN4nvtt18CompressionOptionsC1Ev";
    Destructor_Base :: (this: *CompressionOptions) -> void #cpp_method #foreign libnvtt "_ZN4nvtt18CompressionOptionsD2Ev";
    Destructor :: (this: *CompressionOptions) -> void #cpp_method #foreign libnvtt "_ZN4nvtt18CompressionOptionsD1Ev";

    reset :: (this: *CompressionOptions) -> void #cpp_method #foreign libnvtt "_ZN4nvtt18CompressionOptions5resetEv";

    setFormat :: (this: *CompressionOptions, format: Format) -> void #cpp_method #foreign libnvtt "_ZN4nvtt18CompressionOptions9setFormatENS_6FormatE";
    setQuality :: (this: *CompressionOptions, quality: Quality) -> void #cpp_method #foreign libnvtt "_ZN4nvtt18CompressionOptions10setQualityENS_7QualityE";
    setColorWeights :: (this: *CompressionOptions, red: float, green: float, blue: float, alpha: float = 1.0) -> void #cpp_method #foreign libnvtt "_ZN4nvtt18CompressionOptions15setColorWeightsEffff";
    setRGBMThreshold :: (this: *CompressionOptions, min_m: float) -> void #cpp_method #foreign libnvtt "_ZN4nvtt18CompressionOptions16setRGBMThresholdEf";

    setExternalCompressor :: (this: *CompressionOptions, name: *u8) -> void #cpp_method #foreign libnvtt "_ZN4nvtt18CompressionOptions21setExternalCompressorEPKc";

    setPixelFormat :: (this: *CompressionOptions, bitcount: u32, rmask: u32, gmask: u32, bmask: u32, amask: u32) -> void #cpp_method #foreign libnvtt "_ZN4nvtt18CompressionOptions14setPixelFormatEjjjjj";
    setPixelFormat :: (this: *CompressionOptions, rsize: u8, gsize: u8, bsize: u8, asize: u8) -> void #cpp_method #foreign libnvtt "_ZN4nvtt18CompressionOptions14setPixelFormatEhhhh";

    setPixelType :: (this: *CompressionOptions, pixelType: PixelType) -> void #cpp_method #foreign libnvtt "_ZN4nvtt18CompressionOptions12setPixelTypeENS_9PixelTypeE";

    setPitchAlignment :: (this: *CompressionOptions, pitchAlignment: s32) -> void #cpp_method #foreign libnvtt "_ZN4nvtt18CompressionOptions17setPitchAlignmentEi";

    setQuantization :: (this: *CompressionOptions, colorDithering: bool, alphaDithering: bool, binaryAlpha: bool, alphaThreshold: s32 = 127) -> void #cpp_method #foreign libnvtt "_ZN4nvtt18CompressionOptions15setQuantizationEbbbi";

    setTargetDecoder :: (this: *CompressionOptions, decoder: Decoder) -> void #cpp_method #foreign libnvtt "_ZN4nvtt18CompressionOptions16setTargetDecoderENS_7DecoderE";

    format :: (this: *CompressionOptions) -> Format #cpp_method #foreign libnvtt "_ZNK4nvtt18CompressionOptions6formatEv";
    d3d9Format :: (this: *CompressionOptions) -> u32 #cpp_method #foreign libnvtt "_ZNK4nvtt18CompressionOptions10d3d9FormatEv";
    dxgiFormat :: (this: *CompressionOptions) -> u32 #cpp_method #foreign libnvtt "_ZNK4nvtt18CompressionOptions10dxgiFormatEv";
}
```
### `Compressor`

Variant 1 (Windows):

```jai
Compressor :: struct #type_info_procedures_are_void_pointers {
    Private :: struct {}
    m: *Private;

Constructor :: (this: *Compressor) -> void #cpp_method #foreign libnvtt "??0Compressor@nvtt@@QEAA@XZ"; Destructor :: (this: *Compressor) -> void #cpp_method #foreign libnvtt "??1Compressor@nvtt@@QEAA@XZ";
estimateSize :: (this: *Compressor, inputOptions: *InputOptions, compressionOptions: *CompressionOptions) -> s32 #cpp_method #foreign libnvtt "?estimateSize@Compressor@nvtt@@QEBAHAEBUInputOptions@2@AEBUCompressionOptions@2@@Z"; estimateSize :: (this: *Compressor, inputOptions: InputOptions, compressionOptions: CompressionOptions) -> s32 #no_context;

outputHeader :: (this: *Compressor, img: *Surface, mipmapCount: s32, compressionOptions: *CompressionOptions, outputOptions: *OutputOptions) -> bool #cpp_method #foreign libnvtt "?outputHeader@Compressor@nvtt@@QEBA_NAEBUSurface@2@HAEBUCompressionOptions@2@AEBUOutputOptions@2@@Z"; outputHeader :: (this: *Compressor, img: Surface, mipmapCount: s32, compressionOptions: CompressionOptions, outputOptions: OutputOptions) -> bool #no_context;
compress :: (this: *Compressor, img: *Surface, face: s32, mipmap: s32, compressionOptions: *CompressionOptions, outputOptions: *OutputOptions) -> bool #cpp_method #foreign libnvtt "?compress@Compressor@nvtt@@QEBA_NAEBUSurface@2@HHAEBUCompressionOptions@2@AEBUOutputOptions@2@@Z"; compress :: (this: *Compressor, img: Surface, face: s32, mipmap: s32, compressionOptions: CompressionOptions, outputOptions: OutputOptions) -> bool #no_context;
estimateSize :: (this: *Compressor, img: *Surface, mipmapCount: s32, compressionOptions: *CompressionOptions) -> s32 #cpp_method #foreign libnvtt "?estimateSize@Compressor@nvtt@@QEBAHAEBUSurface@2@HAEBUCompressionOptions@2@@Z"; estimateSize :: (this: *Compressor, img: Surface, mipmapCount: s32, compressionOptions: CompressionOptions) -> s32 #no_context;

outputHeader :: (this: *Compressor, cube: *CubeSurface, mipmapCount: s32, compressionOptions: *CompressionOptions, outputOptions: *OutputOptions) -> bool #cpp_method #foreign libnvtt "?outputHeader@Compressor@nvtt@@QEBA_NAEBUCubeSurface@2@HAEBUCompressionOptions@2@AEBUOutputOptions@2@@Z"; outputHeader :: (this: *Compressor, cube: CubeSurface, mipmapCount: s32, compressionOptions: CompressionOptions, outputOptions: OutputOptions) -> bool #no_context;
compress :: (this: *Compressor, cube: *CubeSurface, mipmap: s32, compressionOptions: *CompressionOptions, outputOptions: *OutputOptions) -> bool #cpp_method #foreign libnvtt "?compress@Compressor@nvtt@@QEBA_NAEBUCubeSurface@2@HAEBUCompressionOptions@2@AEBUOutputOptions@2@@Z"; compress :: (this: *Compressor, cube: CubeSurface, mipmap: s32, compressionOptions: CompressionOptions, outputOptions: OutputOptions) -> bool #no_context;
estimateSize :: (this: *Compressor, cube: *CubeSurface, mipmapCount: s32, compressionOptions: *CompressionOptions) -> s32 #cpp_method #foreign libnvtt "?estimateSize@Compressor@nvtt@@QEBAHAEBUCubeSurface@2@HAEBUCompressionOptions@2@@Z"; estimateSize :: (this: *Compressor, cube: CubeSurface, mipmapCount: s32, compressionOptions: CompressionOptions) -> s32 #no_context;

outputHeader :: (this: *Compressor, type: TextureType, w: s32, h: s32, d: s32, arraySize: s32, mipmapCount: s32, isNormalMap: bool, compressionOptions: *CompressionOptions, outputOptions: *OutputOptions) -> bool #cpp_method #foreign libnvtt "?outputHeader@Compressor@nvtt@@QEBA_NW4TextureType@2@HHHHH_NAEBUCompressionOptions@2@AEBUOutputOptions@2@@Z"; outputHeader :: (this: *Compressor, type: TextureType, w: s32, h: s32, d: s32, arraySize: s32, mipmapCount: s32, isNormalMap: bool, compressionOptions: CompressionOptions, outputOptions: OutputOptions) -> bool #no_context;
compress :: (this: *Compressor, w: s32, h: s32, d: s32, face: s32, mipmap: s32, rgba: *float, compressionOptions: *CompressionOptions, outputOptions: *OutputOptions) -> bool #cpp_method #foreign libnvtt "?compress@Compressor@nvtt@@QEBA_NHHHHHPEBMAEBUCompressionOptions@2@AEBUOutputOptions@2@@Z"; compress :: (this: *Compressor, w: s32, h: s32, d: s32, face: s32, mipmap: s32, rgba: *float, compressionOptions: CompressionOptions, outputOptions: OutputOptions) -> bool #no_context;
estimateSize :: (this: *Compressor, w: s32, h: s32, d: s32, mipmapCount: s32, compressionOptions: *CompressionOptions) -> s32 #cpp_method #foreign libnvtt "?estimateSize@Compressor@nvtt@@QEBAHHHHHAEBUCompressionOptions@2@@Z"; estimateSize :: (this: *Compressor, w: s32, h: s32, d: s32, mipmapCount: s32, compressionOptions: CompressionOptions) -> s32 #no_context;
}
```
Variant 2 (Linux):

```jai
Compressor :: struct #type_info_procedures_are_void_pointers {
    Private :: struct {}
    m: *Private;

Constructor :: (this: *Compressor) -> void #cpp_method #foreign libnvtt "_ZN4nvtt10CompressorC1Ev"; Destructor_Base :: (this: *Compressor) -> void #cpp_method #foreign libnvtt "_ZN4nvtt10CompressorD2Ev"; Destructor :: (this: *Compressor) -> void #cpp_method #foreign libnvtt "_ZN4nvtt10CompressorD1Ev";
estimateSize :: (this: *Compressor, inputOptions: *InputOptions, compressionOptions: *CompressionOptions) -> s32 #cpp_method #foreign libnvtt "_ZNK4nvtt10Compressor12estimateSizeERKNS_12InputOptionsERKNS_18CompressionOptionsE"; estimateSize :: (this: *Compressor, inputOptions: InputOptions, compressionOptions: CompressionOptions) -> s32 #no_context;

outputHeader :: (this: *Compressor, img: *Surface, mipmapCount: s32, compressionOptions: *CompressionOptions, outputOptions: *OutputOptions) -> bool #cpp_method #foreign libnvtt "_ZNK4nvtt10Compressor12outputHeaderERKNS_7SurfaceEiRKNS_18CompressionOptionsERKNS_13OutputOptionsE"; outputHeader :: (this: *Compressor, img: Surface, mipmapCount: s32, compressionOptions: CompressionOptions, outputOptions: OutputOptions) -> bool #no_context;
compress :: (this: *Compressor, img: *Surface, face: s32, mipmap: s32, compressionOptions: *CompressionOptions, outputOptions: *OutputOptions) -> bool #cpp_method #foreign libnvtt "_ZNK4nvtt10Compressor8compressERKNS_7SurfaceEiiRKNS_18CompressionOptionsERKNS_13OutputOptionsE"; compress :: (this: *Compressor, img: Surface, face: s32, mipmap: s32, compressionOptions: CompressionOptions, outputOptions: OutputOptions) -> bool #no_context;
estimateSize :: (this: *Compressor, img: *Surface, mipmapCount: s32, compressionOptions: *CompressionOptions) -> s32 #cpp_method #foreign libnvtt "_ZNK4nvtt10Compressor12estimateSizeERKNS_7SurfaceEiRKNS_18CompressionOptionsE"; estimateSize :: (this: *Compressor, img: Surface, mipmapCount: s32, compressionOptions: CompressionOptions) -> s32 #no_context;

outputHeader :: (this: *Compressor, cube: *CubeSurface, mipmapCount: s32, compressionOptions: *CompressionOptions, outputOptions: *OutputOptions) -> bool #cpp_method #foreign libnvtt "_ZNK4nvtt10Compressor12outputHeaderERKNS_11CubeSurfaceEiRKNS_18CompressionOptionsERKNS_13OutputOptionsE"; outputHeader :: (this: *Compressor, cube: CubeSurface, mipmapCount: s32, compressionOptions: CompressionOptions, outputOptions: OutputOptions) -> bool #no_context;
compress :: (this: *Compressor, cube: *CubeSurface, mipmap: s32, compressionOptions: *CompressionOptions, outputOptions: *OutputOptions) -> bool #cpp_method #foreign libnvtt "_ZNK4nvtt10Compressor8compressERKNS_11CubeSurfaceEiRKNS_18CompressionOptionsERKNS_13OutputOptionsE"; compress :: (this: *Compressor, cube: CubeSurface, mipmap: s32, compressionOptions: CompressionOptions, outputOptions: OutputOptions) -> bool #no_context;
estimateSize :: (this: *Compressor, cube: *CubeSurface, mipmapCount: s32, compressionOptions: *CompressionOptions) -> s32 #cpp_method #foreign libnvtt "_ZNK4nvtt10Compressor12estimateSizeERKNS_11CubeSurfaceEiRKNS_18CompressionOptionsE"; estimateSize :: (this: *Compressor, cube: CubeSurface, mipmapCount: s32, compressionOptions: CompressionOptions) -> s32 #no_context;

outputHeader :: (this: *Compressor, type: TextureType, w: s32, h: s32, d: s32, arraySize: s32, mipmapCount: s32, isNormalMap: bool, compressionOptions: *CompressionOptions, outputOptions: *OutputOptions) -> bool #cpp_method #foreign libnvtt "_ZNK4nvtt10Compressor12outputHeaderENS_11TextureTypeEiiiiibRKNS_18CompressionOptionsERKNS_13OutputOptionsE"; outputHeader :: (this: *Compressor, type: TextureType, w: s32, h: s32, d: s32, arraySize: s32, mipmapCount: s32, isNormalMap: bool, compressionOptions: CompressionOptions, outputOptions: OutputOptions) -> bool #no_context;
compress :: (this: *Compressor, w: s32, h: s32, d: s32, face: s32, mipmap: s32, rgba: *float, compressionOptions: *CompressionOptions, outputOptions: *OutputOptions) -> bool #cpp_method #foreign libnvtt "_ZNK4nvtt10Compressor8compressEiiiiiPKfRKNS_18CompressionOptionsERKNS_13OutputOptionsE"; compress :: (this: *Compressor, w: s32, h: s32, d: s32, face: s32, mipmap: s32, rgba: *float, compressionOptions: CompressionOptions, outputOptions: OutputOptions) -> bool #no_context;
estimateSize :: (this: *Compressor, w: s32, h: s32, d: s32, mipmapCount: s32, compressionOptions: *CompressionOptions) -> s32 #cpp_method #foreign libnvtt "_ZNK4nvtt10Compressor12estimateSizeEiiiiRKNS_18CompressionOptionsE"; estimateSize :: (this: *Compressor, w: s32, h: s32, d: s32, mipmapCount: s32, compressionOptions: CompressionOptions) -> s32 #no_context;
}
```
### `Container`

Variant 1 (Windows):

```jai
Container :: enum s32 {
    DDS   :: 0;
    DDS10 :: 1;
    KTX   :: 2;
}
```
Variant 2 (Linux):

```jai
Container :: enum u32 {
    DDS   :: 0;
    DDS10 :: 1;
    KTX   :: 2;
}
```
### `CubeLayout`

Variant 1 (Windows):

```jai
CubeLayout :: enum s32 {
    VerticalCross     :: 0;
    HorizontalCross   :: 1;
    Column            :: 2;
    Row               :: 3;
    LatitudeLongitude :: 4;
}
```
Variant 2 (Linux):

```jai
CubeLayout :: enum u32 {
    VerticalCross     :: 0;
    HorizontalCross   :: 1;
    Column            :: 2;
    Row               :: 3;
    LatitudeLongitude :: 4;
}
```
### `CubeSurface`

Variant 1 (Windows):

```jai
CubeSurface :: struct #type_info_procedures_are_void_pointers {
Constructor :: (this: *CubeSurface) -> void #cpp_method #foreign libnvtt "??0CubeSurface@nvtt@@QEAA@XZ"; CopyConstructor :: (this: *CubeSurface, img: *CubeSurface) -> void #cpp_method #foreign libnvtt "??0CubeSurface@nvtt@@QEAA@AEBU01@@Z"; CopyConstructor :: (this: *CubeSurface, img: CubeSurface) #no_context;
Destructor :: (this: *CubeSurface) -> void #cpp_method #foreign libnvtt "??1CubeSurface@nvtt@@QEAA@XZ";
    unfold :: (this: *CubeSurface, layout: CubeLayout) -> Surface #cpp_method #cpp_return_type_is_non_pod #foreign libnvtt "?unfold@CubeSurface@nvtt@@QEBA?AUSurface@2@W4CubeLayout@2@@Z";

    average :: (this: *CubeSurface, channel: s32) -> float #cpp_method #foreign libnvtt "?average@CubeSurface@nvtt@@QEBAMH@Z";
    range :: (this: *CubeSurface, channel: s32, minimum_ptr: *float, maximum_ptr: *float) -> void #cpp_method #foreign libnvtt "?range@CubeSurface@nvtt@@QEBAXHPEAM0@Z";
    clamp :: (this: *CubeSurface, channel: s32, low: float = 0.0, high: float = 1.0) -> void #cpp_method #foreign libnvtt "?clamp@CubeSurface@nvtt@@QEAAXHMM@Z";

    irradianceFilter :: (this: *CubeSurface, size: s32, fixupMethod: EdgeFixup) -> CubeSurface #cpp_method #cpp_return_type_is_non_pod #foreign libnvtt "?irradianceFilter@CubeSurface@nvtt@@QEBA?AU12@HW4EdgeFixup@2@@Z";
    cosinePowerFilter :: (this: *CubeSurface, size: s32, cosinePower: float, fixupMethod: EdgeFixup) -> CubeSurface #cpp_method #cpp_return_type_is_non_pod #foreign libnvtt "?cosinePowerFilter@CubeSurface@nvtt@@QEBA?AU12@HMW4EdgeFixup@2@@Z";

    fastResample :: (this: *CubeSurface, size: s32, fixupMethod: EdgeFixup) -> CubeSurface #cpp_method #cpp_return_type_is_non_pod #foreign libnvtt "?fastResample@CubeSurface@nvtt@@QEBA?AU12@HW4EdgeFixup@2@@Z";

    toLinear :: (this: *CubeSurface, gamma: float) -> void #cpp_method #foreign libnvtt "?toLinear@CubeSurface@nvtt@@QEAAXM@Z";
    toGamma :: (this: *CubeSurface, gamma: float) -> void #cpp_method #foreign libnvtt "?toGamma@CubeSurface@nvtt@@QEAAXM@Z";

    Private :: struct {}
    m: *Private;
}
```
Variant 2 (Linux):

```jai
CubeSurface :: struct #type_info_procedures_are_void_pointers {
Constructor :: (this: *CubeSurface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt11CubeSurfaceC1Ev"; CopyConstructor :: (this: *CubeSurface, img: *CubeSurface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt11CubeSurfaceC1ERKS0_"; CopyConstructor :: (this: *CubeSurface, img: CubeSurface) #no_context;
Destructor_Base :: (this: *CubeSurface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt11CubeSurfaceD2Ev"; Destructor :: (this: *CubeSurface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt11CubeSurfaceD1Ev";
    unfold :: (this: *CubeSurface, layout: CubeLayout) -> Surface #cpp_method #cpp_return_type_is_non_pod #foreign libnvtt "_ZNK4nvtt11CubeSurface6unfoldENS_10CubeLayoutE";

    average :: (this: *CubeSurface, channel: s32) -> float #cpp_method #foreign libnvtt "_ZNK4nvtt11CubeSurface7averageEi";
    range :: (this: *CubeSurface, channel: s32, minimum_ptr: *float, maximum_ptr: *float) -> void #cpp_method #foreign libnvtt "_ZNK4nvtt11CubeSurface5rangeEiPfS1_";
    clamp :: (this: *CubeSurface, channel: s32, low: float = 0.0, high: float = 1.0) -> void #cpp_method #foreign libnvtt "_ZN4nvtt11CubeSurface5clampEiff";

    irradianceFilter :: (this: *CubeSurface, size: s32, fixupMethod: EdgeFixup) -> CubeSurface #cpp_method #cpp_return_type_is_non_pod #foreign libnvtt "_ZNK4nvtt11CubeSurface16irradianceFilterEiNS_9EdgeFixupE";
    cosinePowerFilter :: (this: *CubeSurface, size: s32, cosinePower: float, fixupMethod: EdgeFixup) -> CubeSurface #cpp_method #cpp_return_type_is_non_pod #foreign libnvtt "_ZNK4nvtt11CubeSurface17cosinePowerFilterEifNS_9EdgeFixupE";

    fastResample :: (this: *CubeSurface, size: s32, fixupMethod: EdgeFixup) -> CubeSurface #cpp_method #cpp_return_type_is_non_pod #foreign libnvtt "_ZNK4nvtt11CubeSurface12fastResampleEiNS_9EdgeFixupE";

    toLinear :: (this: *CubeSurface, gamma: float) -> void #cpp_method #foreign libnvtt "_ZN4nvtt11CubeSurface8toLinearEf";
    toGamma :: (this: *CubeSurface, gamma: float) -> void #cpp_method #foreign libnvtt "_ZN4nvtt11CubeSurface7toGammaEf";

    detach :: (this: *CubeSurface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt11CubeSurface6detachEv";

    Private :: struct {}
    m: *Private;
}
```
### `Decoder`

Variant 1 (Windows):

```jai
Decoder :: enum s32 {
    D3D10 :: 0;
    D3D9  :: 1;
    NV5x  :: 2;
}
```
Variant 2 (Linux):

```jai
Decoder :: enum u32 {
    D3D10 :: 0;
    D3D9  :: 1;
    NV5x  :: 2;
}
```
### `EdgeFixup`

Variant 1 (Windows):

```jai
EdgeFixup :: enum s32 {
    None    :: 0;
    Stretch :: 1;
    Warp    :: 2;
    Average :: 3;
}
```
Variant 2 (Linux):

```jai
EdgeFixup :: enum u32 {
    None    :: 0;
    Stretch :: 1;
    Warp    :: 2;
    Average :: 3;
}
```
### `Error`

Variant 1 (Windows):

```jai
Error :: enum s32 {
    Unknown                 :: 0;
    InvalidInput            :: 1;
    UnsupportedFeature      :: 2;
    CudaError               :: 3;
    FileOpen                :: 4;
    FileWrite               :: 5;
    UnsupportedOutputFormat :: 6;
    Count                   :: 7;
}
```
Variant 2 (Linux):

```jai
Error :: enum u32 {
    Unknown                 :: 0;
    InvalidInput            :: 1;
    UnsupportedFeature      :: 2;
    CudaError               :: 3;
    FileOpen                :: 4;
    FileWrite               :: 5;
    UnsupportedOutputFormat :: 6;
    Count                   :: 7;
}
```
### `ErrorHandler`

Variant 1 (Windows):

```jai
ErrorHandler :: struct {
    vtable: *ErrorHandler_VTable;
}
```
Variant 2 (Linux):

```jai
ErrorHandler :: struct {
    vtable: *ErrorHandler_VTable;

    Destructor_Base :: (this: *ErrorHandler) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12ErrorHandlerD2Ev";

    virtual_Destructor :: (this: *ErrorHandler) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12ErrorHandlerD1Ev";
    virtual_Destructor_Deleting :: (this: *ErrorHandler) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12ErrorHandlerD0Ev";
}
```
### `ErrorHandler_VTable`

Variant 1 (Windows):

```jai
ErrorHandler_VTable :: struct #type_info_none {
    Destructor: #type (this: *ErrorHandler, flags: Msvc_Destructor_Flags = .NONE) -> void #cpp_method;

    error:      #type (this: *ErrorHandler, e: Error) -> void #cpp_method;
}
```
Variant 2 (Linux):

```jai
ErrorHandler_VTable :: struct #type_info_none {
    Destructor:          #type (this: *ErrorHandler) -> void #cpp_method;
    Destructor_Deleting: #type (this: *ErrorHandler) -> void #cpp_method;

    error:               #type (this: *ErrorHandler, e: Error) -> void #cpp_method;
}
```
### `Format`

Variant 1 (Windows):

```jai
Format :: enum s32 {
    RGB           :: 0;
    RGBA          :: 0;

    DXT1          :: 1;
    DXT1a         :: 2;
    DXT3          :: 3;
    DXT5          :: 4;
    DXT5n         :: 5;

    BC1           :: 1;
    BC1a          :: 2;
    BC2           :: 3;
    BC3           :: 4;
    BC3n          :: 5;
    BC4           :: 6;
    BC5           :: 7;

    DXT1n         :: 8;
    CTX1          :: 9;

    BC6           :: 10;
    BC7           :: 11;

    BC3_RGBM      :: 12;

    ETC1          :: 13;
    ETC2_R        :: 14;
    ETC2_RG       :: 15;
    ETC2_RGB      :: 16;
    ETC2_RGBA     :: 17;
    ETC2_RGB_A1   :: 18;

    ETC2_RGBM     :: 19;

    PVR_2BPP_RGB  :: 20;
    PVR_4BPP_RGB  :: 21;
    PVR_2BPP_RGBA :: 22;
    PVR_4BPP_RGBA :: 23;

    Count         :: 24;
}
```
Variant 2 (Linux):

```jai
Format :: enum u32 {
    RGB           :: 0;
    RGBA          :: 0;

    DXT1          :: 1;
    DXT1a         :: 2;
    DXT3          :: 3;
    DXT5          :: 4;
    DXT5n         :: 5;

    BC1           :: 1;
    BC1a          :: 2;
    BC2           :: 3;
    BC3           :: 4;
    BC3n          :: 5;
    BC4           :: 6;
    BC5           :: 7;

    DXT1n         :: 8;
    CTX1          :: 9;

    BC6           :: 10;
    BC7           :: 11;

    BC3_RGBM      :: 12;

    ETC1          :: 13;
    ETC2_R        :: 14;
    ETC2_RG       :: 15;
    ETC2_RGB      :: 16;
    ETC2_RGBA     :: 17;
    ETC2_RGB_A1   :: 18;

    ETC2_RGBM     :: 19;

    PVR_2BPP_RGB  :: 20;
    PVR_4BPP_RGB  :: 21;
    PVR_2BPP_RGBA :: 22;
    PVR_4BPP_RGBA :: 23;

    Count         :: 24;
}
```
### `InputFormat`

Variant 1 (Windows):

```jai
InputFormat :: enum s32 {
    BGRA_8UB :: 0;
    RGBA_16F :: 1;
    RGBA_32F :: 2;
    R_32F    :: 3;
}
```
Variant 2 (Linux):

```jai
InputFormat :: enum u32 {
    BGRA_8UB :: 0;
    RGBA_16F :: 1;
    RGBA_32F :: 2;
    R_32F    :: 3;
}
```
### `InputOptions`

Variant 1 (Windows):

```jai
InputOptions :: struct {
    Private :: struct {}
    m: *Private;

    Constructor :: (this: *InputOptions) -> void #cpp_method #foreign libnvtt "??0InputOptions@nvtt@@QEAA@XZ";
    Destructor :: (this: *InputOptions) -> void #cpp_method #foreign libnvtt "??1InputOptions@nvtt@@QEAA@XZ";

    reset :: (this: *InputOptions) -> void #cpp_method #foreign libnvtt "?reset@InputOptions@nvtt@@QEAAXXZ";

    setTextureLayout :: (this: *InputOptions, type: TextureType, w: s32, h: s32, d: s32 = 1, arraySize: s32 = 1) -> void #cpp_method #foreign libnvtt "?setTextureLayout@InputOptions@nvtt@@QEAAXW4TextureType@2@HHHH@Z";
    resetTextureLayout :: (this: *InputOptions) -> void #cpp_method #foreign libnvtt "?resetTextureLayout@InputOptions@nvtt@@QEAAXXZ";

    setMipmapData :: (this: *InputOptions, data: *void, w: s32, h: s32, d: s32 = 1, face: s32 = 0, mipmap: s32 = 0) -> bool #cpp_method #foreign libnvtt "?setMipmapData@InputOptions@nvtt@@QEAA_NPEBXHHHHH@Z";

    setFormat :: (this: *InputOptions, format: InputFormat) -> void #cpp_method #foreign libnvtt "?setFormat@InputOptions@nvtt@@QEAAXW4InputFormat@2@@Z";

    setAlphaMode :: (this: *InputOptions, alphaMode: AlphaMode) -> void #cpp_method #foreign libnvtt "?setAlphaMode@InputOptions@nvtt@@QEAAXW4AlphaMode@2@@Z";

    setGamma :: (this: *InputOptions, inputGamma: float, outputGamma: float) -> void #cpp_method #foreign libnvtt "?setGamma@InputOptions@nvtt@@QEAAXMM@Z";

    setWrapMode :: (this: *InputOptions, mode: WrapMode) -> void #cpp_method #foreign libnvtt "?setWrapMode@InputOptions@nvtt@@QEAAXW4WrapMode@2@@Z";

    setMipmapFilter :: (this: *InputOptions, filter: MipmapFilter) -> void #cpp_method #foreign libnvtt "?setMipmapFilter@InputOptions@nvtt@@QEAAXW4MipmapFilter@2@@Z";
    setMipmapGeneration :: (this: *InputOptions, enabled: bool, maxLevel: s32 = -1) -> void #cpp_method #foreign libnvtt "?setMipmapGeneration@InputOptions@nvtt@@QEAAX_NH@Z";
    setKaiserParameters :: (this: *InputOptions, width: float, alpha: float, stretch: float) -> void #cpp_method #foreign libnvtt "?setKaiserParameters@InputOptions@nvtt@@QEAAXMMM@Z";

    setNormalMap :: (this: *InputOptions, b: bool) -> void #cpp_method #foreign libnvtt "?setNormalMap@InputOptions@nvtt@@QEAAX_N@Z";
    setConvertToNormalMap :: (this: *InputOptions, convert: bool) -> void #cpp_method #foreign libnvtt "?setConvertToNormalMap@InputOptions@nvtt@@QEAAX_N@Z";
    setHeightEvaluation :: (this: *InputOptions, redScale: float, greenScale: float, blueScale: float, alphaScale: float) -> void #cpp_method #foreign libnvtt "?setHeightEvaluation@InputOptions@nvtt@@QEAAXMMMM@Z";
    setNormalFilter :: (this: *InputOptions, sm: float, medium: float, big: float, large: float) -> void #cpp_method #foreign libnvtt "?setNormalFilter@InputOptions@nvtt@@QEAAXMMMM@Z";
    setNormalizeMipmaps :: (this: *InputOptions, b: bool) -> void #cpp_method #foreign libnvtt "?setNormalizeMipmaps@InputOptions@nvtt@@QEAAX_N@Z";

    setMaxExtents :: (this: *InputOptions, d: s32) -> void #cpp_method #foreign libnvtt "?setMaxExtents@InputOptions@nvtt@@QEAAXH@Z";
    setRoundMode :: (this: *InputOptions, mode: RoundMode) -> void #cpp_method #foreign libnvtt "?setRoundMode@InputOptions@nvtt@@QEAAXW4RoundMode@2@@Z";
}
```
Variant 2 (Linux):

```jai
InputOptions :: struct {
    Private :: struct {}
    m: *Private;

    Constructor :: (this: *InputOptions) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12InputOptionsC1Ev";
    Destructor_Base :: (this: *InputOptions) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12InputOptionsD2Ev";
    Destructor :: (this: *InputOptions) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12InputOptionsD1Ev";

    reset :: (this: *InputOptions) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12InputOptions5resetEv";

    setTextureLayout :: (this: *InputOptions, type: TextureType, w: s32, h: s32, d: s32 = 1, arraySize: s32 = 1) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12InputOptions16setTextureLayoutENS_11TextureTypeEiiii";
    resetTextureLayout :: (this: *InputOptions) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12InputOptions18resetTextureLayoutEv";

    setMipmapData :: (this: *InputOptions, data: *void, w: s32, h: s32, d: s32 = 1, face: s32 = 0, mipmap: s32 = 0) -> bool #cpp_method #foreign libnvtt "_ZN4nvtt12InputOptions13setMipmapDataEPKviiiii";

    setFormat :: (this: *InputOptions, format: InputFormat) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12InputOptions9setFormatENS_11InputFormatE";

    setAlphaMode :: (this: *InputOptions, alphaMode: AlphaMode) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12InputOptions12setAlphaModeENS_9AlphaModeE";

    setGamma :: (this: *InputOptions, inputGamma: float, outputGamma: float) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12InputOptions8setGammaEff";

    setWrapMode :: (this: *InputOptions, mode: WrapMode) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12InputOptions11setWrapModeENS_8WrapModeE";

    setMipmapFilter :: (this: *InputOptions, filter: MipmapFilter) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12InputOptions15setMipmapFilterENS_12MipmapFilterE";
    setMipmapGeneration :: (this: *InputOptions, enabled: bool, maxLevel: s32 = -1) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12InputOptions19setMipmapGenerationEbi";
    setKaiserParameters :: (this: *InputOptions, width: float, alpha: float, stretch: float) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12InputOptions19setKaiserParametersEfff";

    setNormalMap :: (this: *InputOptions, b: bool) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12InputOptions12setNormalMapEb";
    setConvertToNormalMap :: (this: *InputOptions, convert: bool) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12InputOptions21setConvertToNormalMapEb";
    setHeightEvaluation :: (this: *InputOptions, redScale: float, greenScale: float, blueScale: float, alphaScale: float) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12InputOptions19setHeightEvaluationEffff";
    setNormalFilter :: (this: *InputOptions, sm: float, medium: float, big: float, large: float) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12InputOptions15setNormalFilterEffff";
    setNormalizeMipmaps :: (this: *InputOptions, b: bool) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12InputOptions19setNormalizeMipmapsEb";

    setMaxExtents :: (this: *InputOptions, d: s32) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12InputOptions13setMaxExtentsEi";
    setRoundMode :: (this: *InputOptions, mode: RoundMode) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12InputOptions12setRoundModeENS_9RoundModeE";
}
```
### `MipmapFilter`

Variant 1 (Windows):

```jai
MipmapFilter :: enum s32 {
    Box      :: 0;
    Triangle :: 1;
    Kaiser   :: 2;
}
```
Variant 2 (Linux):

```jai
MipmapFilter :: enum u32 {
    Box      :: 0;
    Triangle :: 1;
    Kaiser   :: 2;
}
```
### `NormalTransform`

Variant 1 (Windows):

```jai
NormalTransform :: enum s32 {
    Orthographic  :: 0;
    Stereographic :: 1;
    Paraboloid    :: 2;
    Quartic       :: 3;
}
```
Variant 2 (Linux):

```jai
NormalTransform :: enum u32 {
    Orthographic  :: 0;
    Stereographic :: 1;
    Paraboloid    :: 2;
    Quartic       :: 3;
}
```
### `OutputHandler`

Variant 1 (Windows):

```jai
OutputHandler :: struct {
    vtable: *OutputHandler_VTable;
}
```
Variant 2 (Linux):

```jai
OutputHandler :: struct {
    vtable: *OutputHandler_VTable;

    Destructor_Base :: (this: *OutputHandler) -> void #cpp_method #foreign libnvtt "_ZN4nvtt13OutputHandlerD2Ev";

    virtual_Destructor :: (this: *OutputHandler) -> void #cpp_method #foreign libnvtt "_ZN4nvtt13OutputHandlerD1Ev";
    virtual_Destructor_Deleting :: (this: *OutputHandler) -> void #cpp_method #foreign libnvtt "_ZN4nvtt13OutputHandlerD0Ev";
}
```
### `OutputHandler_VTable`

Variant 1 (Windows):

```jai
OutputHandler_VTable :: struct #type_info_none {
    Destructor: #type (this: *OutputHandler, flags: Msvc_Destructor_Flags = .NONE) -> void #cpp_method;

    beginImage: #type (this: *OutputHandler, size: s32, width: s32, height: s32, depth: s32, face: s32, miplevel: s32) -> void #cpp_method;

    writeData:  #type (this: *OutputHandler, data: *void, size: s32) -> bool #cpp_method;

    endImage:   #type (this: *OutputHandler) -> void #cpp_method;
}
```
Variant 2 (Linux):

```jai
OutputHandler_VTable :: struct #type_info_none {
    Destructor:          #type (this: *OutputHandler) -> void #cpp_method;
    Destructor_Deleting: #type (this: *OutputHandler) -> void #cpp_method;

    beginImage:          #type (this: *OutputHandler, size: s32, width: s32, height: s32, depth: s32, face: s32, miplevel: s32) -> void #cpp_method;

    writeData:           #type (this: *OutputHandler, data: *void, size: s32) -> bool #cpp_method;

    endImage:            #type (this: *OutputHandler) -> void #cpp_method;
}
```
### `OutputOptions`

Variant 1 (Windows):

```jai
OutputOptions :: struct {
    Private :: struct {}
    m: *Private;

    Constructor :: (this: *OutputOptions) -> void #cpp_method #foreign libnvtt "??0OutputOptions@nvtt@@QEAA@XZ";
    Destructor :: (this: *OutputOptions) -> void #cpp_method #foreign libnvtt "??1OutputOptions@nvtt@@QEAA@XZ";

    reset :: (this: *OutputOptions) -> void #cpp_method #foreign libnvtt "?reset@OutputOptions@nvtt@@QEAAXXZ";

    setFileName :: (this: *OutputOptions, fileName: *u8) -> void #cpp_method #foreign libnvtt "?setFileName@OutputOptions@nvtt@@QEAAXPEBD@Z";
    setFileHandle :: (this: *OutputOptions, fp: *void) -> void #cpp_method #foreign libnvtt "?setFileHandle@OutputOptions@nvtt@@QEAAXPEAX@Z";

    setOutputHandler :: (this: *OutputOptions, outputHandler: *OutputHandler) -> void #cpp_method #foreign libnvtt "?setOutputHandler@OutputOptions@nvtt@@QEAAXPEAUOutputHandler@2@@Z";
    setErrorHandler :: (this: *OutputOptions, errorHandler: *ErrorHandler) -> void #cpp_method #foreign libnvtt "?setErrorHandler@OutputOptions@nvtt@@QEAAXPEAUErrorHandler@2@@Z";

    setOutputHeader :: (this: *OutputOptions, outputHeader: bool) -> void #cpp_method #foreign libnvtt "?setOutputHeader@OutputOptions@nvtt@@QEAAX_N@Z";
    setContainer :: (this: *OutputOptions, container: Container) -> void #cpp_method #foreign libnvtt "?setContainer@OutputOptions@nvtt@@QEAAXW4Container@2@@Z";
    setUserVersion :: (this: *OutputOptions, version: s32) -> void #cpp_method #foreign libnvtt "?setUserVersion@OutputOptions@nvtt@@QEAAXH@Z";
    setSrgbFlag :: (this: *OutputOptions, b: bool) -> void #cpp_method #foreign libnvtt "?setSrgbFlag@OutputOptions@nvtt@@QEAAX_N@Z";
}
```
Variant 2 (Linux):

```jai
OutputOptions :: struct {
    Private :: struct {}
    m: *Private;

    Constructor :: (this: *OutputOptions) -> void #cpp_method #foreign libnvtt "_ZN4nvtt13OutputOptionsC1Ev";
    Destructor_Base :: (this: *OutputOptions) -> void #cpp_method #foreign libnvtt "_ZN4nvtt13OutputOptionsD2Ev";
    Destructor :: (this: *OutputOptions) -> void #cpp_method #foreign libnvtt "_ZN4nvtt13OutputOptionsD1Ev";

    reset :: (this: *OutputOptions) -> void #cpp_method #foreign libnvtt "_ZN4nvtt13OutputOptions5resetEv";

    setFileName :: (this: *OutputOptions, fileName: *u8) -> void #cpp_method #foreign libnvtt "_ZN4nvtt13OutputOptions11setFileNameEPKc";
    setFileHandle :: (this: *OutputOptions, fp: *void) -> void #cpp_method #foreign libnvtt "_ZN4nvtt13OutputOptions13setFileHandleEPv";

    setOutputHandler :: (this: *OutputOptions, outputHandler: *OutputHandler) -> void #cpp_method #foreign libnvtt "_ZN4nvtt13OutputOptions16setOutputHandlerEPNS_13OutputHandlerE";
    setErrorHandler :: (this: *OutputOptions, errorHandler: *ErrorHandler) -> void #cpp_method #foreign libnvtt "_ZN4nvtt13OutputOptions15setErrorHandlerEPNS_12ErrorHandlerE";

    setOutputHeader :: (this: *OutputOptions, outputHeader: bool) -> void #cpp_method #foreign libnvtt "_ZN4nvtt13OutputOptions15setOutputHeaderEb";
    setContainer :: (this: *OutputOptions, container: Container) -> void #cpp_method #foreign libnvtt "_ZN4nvtt13OutputOptions12setContainerENS_9ContainerE";
    setUserVersion :: (this: *OutputOptions, version: s32) -> void #cpp_method #foreign libnvtt "_ZN4nvtt13OutputOptions14setUserVersionEi";
    setSrgbFlag :: (this: *OutputOptions, b: bool) -> void #cpp_method #foreign libnvtt "_ZN4nvtt13OutputOptions11setSrgbFlagEb";
}
```
### `PixelType`

Variant 1 (Windows):

```jai
PixelType :: enum s32 {
    UnsignedNorm  :: 0;
    SignedNorm    :: 1;
    UnsignedInt   :: 2;
    SignedInt     :: 3;
    Float         :: 4;
    UnsignedFloat :: 5;
    SharedExp     :: 6;
}
```
Variant 2 (Linux):

```jai
PixelType :: enum u32 {
    UnsignedNorm  :: 0;
    SignedNorm    :: 1;
    UnsignedInt   :: 2;
    SignedInt     :: 3;
    Float         :: 4;
    UnsignedFloat :: 5;
    SharedExp     :: 6;
}
```
### `Quality`

Variant 1 (Windows):

```jai
Quality :: enum s32 {
    Fastest    :: 0;
    Normal     :: 1;
    Production :: 2;
    Highest    :: 3;
}
```
Variant 2 (Linux):

```jai
Quality :: enum u32 {
    Fastest    :: 0;
    Normal     :: 1;
    Production :: 2;
    Highest    :: 3;
}
```
### `ResizeFilter`

Variant 1 (Windows):

```jai
ResizeFilter :: enum s32 {
    Box      :: 0;
    Triangle :: 1;
    Kaiser   :: 2;
    Mitchell :: 3;
}
```
Variant 2 (Linux):

```jai
ResizeFilter :: enum u32 {
    Box      :: 0;
    Triangle :: 1;
    Kaiser   :: 2;
    Mitchell :: 3;
}
```
### `RoundMode`

Variant 1 (Windows):

```jai
RoundMode :: enum s32 {
    None                     :: 0;
    ToNextPowerOfTwo         :: 1;
    ToNearestPowerOfTwo      :: 2;
    ToPreviousPowerOfTwo     :: 3;
    ToNextMultipleOfFour     :: 4;
    ToNearestMultipleOfFour  :: 5;
    ToPreviousMultipleOfFour :: 6;
}
```
Variant 2 (Linux):

```jai
RoundMode :: enum u32 {
    None                     :: 0;
    ToNextPowerOfTwo         :: 1;
    ToNearestPowerOfTwo      :: 2;
    ToPreviousPowerOfTwo     :: 3;
    ToNextMultipleOfFour     :: 4;
    ToNearestMultipleOfFour  :: 5;
    ToPreviousMultipleOfFour :: 6;
}
```
### `ShapeRestriction`

Variant 1 (Windows):

```jai
ShapeRestriction :: enum s32 {
    None   :: 0;
    Square :: 1;
}
```
Variant 2 (Linux):

```jai
ShapeRestriction :: enum u32 {
    None   :: 0;
    Square :: 1;
}
```
### `Surface`

Variant 1 (Windows):

```jai
Surface :: struct #type_info_none {
Constructor :: (this: *Surface) -> void #cpp_method #foreign libnvtt "??0Surface@nvtt@@QEAA@XZ"; CopyConstructor :: (this: *Surface, img: *Surface) -> void #cpp_method #foreign libnvtt "??0Surface@nvtt@@QEAA@AEBU01@@Z"; CopyConstructor :: (this: *Surface, img: Surface) #no_context;
Destructor :: (this: *Surface) -> void #cpp_method #foreign libnvtt "??1Surface@nvtt@@QEAA@XZ";
copyChannel :: (this: *Surface, srcImage: *Surface, srcChannel: s32, dstChannel: s32) -> bool #cpp_method #foreign libnvtt "?copyChannel@Surface@nvtt@@QEAA_NAEBU12@HH@Z"; copyChannel :: (this: *Surface, srcImage: Surface, srcChannel: s32, dstChannel: s32) -> bool #no_context;

addChannel :: (this: *Surface, img: *Surface, srcChannel: s32, dstChannel: s32, scale: float) -> bool #cpp_method #foreign libnvtt "?addChannel@Surface@nvtt@@QEAA_NAEBU12@HHM@Z"; addChannel :: (this: *Surface, img: Surface, srcChannel: s32, dstChannel: s32, scale: float) -> bool #no_context;

copy :: (this: *Surface, src: *Surface, xsrc: s32, ysrc: s32, zsrc: s32, xsize: s32, ysize: s32, zsize: s32, xdst: s32, ydst: s32, zdst: s32) -> bool #cpp_method #foreign libnvtt "?copy@Surface@nvtt@@QEAA_NAEBU12@HHHHHHHHH@Z"; copy :: (this: *Surface, src: Surface, xsrc: s32, ysrc: s32, zsrc: s32, xsize: s32, ysize: s32, zsize: s32, xdst: s32, ydst: s32, zdst: s32) -> bool #no_context;

    Private :: struct {}
    m: *Private;
}
```
Variant 2 (Linux):

```jai
Surface :: struct #type_info_none {
Constructor :: (this: *Surface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7SurfaceC1Ev"; CopyConstructor :: (this: *Surface, img: *Surface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7SurfaceC1ERKS0_"; CopyConstructor :: (this: *Surface, img: Surface) #no_context;
Destructor_Base :: (this: *Surface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7SurfaceD2Ev"; Destructor :: (this: *Surface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7SurfaceD1Ev";
copyChannel :: (this: *Surface, srcImage: *Surface, srcChannel: s32, dstChannel: s32) -> bool #cpp_method #foreign libnvtt "_ZN4nvtt7Surface11copyChannelERKS0_ii"; copyChannel :: (this: *Surface, srcImage: Surface, srcChannel: s32, dstChannel: s32) -> bool #no_context;

addChannel :: (this: *Surface, img: *Surface, srcChannel: s32, dstChannel: s32, scale: float) -> bool #cpp_method #foreign libnvtt "_ZN4nvtt7Surface10addChannelERKS0_iif"; addChannel :: (this: *Surface, img: Surface, srcChannel: s32, dstChannel: s32, scale: float) -> bool #no_context;

copy :: (this: *Surface, src: *Surface, xsrc: s32, ysrc: s32, zsrc: s32, xsize: s32, ysize: s32, zsize: s32, xdst: s32, ydst: s32, zdst: s32) -> bool #cpp_method #foreign libnvtt "_ZN4nvtt7Surface4copyERKS0_iiiiiiiii"; copy :: (this: *Surface, src: Surface, xsrc: s32, ysrc: s32, zsrc: s32, xsize: s32, ysize: s32, zsize: s32, xdst: s32, ydst: s32, zdst: s32) -> bool #no_context;

    detach :: (this: *Surface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface6detachEv";

    Private :: struct {}
    m: *Private;
}
```
### `TaskDispatcher`

Variant 1 (Windows):

```jai
TaskDispatcher :: struct {
    vtable: *TaskDispatcher_VTable;
}
```
Variant 2 (Linux):

```jai
TaskDispatcher :: struct {
    vtable: *TaskDispatcher_VTable;

    Destructor_Base :: (this: *TaskDispatcher) -> void #cpp_method #foreign libnvtt "_ZN4nvtt14TaskDispatcherD2Ev";

    virtual_Destructor :: (this: *TaskDispatcher) -> void #cpp_method #foreign libnvtt "_ZN4nvtt14TaskDispatcherD1Ev";
    virtual_Destructor_Deleting :: (this: *TaskDispatcher) -> void #cpp_method #foreign libnvtt "_ZN4nvtt14TaskDispatcherD0Ev";
}
```
### `TaskDispatcher_VTable`

Variant 1 (Windows):

```jai
TaskDispatcher_VTable :: struct #type_info_none {
    Destructor: #type (this: *TaskDispatcher, flags: Msvc_Destructor_Flags = .NONE) -> void #cpp_method;

    dispatch:   #type (this: *TaskDispatcher, task: Task, _context: *void, count: s32) -> void #cpp_method;
}
```
Variant 2 (Linux):

```jai
TaskDispatcher_VTable :: struct #type_info_none {
    Destructor:          #type (this: *TaskDispatcher) -> void #cpp_method;
    Destructor_Deleting: #type (this: *TaskDispatcher) -> void #cpp_method;

    dispatch:            #type (this: *TaskDispatcher, task: Task, _context: *void, count: s32) -> void #cpp_method;
}
```
### `TextureType`

Variant 1 (Windows):

```jai
TextureType :: enum s32 {
    _2D   :: 0;
    Cube  :: 1;
    _3D   :: 2;
    Array :: 3;
}
```
Variant 2 (Linux):

```jai
TextureType :: enum u32 {
    _2D   :: 0;
    Cube  :: 1;
    _3D   :: 2;
    Array :: 3;
}
```
### `ToneMapper`

Variant 1 (Windows):

```jai
ToneMapper :: enum s32 {
    Linear    :: 0;
    Reindhart :: 1;
    Halo      :: 2;
    Lightmap  :: 3;
}
```
Variant 2 (Linux):

```jai
ToneMapper :: enum u32 {
    Linear    :: 0;
    Reindhart :: 1;
    Halo      :: 2;
    Lightmap  :: 3;
}
```
### `WrapMode`

Variant 1 (Windows):

```jai
WrapMode :: enum s32 {
    Clamp  :: 0;
    Repeat :: 1;
    Mirror :: 2;
}
```
Variant 2 (Linux):

```jai
WrapMode :: enum u32 {
    Clamp  :: 0;
    Repeat :: 1;
    Mirror :: 2;
}
```
## Contracts and constraints

This module binds NVIDIA Texture Tools 2.1.0 (`NVTT_VERSION == 20100`) through its public C++ API. `Compressor`, `Surface`, and `CubeSurface` member methods follow the corresponding NVTT declarations for that version; the generated binding flattens the `nvtt` namespace, detects enum prefixes, omits the conflicting `Context` name, and gives selected types custom type-info behavior. This NVTT version and generation contract define the member API represented by the target-specific bindings; the separate generated signatures in this page retain Jai-specific aliases and ABI details. `Surface` and related non-POD objects must be created, copied, and destroyed according to the NVTT C++ lifetime contract. Output and error handlers are callback interfaces, so the Jai object and callback storage must remain live while native code can call them. The packaged libraries and generated declarations are separate for Windows, Linux, and macOS; the API is unavailable on other targets. Compression results, format support, and errors follow NVTT, and the binding itself does not validate image dimensions or data buffers. See the [NVIDIA Texture Tools API](https://github.com/castano/nvidia-texture-tools) for the version-pinned public C++ contract; `NVTT_VERSION` is the binding's version pin.


Platform coverage note: PS5 branches exist, but their declarations are not specified here; exports for those targets are unspecified.

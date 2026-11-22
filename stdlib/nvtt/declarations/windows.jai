// Generated from docs/specs/stdlib/modules/nvtt.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

// The declarations for `OS == .WINDOWS`, loaded by declarations.jai.

#scope_file
#if OS == .WINDOWS {
    #import "Windows";
}

#if OS == .MACOS {
    #library,system,link_always "c++"; @native(external)
} else #if OS == .LINUX {
    #library,system,link_always "stdc++"; @native(external)
}

libnvtt :: #library,system "nvtt"; @native(external)
#scope_export

AlphaMode :: enum s32 {
    None :: 0;
    Transparency :: 1;
    Premultiplied :: 2;

}

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

Compressor :: struct #type_info_procedures_are_void_pointers {
    Private :: struct {}
    m: *Private;

    Constructor :: (this: *Compressor) -> void #cpp_method #foreign libnvtt "??0Compressor@nvtt@@QEAA@XZ";
    Destructor :: (this: *Compressor) -> void #cpp_method #foreign libnvtt "??1Compressor@nvtt@@QEAA@XZ";
    estimateSize :: (this: *Compressor, inputOptions: *InputOptions, compressionOptions: *CompressionOptions) -> s32 #cpp_method #foreign libnvtt "?estimateSize@Compressor@nvtt@@QEBAHAEBUInputOptions@2@AEBUCompressionOptions@2@@Z";
    estimateSize :: (this: *Compressor, inputOptions: InputOptions, compressionOptions: CompressionOptions) -> s32 #no_context {
        return estimateSize(this, *inputOptions, *compressionOptions);
    }

    outputHeader :: (this: *Compressor, img: *Surface, mipmapCount: s32, compressionOptions: *CompressionOptions, outputOptions: *OutputOptions) -> bool #cpp_method #foreign libnvtt "?outputHeader@Compressor@nvtt@@QEBA_NAEBUSurface@2@HAEBUCompressionOptions@2@AEBUOutputOptions@2@@Z";
    outputHeader :: (this: *Compressor, img: Surface, mipmapCount: s32, compressionOptions: CompressionOptions, outputOptions: OutputOptions) -> bool #no_context {
        return outputHeader(this, *img, mipmapCount, *compressionOptions, *outputOptions);
    }
    compress :: (this: *Compressor, img: *Surface, face: s32, mipmap: s32, compressionOptions: *CompressionOptions, outputOptions: *OutputOptions) -> bool #cpp_method #foreign libnvtt "?compress@Compressor@nvtt@@QEBA_NAEBUSurface@2@HHAEBUCompressionOptions@2@AEBUOutputOptions@2@@Z";
    compress :: (this: *Compressor, img: Surface, face: s32, mipmap: s32, compressionOptions: CompressionOptions, outputOptions: OutputOptions) -> bool #no_context {
        return compress(this, *img, face, mipmap, *compressionOptions, *outputOptions);
    }
    estimateSize :: (this: *Compressor, img: *Surface, mipmapCount: s32, compressionOptions: *CompressionOptions) -> s32 #cpp_method #foreign libnvtt "?estimateSize@Compressor@nvtt@@QEBAHAEBUSurface@2@HAEBUCompressionOptions@2@@Z";
    estimateSize :: (this: *Compressor, img: Surface, mipmapCount: s32, compressionOptions: CompressionOptions) -> s32 #no_context {
        return estimateSize(this, *img, mipmapCount, *compressionOptions);
    }

    outputHeader :: (this: *Compressor, cube: *CubeSurface, mipmapCount: s32, compressionOptions: *CompressionOptions, outputOptions: *OutputOptions) -> bool #cpp_method #foreign libnvtt "?outputHeader@Compressor@nvtt@@QEBA_NAEBUCubeSurface@2@HAEBUCompressionOptions@2@AEBUOutputOptions@2@@Z";
    outputHeader :: (this: *Compressor, cube: CubeSurface, mipmapCount: s32, compressionOptions: CompressionOptions, outputOptions: OutputOptions) -> bool #no_context {
        return outputHeader(this, *cube, mipmapCount, *compressionOptions, *outputOptions);
    }
    compress :: (this: *Compressor, cube: *CubeSurface, mipmap: s32, compressionOptions: *CompressionOptions, outputOptions: *OutputOptions) -> bool #cpp_method #foreign libnvtt "?compress@Compressor@nvtt@@QEBA_NAEBUCubeSurface@2@HAEBUCompressionOptions@2@AEBUOutputOptions@2@@Z";
    compress :: (this: *Compressor, cube: CubeSurface, mipmap: s32, compressionOptions: CompressionOptions, outputOptions: OutputOptions) -> bool #no_context {
        return compress(this, *cube, mipmap, *compressionOptions, *outputOptions);
    }
    estimateSize :: (this: *Compressor, cube: *CubeSurface, mipmapCount: s32, compressionOptions: *CompressionOptions) -> s32 #cpp_method #foreign libnvtt "?estimateSize@Compressor@nvtt@@QEBAHAEBUCubeSurface@2@HAEBUCompressionOptions@2@@Z";
    estimateSize :: (this: *Compressor, cube: CubeSurface, mipmapCount: s32, compressionOptions: CompressionOptions) -> s32 #no_context {
        return estimateSize(this, *cube, mipmapCount, *compressionOptions);
    }

    outputHeader :: (this: *Compressor, type: TextureType, w: s32, h: s32, d: s32, arraySize: s32, mipmapCount: s32, isNormalMap: bool, compressionOptions: *CompressionOptions, outputOptions: *OutputOptions) -> bool #cpp_method #foreign libnvtt "?outputHeader@Compressor@nvtt@@QEBA_NW4TextureType@2@HHHHH_NAEBUCompressionOptions@2@AEBUOutputOptions@2@@Z";
    outputHeader :: (this: *Compressor, type: TextureType, w: s32, h: s32, d: s32, arraySize: s32, mipmapCount: s32, isNormalMap: bool, compressionOptions: CompressionOptions, outputOptions: OutputOptions) -> bool #no_context {
        return outputHeader(this, type, w, h, d, arraySize, mipmapCount, isNormalMap, *compressionOptions, *outputOptions);
    }
    compress :: (this: *Compressor, w: s32, h: s32, d: s32, face: s32, mipmap: s32, rgba: *float, compressionOptions: *CompressionOptions, outputOptions: *OutputOptions) -> bool #cpp_method #foreign libnvtt "?compress@Compressor@nvtt@@QEBA_NHHHHHPEBMAEBUCompressionOptions@2@AEBUOutputOptions@2@@Z";
    compress :: (this: *Compressor, w: s32, h: s32, d: s32, face: s32, mipmap: s32, rgba: *float, compressionOptions: CompressionOptions, outputOptions: OutputOptions) -> bool #no_context {
        return compress(this, w, h, d, face, mipmap, rgba, *compressionOptions, *outputOptions);
    }
    estimateSize :: (this: *Compressor, w: s32, h: s32, d: s32, mipmapCount: s32, compressionOptions: *CompressionOptions) -> s32 #cpp_method #foreign libnvtt "?estimateSize@Compressor@nvtt@@QEBAHHHHHAEBUCompressionOptions@2@@Z";
    estimateSize :: (this: *Compressor, w: s32, h: s32, d: s32, mipmapCount: s32, compressionOptions: CompressionOptions) -> s32 #no_context {
        return estimateSize(this, w, h, d, mipmapCount, *compressionOptions);
    }

    enableCudaAcceleration :: (this: *Compressor, enable: bool) -> void #cpp_method #foreign libnvtt "?enableCudaAcceleration@Compressor@nvtt@@QEAAX_N@Z";
    isCudaAccelerationEnabled :: (this: *Compressor) -> bool #cpp_method #foreign libnvtt "?isCudaAccelerationEnabled@Compressor@nvtt@@QEBA_NXZ";
    process :: (this: *Compressor, inputOptions: *InputOptions, compressionOptions: *CompressionOptions, outputOptions: *OutputOptions) -> bool #cpp_method #foreign libnvtt "?process@Compressor@nvtt@@QEBA_NAEBUInputOptions@2@AEBUCompressionOptions@2@AEBUOutputOptions@2@@Z";
    setTaskDispatcher :: (this: *Compressor, disp: *TaskDispatcher) -> void #cpp_method #foreign libnvtt "?setTaskDispatcher@Compressor@nvtt@@QEAAXPEAUTaskDispatcher@2@@Z";
}

Container :: enum s32 {
    DDS :: 0;
    DDS10 :: 1;
    KTX :: 2;

}

CubeLayout :: enum s32 {
    VerticalCross :: 0;
    HorizontalCross :: 1;
    Column :: 2;
    Row :: 3;
    LatitudeLongitude :: 4;

}

CubeSurface :: struct #type_info_procedures_are_void_pointers {
    Constructor :: (this: *CubeSurface) -> void #cpp_method #foreign libnvtt "??0CubeSurface@nvtt@@QEAA@XZ";
    CopyConstructor :: (this: *CubeSurface, img: *CubeSurface) -> void #cpp_method #foreign libnvtt "??0CubeSurface@nvtt@@QEAA@AEBU01@@Z";
    CopyConstructor :: (this: *CubeSurface, img: CubeSurface) #no_context {
        CopyConstructor(this, *img);
    }
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

    countMipmaps :: (this: *CubeSurface) -> s32 #cpp_method #foreign libnvtt "?countMipmaps@CubeSurface@nvtt@@QEBAHXZ";
    detach :: (this: *CubeSurface) -> void #cpp_method #foreign libnvtt "?detach@CubeSurface@nvtt@@QEAAXXZ";
    edgeLength :: (this: *CubeSurface) -> s32 #cpp_method #foreign libnvtt "?edgeLength@CubeSurface@nvtt@@QEBAHXZ";
    face :: (this: *CubeSurface, face: s32) -> *Surface #cpp_method #foreign libnvtt "?face@CubeSurface@nvtt@@QEAAAEAUSurface@2@H@Z";
    face :: (this: *CubeSurface, face: s32) -> *Surface #cpp_method #foreign libnvtt "?face@CubeSurface@nvtt@@QEBAAEBUSurface@2@H@Z";
    fold :: (this: *CubeSurface, img: *Surface, layout: CubeLayout) -> void #cpp_method #foreign libnvtt "?fold@CubeSurface@nvtt@@QEAAXAEBUSurface@2@W4CubeLayout@2@@Z";
    isNull :: (this: *CubeSurface) -> bool #cpp_method #foreign libnvtt "?isNull@CubeSurface@nvtt@@QEBA_NXZ";
    load :: (this: *CubeSurface, fileName: *u8, mipmap: s32) -> bool #cpp_method #foreign libnvtt "?load@CubeSurface@nvtt@@QEAA_NPEBDH@Z";
    save :: (this: *CubeSurface, fileName: *u8) -> bool #cpp_method #foreign libnvtt "?save@CubeSurface@nvtt@@QEBA_NPEBD@Z";
}

Decoder :: enum s32 {
    D3D10 :: 0;
    D3D9 :: 1;
    NV5x :: 2;

}

EdgeFixup :: enum s32 {
    None :: 0;
    Stretch :: 1;
    Warp :: 2;
    Average :: 3;

}

Error :: enum s32 {
    Unknown :: 0;
    InvalidInput :: 1;
    UnsupportedFeature :: 2;
    CudaError :: 3;
    FileOpen :: 4;
    FileWrite :: 5;
    UnsupportedOutputFormat :: 6;
    Count :: 7;

}

ErrorHandler :: struct {
    vtable: *ErrorHandler_VTable;

}

ErrorHandler_VTable :: struct #type_info_none {
    Destructor: #type (this: *ErrorHandler, flags: Msvc_Destructor_Flags = .NONE) -> void #cpp_method;

    error: #type (this: *ErrorHandler, e: Error) -> void #cpp_method;

}

Format :: enum s32 {
    RGB :: 0;
    RGBA :: 0;

    DXT1 :: 1;
    DXT1a :: 2;
    DXT3 :: 3;
    DXT5 :: 4;
    DXT5n :: 5;

    BC1 :: 1;
    BC1a :: 2;
    BC2 :: 3;
    BC3 :: 4;
    BC3n :: 5;
    BC4 :: 6;
    BC5 :: 7;

    DXT1n :: 8;
    CTX1 :: 9;

    BC6 :: 10;
    BC7 :: 11;

    BC3_RGBM :: 12;

    ETC1 :: 13;
    ETC2_R :: 14;
    ETC2_RG :: 15;
    ETC2_RGB :: 16;
    ETC2_RGBA :: 17;
    ETC2_RGB_A1 :: 18;

    ETC2_RGBM :: 19;

    PVR_2BPP_RGB :: 20;
    PVR_4BPP_RGB :: 21;
    PVR_2BPP_RGBA :: 22;
    PVR_4BPP_RGBA :: 23;

    Count :: 24;

}

InputFormat :: enum s32 {
    BGRA_8UB :: 0;
    RGBA_16F :: 1;
    RGBA_32F :: 2;
    R_32F :: 3;

}

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

MipmapFilter :: enum s32 {
    Box :: 0;
    Triangle :: 1;
    Kaiser :: 2;

}

NormalTransform :: enum s32 {
    Orthographic :: 0;
    Stereographic :: 1;
    Paraboloid :: 2;
    Quartic :: 3;

}

OutputHandler :: struct {
    vtable: *OutputHandler_VTable;

}

OutputHandler_VTable :: struct #type_info_none {
    Destructor: #type (this: *OutputHandler, flags: Msvc_Destructor_Flags = .NONE) -> void #cpp_method;

    beginImage: #type (this: *OutputHandler, size: s32, width: s32, height: s32, depth: s32, face: s32, miplevel: s32) -> void #cpp_method;

    writeData: #type (this: *OutputHandler, data: *void, size: s32) -> bool #cpp_method;

    endImage: #type (this: *OutputHandler) -> void #cpp_method;

}

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

PixelType :: enum s32 {
    UnsignedNorm :: 0;
    SignedNorm :: 1;
    UnsignedInt :: 2;
    SignedInt :: 3;
    Float :: 4;
    UnsignedFloat :: 5;
    SharedExp :: 6;

}

Quality :: enum s32 {
    Fastest :: 0;
    Normal :: 1;
    Production :: 2;
    Highest :: 3;

}

ResizeFilter :: enum s32 {
    Box :: 0;
    Triangle :: 1;
    Kaiser :: 2;
    Mitchell :: 3;

}

RoundMode :: enum s32 {
    None :: 0;
    ToNextPowerOfTwo :: 1;
    ToNearestPowerOfTwo :: 2;
    ToPreviousPowerOfTwo :: 3;
    ToNextMultipleOfFour :: 4;
    ToNearestMultipleOfFour :: 5;
    ToPreviousMultipleOfFour :: 6;

}

ShapeRestriction :: enum s32 {
    None :: 0;
    Square :: 1;

}

Surface :: struct #type_info_none {
    Constructor :: (this: *Surface) -> void #cpp_method #foreign libnvtt "??0Surface@nvtt@@QEAA@XZ";
    CopyConstructor :: (this: *Surface, img: *Surface) -> void #cpp_method #foreign libnvtt "??0Surface@nvtt@@QEAA@AEBU01@@Z";
    CopyConstructor :: (this: *Surface, img: Surface) #no_context {
        CopyConstructor(this, *img);
    }
    Destructor :: (this: *Surface) -> void #cpp_method #foreign libnvtt "??1Surface@nvtt@@QEAA@XZ";
    copyChannel :: (this: *Surface, srcImage: *Surface, srcChannel: s32, dstChannel: s32) -> bool #cpp_method #foreign libnvtt "?copyChannel@Surface@nvtt@@QEAA_NAEBU12@HH@Z";
    copyChannel :: (this: *Surface, srcImage: Surface, srcChannel: s32, dstChannel: s32) -> bool #no_context {
        return copyChannel(this, *srcImage, srcChannel, dstChannel);
    }

    addChannel :: (this: *Surface, img: *Surface, srcChannel: s32, dstChannel: s32, scale: float) -> bool #cpp_method #foreign libnvtt "?addChannel@Surface@nvtt@@QEAA_NAEBU12@HHM@Z";
    addChannel :: (this: *Surface, img: Surface, srcChannel: s32, dstChannel: s32, scale: float) -> bool #no_context {
        return addChannel(this, *img, srcChannel, dstChannel, scale);
    }

    copy :: (this: *Surface, src: *Surface, xsrc: s32, ysrc: s32, zsrc: s32, xsize: s32, ysize: s32, zsize: s32, xdst: s32, ydst: s32, zdst: s32) -> bool #cpp_method #foreign libnvtt "?copy@Surface@nvtt@@QEAA_NAEBU12@HHHHHHHHH@Z";
    copy :: (this: *Surface, src: Surface, xsrc: s32, ysrc: s32, zsrc: s32, xsize: s32, ysize: s32, zsize: s32, xdst: s32, ydst: s32, zdst: s32) -> bool #no_context {
        return copy(this, *src, xsrc, ysrc, zsrc, xsize, ysize, zsize, xdst, ydst, zdst);
    }

    Private :: struct {}
    m: *Private;

    abs :: (this: *Surface, channel: s32) -> void #cpp_method #foreign libnvtt "?abs@Surface@nvtt@@QEAAXH@Z";
    alphaMode :: (this: *Surface) -> AlphaMode #cpp_method #foreign libnvtt "?alphaMode@Surface@nvtt@@QEBA?AW4AlphaMode@2@XZ";
    alphaTestCoverage :: (this: *Surface, alphaRef: float = 0.5, alpha_channel: s32 = 3) -> float #cpp_method #foreign libnvtt "?alphaTestCoverage@Surface@nvtt@@QEBAMMH@Z";
    average :: (this: *Surface, channel: s32, alpha_channel: s32 = -1, gamma: float = 2.2) -> float #cpp_method #foreign libnvtt "?average@Surface@nvtt@@QEBAMHHM@Z";
    binarize :: (this: *Surface, channel: s32, threshold: float, dither: bool) -> void #cpp_method #foreign libnvtt "?binarize@Surface@nvtt@@QEAAXHM_N@Z";
    blend :: (this: *Surface, r: float, g: float, b: float, a: float, t: float) -> void #cpp_method #foreign libnvtt "?blend@Surface@nvtt@@QEAAXMMMMM@Z";
    blockScaleCoCg :: (this: *Surface, bits: s32 = 5, threshold: float = 0.0) -> void #cpp_method #foreign libnvtt "?blockScaleCoCg@Surface@nvtt@@QEAAXHM@Z";
    buildNextMipmap :: (this: *Surface, filter: MipmapFilter, min_size: s32 = 1) -> bool #cpp_method #foreign libnvtt "?buildNextMipmap@Surface@nvtt@@QEAA_NW4MipmapFilter@2@H@Z";
    buildNextMipmap :: (this: *Surface, filter: MipmapFilter, filterWidth: float, params: *float = null, min_size: s32 = 1) -> bool #cpp_method #foreign libnvtt "?buildNextMipmap@Surface@nvtt@@QEAA_NW4MipmapFilter@2@MPEBMH@Z";
    buildNextMipmapSolidColor :: (this: *Surface, color_components: *float) -> bool #cpp_method #foreign libnvtt "?buildNextMipmapSolidColor@Surface@nvtt@@QEAA_NQEBM@Z";
    canMakeNextMipmap :: (this: *Surface, min_size: s32 = 1) -> bool #cpp_method #foreign libnvtt "?canMakeNextMipmap@Surface@nvtt@@QEAA_NH@Z";
    canvasSize :: (this: *Surface, w: s32, h: s32, d: s32) -> void #cpp_method #foreign libnvtt "?canvasSize@Surface@nvtt@@QEAAXHHH@Z";
    channel :: (this: *Surface, i: s32) -> *float #cpp_method #foreign libnvtt "?channel@Surface@nvtt@@QEBAPEBMH@Z";
    clamp :: (this: *Surface, channel: s32, low: float = 0.0, high: float = 1.0) -> void #cpp_method #foreign libnvtt "?clamp@Surface@nvtt@@QEAAXHMM@Z";
    convolve :: (this: *Surface, channel: s32, kernelSize: s32, kernelData: *float) -> void #cpp_method #foreign libnvtt "?convolve@Surface@nvtt@@QEAAXHHPEAM@Z";
    copyChannel :: (this: *Surface, srcImage: *Surface, srcChannel: s32) -> bool #cpp_method #foreign libnvtt "?copyChannel@Surface@nvtt@@QEAA_NAEBU12@H@Z";
    countMipmaps :: (this: *Surface) -> s32 #cpp_method #foreign libnvtt "?countMipmaps@Surface@nvtt@@QEBAHXZ";
    countMipmaps :: (this: *Surface, min_size: s32) -> s32 #cpp_method #foreign libnvtt "?countMipmaps@Surface@nvtt@@QEBAHH@Z";
    createCleanMap :: (this: *Surface) -> Surface #cpp_method #cpp_return_type_is_non_pod #foreign libnvtt "?createCleanMap@Surface@nvtt@@QEBA?AU12@XZ";
    createSubImage :: (this: *Surface, x0: s32, x1: s32, y0: s32, y1: s32, z0: s32, z1: s32) -> Surface #cpp_method #cpp_return_type_is_non_pod #foreign libnvtt "?createSubImage@Surface@nvtt@@QEBA?AU12@HHHHHH@Z";
    createToksvigMap :: (this: *Surface, power: float) -> Surface #cpp_method #cpp_return_type_is_non_pod #foreign libnvtt "?createToksvigMap@Surface@nvtt@@QEBA?AU12@M@Z";
    data :: (this: *Surface) -> *float #cpp_method #foreign libnvtt "?data@Surface@nvtt@@QEBAPEBMXZ";
    depth :: (this: *Surface) -> s32 #cpp_method #foreign libnvtt "?depth@Surface@nvtt@@QEBAHXZ";
    detach :: (this: *Surface) -> void #cpp_method #foreign libnvtt "?detach@Surface@nvtt@@QEAAXXZ";
    expandNormals :: (this: *Surface, scale: float = 2.0, bias: float = -1.0) -> void #cpp_method #foreign libnvtt "?expandNormals@Surface@nvtt@@QEAAXMM@Z";
    fill :: (this: *Surface, r: float, g: float, b: float, a: float) -> void #cpp_method #foreign libnvtt "?fill@Surface@nvtt@@QEAAXMMMM@Z";
    flipX :: (this: *Surface) -> void #cpp_method #foreign libnvtt "?flipX@Surface@nvtt@@QEAAXXZ";
    flipY :: (this: *Surface) -> void #cpp_method #foreign libnvtt "?flipY@Surface@nvtt@@QEAAXXZ";
    flipZ :: (this: *Surface) -> void #cpp_method #foreign libnvtt "?flipZ@Surface@nvtt@@QEAAXXZ";
    fromLogScale :: (this: *Surface, channel: s32, base: float) -> void #cpp_method #foreign libnvtt "?fromLogScale@Surface@nvtt@@QEAAXHM@Z";
    fromLUVW :: (this: *Surface, range: float = 1.0) -> void #cpp_method #foreign libnvtt "?fromLUVW@Surface@nvtt@@QEAAXM@Z";
    fromRGBE :: (this: *Surface, mantissaBits: s32, exponentBits: s32) -> void #cpp_method #foreign libnvtt "?fromRGBE@Surface@nvtt@@QEAAXHH@Z";
    fromRGBM :: (this: *Surface, range: float = 1.0, threshold: float = 0.25) -> void #cpp_method #foreign libnvtt "?fromRGBM@Surface@nvtt@@QEAAXMM@Z";
    fromYCoCg :: (this: *Surface) -> void #cpp_method #foreign libnvtt "?fromYCoCg@Surface@nvtt@@QEAAXXZ";
    height :: (this: *Surface) -> s32 #cpp_method #foreign libnvtt "?height@Surface@nvtt@@QEBAHXZ";
    histogram :: (this: *Surface, channel: s32, rangeMin: float, rangeMax: float, binCount: s32, binPtr: *s32) -> void #cpp_method #foreign libnvtt "?histogram@Surface@nvtt@@QEBAXHMMHPEAH@Z";
    isNormalMap :: (this: *Surface) -> bool #cpp_method #foreign libnvtt "?isNormalMap@Surface@nvtt@@QEBA_NXZ";
    isNull :: (this: *Surface) -> bool #cpp_method #foreign libnvtt "?isNull@Surface@nvtt@@QEBA_NXZ";
    load :: (this: *Surface, fileName: *u8, hasAlpha: *bool = null) -> bool #cpp_method #foreign libnvtt "?load@Surface@nvtt@@QEAA_NPEBDPEA_N@Z";
    normalizeNormalMap :: (this: *Surface) -> void #cpp_method #foreign libnvtt "?normalizeNormalMap@Surface@nvtt@@QEAAXXZ";
    packNormals :: (this: *Surface, scale: float = 0.5, bias: float = 0.5) -> void #cpp_method #foreign libnvtt "?packNormals@Surface@nvtt@@QEAAXMM@Z";
    premultiplyAlpha :: (this: *Surface) -> void #cpp_method #foreign libnvtt "?premultiplyAlpha@Surface@nvtt@@QEAAXXZ";
    quantize :: (this: *Surface, channel: s32, bits: s32, exactEndPoints: bool, dither: bool) -> void #cpp_method #foreign libnvtt "?quantize@Surface@nvtt@@QEAAXHH_N0@Z";
    range :: (this: *Surface, channel: s32, rangeMin: *float, rangeMax: *float, alpha_channel: s32 = -1, alpha_ref: float = 0.0) -> void #cpp_method #foreign libnvtt "?range@Surface@nvtt@@QEBAXHPEAM0HM@Z";
    reconstructNormals :: (this: *Surface, xform: NormalTransform) -> void #cpp_method #foreign libnvtt "?reconstructNormals@Surface@nvtt@@QEAAXW4NormalTransform@2@@Z";
    resize :: (this: *Surface, w: s32, h: s32, d: s32, filter: ResizeFilter) -> void #cpp_method #foreign libnvtt "?resize@Surface@nvtt@@QEAAXHHHW4ResizeFilter@2@@Z";
    resize :: (this: *Surface, w: s32, h: s32, d: s32, filter: ResizeFilter, filterWidth: float, params: *float = null) -> void #cpp_method #foreign libnvtt "?resize@Surface@nvtt@@QEAAXHHHW4ResizeFilter@2@MPEBM@Z";
    resize :: (this: *Surface, maxExtent: s32, mode: RoundMode, filter: ResizeFilter) -> void #cpp_method #foreign libnvtt "?resize@Surface@nvtt@@QEAAXHW4RoundMode@2@W4ResizeFilter@2@@Z";
    resize :: (this: *Surface, maxExtent: s32, mode: RoundMode, filter: ResizeFilter, filterWidth: float, params: *float = null) -> void #cpp_method #foreign libnvtt "?resize@Surface@nvtt@@QEAAXHW4RoundMode@2@W4ResizeFilter@2@MPEBM@Z";
    resize_make_square :: (this: *Surface, maxExtent: s32, roundMode: RoundMode, filter: ResizeFilter) -> void #cpp_method #foreign libnvtt "?resize_make_square@Surface@nvtt@@QEAAXHW4RoundMode@2@W4ResizeFilter@2@@Z";
    save :: (this: *Surface, fileName: *u8, hasAlpha: bool = false, hdr: bool = false) -> bool #cpp_method #foreign libnvtt "?save@Surface@nvtt@@QEBA_NPEBD_N1@Z";
    scaleAlphaToCoverage :: (this: *Surface, coverage: float, alphaRef: float = 0.5, alpha_channel: s32 = 3) -> void #cpp_method #foreign libnvtt "?scaleAlphaToCoverage@Surface@nvtt@@QEAAXMMH@Z";
    scaleBias :: (this: *Surface, channel: s32, scale: float, bias: float) -> void #cpp_method #foreign libnvtt "?scaleBias@Surface@nvtt@@QEAAXHMM@Z";
    setAlphaMode :: (this: *Surface, alphaMode: AlphaMode) -> void #cpp_method #foreign libnvtt "?setAlphaMode@Surface@nvtt@@QEAAXW4AlphaMode@2@@Z";
    setAtlasBorder :: (this: *Surface, w: s32, h: s32, r: float, g: float, b: float, a: float) -> void #cpp_method #foreign libnvtt "?setAtlasBorder@Surface@nvtt@@QEAAXHHMMMM@Z";
    setBorder :: (this: *Surface, r: float, g: float, b: float, a: float) -> void #cpp_method #foreign libnvtt "?setBorder@Surface@nvtt@@QEAAXMMMM@Z";
    setImage :: (this: *Surface, w: s32, h: s32, d: s32) -> bool #cpp_method #foreign libnvtt "?setImage@Surface@nvtt@@QEAA_NHHH@Z";
    setImage :: (this: *Surface, format: InputFormat, w: s32, h: s32, d: s32, data: *void) -> bool #cpp_method #foreign libnvtt "?setImage@Surface@nvtt@@QEAA_NW4InputFormat@2@HHHPEBX@Z";
    setImage :: (this: *Surface, format: InputFormat, w: s32, h: s32, d: s32, r: *void, g: *void, b: *void, a: *void) -> bool #cpp_method #foreign libnvtt "?setImage@Surface@nvtt@@QEAA_NW4InputFormat@2@HHHPEBX111@Z";
    setImage2D :: (this: *Surface, format: Format, decoder: Decoder, w: s32, h: s32, data: *void) -> bool #cpp_method #foreign libnvtt "?setImage2D@Surface@nvtt@@QEAA_NW4Format@2@W4Decoder@2@HHPEBX@Z";
    setNormalMap :: (this: *Surface, isNormalMap: bool) -> void #cpp_method #foreign libnvtt "?setNormalMap@Surface@nvtt@@QEAAX_N@Z";
    setWrapMode :: (this: *Surface, mode: WrapMode) -> void #cpp_method #foreign libnvtt "?setWrapMode@Surface@nvtt@@QEAAXW4WrapMode@2@@Z";
    swizzle :: (this: *Surface, r: s32, g: s32, b: s32, a: s32) -> void #cpp_method #foreign libnvtt "?swizzle@Surface@nvtt@@QEAAXHHHH@Z";
    toCleanNormalMap :: (this: *Surface) -> void #cpp_method #foreign libnvtt "?toCleanNormalMap@Surface@nvtt@@QEAAXXZ";
    toGamma :: (this: *Surface, gamma: float) -> void #cpp_method #foreign libnvtt "?toGamma@Surface@nvtt@@QEAAXM@Z";
    toGamma :: (this: *Surface, channel: s32, gamma: float) -> void #cpp_method #foreign libnvtt "?toGamma@Surface@nvtt@@QEAAXHM@Z";
    toGreyScale :: (this: *Surface, redScale: float, greenScale: float, blueScale: float, alphaScale: float) -> void #cpp_method #foreign libnvtt "?toGreyScale@Surface@nvtt@@QEAAXMMMM@Z";
    toLinear :: (this: *Surface, gamma: float) -> void #cpp_method #foreign libnvtt "?toLinear@Surface@nvtt@@QEAAXM@Z";
    toLinear :: (this: *Surface, channel: s32, gamma: float) -> void #cpp_method #foreign libnvtt "?toLinear@Surface@nvtt@@QEAAXHM@Z";
    toLinearFromSrgb :: (this: *Surface) -> void #cpp_method #foreign libnvtt "?toLinearFromSrgb@Surface@nvtt@@QEAAXXZ";
    toLM :: (this: *Surface, range: float = 1.0, threshold: float = 0.0) -> void #cpp_method #foreign libnvtt "?toLM@Surface@nvtt@@QEAAXMM@Z";
    toLogScale :: (this: *Surface, channel: s32, base: float) -> void #cpp_method #foreign libnvtt "?toLogScale@Surface@nvtt@@QEAAXHM@Z";
    toLUVW :: (this: *Surface, range: float = 1.0) -> void #cpp_method #foreign libnvtt "?toLUVW@Surface@nvtt@@QEAAXM@Z";
    toneMap :: (this: *Surface, tm: ToneMapper, parameters: *float) -> void #cpp_method #foreign libnvtt "?toneMap@Surface@nvtt@@QEAAXW4ToneMapper@2@PEAM@Z";
    toNormalMap :: (this: *Surface, sm: float, medium: float, big: float, large: float) -> void #cpp_method #foreign libnvtt "?toNormalMap@Surface@nvtt@@QEAAXMMMM@Z";
    toRGBE :: (this: *Surface, mantissaBits: s32, exponentBits: s32) -> void #cpp_method #foreign libnvtt "?toRGBE@Surface@nvtt@@QEAAXHH@Z";
    toRGBM :: (this: *Surface, range: float = 1.0, threshold: float = 0.25) -> void #cpp_method #foreign libnvtt "?toRGBM@Surface@nvtt@@QEAAXMM@Z";
    toSrgb :: (this: *Surface) -> void #cpp_method #foreign libnvtt "?toSrgb@Surface@nvtt@@QEAAXXZ";
    toXenonSrgb :: (this: *Surface) -> void #cpp_method #foreign libnvtt "?toXenonSrgb@Surface@nvtt@@QEAAXXZ";
    toYCoCg :: (this: *Surface) -> void #cpp_method #foreign libnvtt "?toYCoCg@Surface@nvtt@@QEAAXXZ";
    transform :: (this: *Surface, w0: *float, w1: *float, w2: *float, w3: *float, offset: *float) -> void #cpp_method #foreign libnvtt "?transform@Surface@nvtt@@QEAAXQEBM0000@Z";
    transformNormals :: (this: *Surface, xform: NormalTransform) -> void #cpp_method #foreign libnvtt "?transformNormals@Surface@nvtt@@QEAAXW4NormalTransform@2@@Z";
    type :: (this: *Surface) -> TextureType #cpp_method #foreign libnvtt "?type@Surface@nvtt@@QEBA?AW4TextureType@2@XZ";
    width :: (this: *Surface) -> s32 #cpp_method #foreign libnvtt "?width@Surface@nvtt@@QEBAHXZ";
    wrapMode :: (this: *Surface) -> WrapMode #cpp_method #foreign libnvtt "?wrapMode@Surface@nvtt@@QEBA?AW4WrapMode@2@XZ";
}

TaskDispatcher :: struct {
    vtable: *TaskDispatcher_VTable;

}

TaskDispatcher_VTable :: struct #type_info_none {
    Destructor: #type (this: *TaskDispatcher, flags: Msvc_Destructor_Flags = .NONE) -> void #cpp_method;

    dispatch: #type (this: *TaskDispatcher, task: Task, _context: *void, count: s32) -> void #cpp_method;

}

TextureType :: enum s32 {
    _2D :: 0;
    Cube :: 1;
    _3D :: 2;
    Array :: 3;

}

ToneMapper :: enum s32 {
    Linear :: 0;
    Reindhart :: 1;
    Halo :: 2;
    Lightmap :: 3;

}

WrapMode :: enum s32 {
    Clamp :: 0;
    Repeat :: 1;
    Mirror :: 2;

}

Task :: #type (_context: *void, id: s32) -> void #c_call;

WarpFunction :: #type (x: *float, y: *float, d: *float) -> void #c_call;

Context :: Compressor;

NVTT_VERSION :: 20100;

angularError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "?angularError@nvtt@@YAMAEBUSurface@1@0@Z";

cieLabError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "?cieLabError@nvtt@@YAMAEBUSurface@1@0@Z";

diff :: (reference: *Surface, img: *Surface, scale: float) -> Surface #cpp_return_type_is_non_pod #foreign libnvtt "?diff@nvtt@@YA?AUSurface@1@AEBU21@0M@Z";

errorString :: (e: Error) -> *u8 #foreign libnvtt "?errorString@nvtt@@YAPEBDW4Error@1@@Z";

histogram :: (img: *Surface, width: s32, height: s32) -> Surface #cpp_return_type_is_non_pod #foreign libnvtt "?histogram@nvtt@@YA?AUSurface@1@AEBU21@HH@Z";

histogram :: (img: *Surface, minRange: float, maxRange: float, width: s32, height: s32) -> Surface #cpp_return_type_is_non_pod #foreign libnvtt "?histogram@nvtt@@YA?AUSurface@1@AEBU21@MMHH@Z";

operator_assign :: (this: *Surface, img: *Surface) -> void #cpp_method #foreign libnvtt "??4Surface@nvtt@@QEAAXAEBU01@@Z";

operator_assign :: (this: *CubeSurface, img: *CubeSurface) -> void #cpp_method #foreign libnvtt "??4CubeSurface@nvtt@@QEAAXAEBU01@@Z";

rmsAlphaError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "?rmsAlphaError@nvtt@@YAMAEBUSurface@1@0@Z";

rmsError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "?rmsError@nvtt@@YAMAEBUSurface@1@0@Z";

rmsToneMappedError :: (reference: *Surface, img: *Surface, exposure: float) -> float #foreign libnvtt "?rmsToneMappedError@nvtt@@YAMAEBUSurface@1@0M@Z";

version :: () -> u32 #foreign libnvtt "?version@nvtt@@YAIXZ";

angularError :: (reference: Surface, img: Surface) -> float #no_context {
    return angularError(*reference, *img);
}

cieLabError :: (reference: Surface, img: Surface) -> float #no_context {
    return cieLabError(*reference, *img);
}

diff :: (reference: Surface, img: Surface, scale: float) -> Surface #no_context {
    return diff(*reference, *img, scale);
}

ErrorHandler_Destructor :: inline (this: *ErrorHandler, flags: Msvc_Destructor_Flags = .NONE) {
    this.vtable.Destructor(this, flags);
}

ErrorHandler_error :: inline (this: *ErrorHandler, e: Error) {
    this.vtable.error(this, e);
}

histogram :: (img: Surface, width: s32, height: s32) -> Surface #no_context {
    return histogram(*img, width, height);
}

histogram :: (img: Surface, minRange: float, maxRange: float, width: s32, height: s32) -> Surface #no_context {
    return histogram(*img, minRange, maxRange, width, height);
}

operator_assign :: (this: *Surface, img: Surface) #no_context {
    operator_assign(this, *img);
}

operator_assign :: (this: *CubeSurface, img: CubeSurface) #no_context {
    operator_assign(this, *img);
}

OutputHandler_beginImage :: inline (this: *OutputHandler, size: s32, width: s32, height: s32, depth: s32, face: s32, miplevel: s32) {
    this.vtable.beginImage(this, size, width, height, depth, face, miplevel);
}

OutputHandler_Destructor :: inline (this: *OutputHandler, flags: Msvc_Destructor_Flags = .NONE) {
    this.vtable.Destructor(this, flags);
}

OutputHandler_endImage :: inline (this: *OutputHandler) {
    this.vtable.endImage(this);
}

OutputHandler_writeData :: inline (this: *OutputHandler, data: *void, size: s32) -> bool {
    return this.vtable.writeData(this, data, size);
}

rmsAlphaError :: (reference: Surface, img: Surface) -> float #no_context {
    return rmsAlphaError(*reference, *img);
}

rmsError :: (reference: Surface, img: Surface) -> float #no_context {
    return rmsError(*reference, *img);
}

rmsToneMappedError :: (reference: Surface, img: Surface, exposure: float) -> float #no_context {
    return rmsToneMappedError(*reference, *img, exposure);
}

TaskDispatcher_Destructor :: inline (this: *TaskDispatcher, flags: Msvc_Destructor_Flags = .NONE) {
    this.vtable.Destructor(this, flags);
}

TaskDispatcher_dispatch :: inline (this: *TaskDispatcher, task: Task, _context: *void, count: s32) {
    this.vtable.dispatch(this, task, _context, count);
}

vtable :: (obj: *OutputHandler) -> *OutputHandler_VTable {
    return obj.vtable;
}

vtable :: (obj: *ErrorHandler) -> *ErrorHandler_VTable {
    return obj.vtable;
}

vtable :: (obj: *TaskDispatcher) -> *TaskDispatcher_VTable {
    return obj.vtable;
}

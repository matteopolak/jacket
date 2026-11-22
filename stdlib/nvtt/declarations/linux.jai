// Generated from docs/specs/stdlib/modules/nvtt.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

// The declarations for `OS == .LINUX`, loaded by declarations.jai.

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

AlphaMode :: enum u32 {
    None :: 0;
    Transparency :: 1;
    Premultiplied :: 2;

}

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

Compressor :: struct #type_info_procedures_are_void_pointers {
    Private :: struct {}
    m: *Private;

    Constructor :: (this: *Compressor) -> void #cpp_method #foreign libnvtt "_ZN4nvtt10CompressorC1Ev";
    Destructor_Base :: (this: *Compressor) -> void #cpp_method #foreign libnvtt "_ZN4nvtt10CompressorD2Ev";
    Destructor :: (this: *Compressor) -> void #cpp_method #foreign libnvtt "_ZN4nvtt10CompressorD1Ev";
    estimateSize :: (this: *Compressor, inputOptions: *InputOptions, compressionOptions: *CompressionOptions) -> s32 #cpp_method #foreign libnvtt "_ZNK4nvtt10Compressor12estimateSizeERKNS_12InputOptionsERKNS_18CompressionOptionsE";
    estimateSize :: (this: *Compressor, inputOptions: InputOptions, compressionOptions: CompressionOptions) -> s32 #no_context {
        return estimateSize(this, *inputOptions, *compressionOptions);
    }

    outputHeader :: (this: *Compressor, img: *Surface, mipmapCount: s32, compressionOptions: *CompressionOptions, outputOptions: *OutputOptions) -> bool #cpp_method #foreign libnvtt "_ZNK4nvtt10Compressor12outputHeaderERKNS_7SurfaceEiRKNS_18CompressionOptionsERKNS_13OutputOptionsE";
    outputHeader :: (this: *Compressor, img: Surface, mipmapCount: s32, compressionOptions: CompressionOptions, outputOptions: OutputOptions) -> bool #no_context {
        return outputHeader(this, *img, mipmapCount, *compressionOptions, *outputOptions);
    }
    compress :: (this: *Compressor, img: *Surface, face: s32, mipmap: s32, compressionOptions: *CompressionOptions, outputOptions: *OutputOptions) -> bool #cpp_method #foreign libnvtt "_ZNK4nvtt10Compressor8compressERKNS_7SurfaceEiiRKNS_18CompressionOptionsERKNS_13OutputOptionsE";
    compress :: (this: *Compressor, img: Surface, face: s32, mipmap: s32, compressionOptions: CompressionOptions, outputOptions: OutputOptions) -> bool #no_context {
        return compress(this, *img, face, mipmap, *compressionOptions, *outputOptions);
    }
    estimateSize :: (this: *Compressor, img: *Surface, mipmapCount: s32, compressionOptions: *CompressionOptions) -> s32 #cpp_method #foreign libnvtt "_ZNK4nvtt10Compressor12estimateSizeERKNS_7SurfaceEiRKNS_18CompressionOptionsE";
    estimateSize :: (this: *Compressor, img: Surface, mipmapCount: s32, compressionOptions: CompressionOptions) -> s32 #no_context {
        return estimateSize(this, *img, mipmapCount, *compressionOptions);
    }

    outputHeader :: (this: *Compressor, cube: *CubeSurface, mipmapCount: s32, compressionOptions: *CompressionOptions, outputOptions: *OutputOptions) -> bool #cpp_method #foreign libnvtt "_ZNK4nvtt10Compressor12outputHeaderERKNS_11CubeSurfaceEiRKNS_18CompressionOptionsERKNS_13OutputOptionsE";
    outputHeader :: (this: *Compressor, cube: CubeSurface, mipmapCount: s32, compressionOptions: CompressionOptions, outputOptions: OutputOptions) -> bool #no_context {
        return outputHeader(this, *cube, mipmapCount, *compressionOptions, *outputOptions);
    }
    compress :: (this: *Compressor, cube: *CubeSurface, mipmap: s32, compressionOptions: *CompressionOptions, outputOptions: *OutputOptions) -> bool #cpp_method #foreign libnvtt "_ZNK4nvtt10Compressor8compressERKNS_11CubeSurfaceEiRKNS_18CompressionOptionsERKNS_13OutputOptionsE";
    compress :: (this: *Compressor, cube: CubeSurface, mipmap: s32, compressionOptions: CompressionOptions, outputOptions: OutputOptions) -> bool #no_context {
        return compress(this, *cube, mipmap, *compressionOptions, *outputOptions);
    }
    estimateSize :: (this: *Compressor, cube: *CubeSurface, mipmapCount: s32, compressionOptions: *CompressionOptions) -> s32 #cpp_method #foreign libnvtt "_ZNK4nvtt10Compressor12estimateSizeERKNS_11CubeSurfaceEiRKNS_18CompressionOptionsE";
    estimateSize :: (this: *Compressor, cube: CubeSurface, mipmapCount: s32, compressionOptions: CompressionOptions) -> s32 #no_context {
        return estimateSize(this, *cube, mipmapCount, *compressionOptions);
    }

    outputHeader :: (this: *Compressor, type: TextureType, w: s32, h: s32, d: s32, arraySize: s32, mipmapCount: s32, isNormalMap: bool, compressionOptions: *CompressionOptions, outputOptions: *OutputOptions) -> bool #cpp_method #foreign libnvtt "_ZNK4nvtt10Compressor12outputHeaderENS_11TextureTypeEiiiiibRKNS_18CompressionOptionsERKNS_13OutputOptionsE";
    outputHeader :: (this: *Compressor, type: TextureType, w: s32, h: s32, d: s32, arraySize: s32, mipmapCount: s32, isNormalMap: bool, compressionOptions: CompressionOptions, outputOptions: OutputOptions) -> bool #no_context {
        return outputHeader(this, type, w, h, d, arraySize, mipmapCount, isNormalMap, *compressionOptions, *outputOptions);
    }
    compress :: (this: *Compressor, w: s32, h: s32, d: s32, face: s32, mipmap: s32, rgba: *float, compressionOptions: *CompressionOptions, outputOptions: *OutputOptions) -> bool #cpp_method #foreign libnvtt "_ZNK4nvtt10Compressor8compressEiiiiiPKfRKNS_18CompressionOptionsERKNS_13OutputOptionsE";
    compress :: (this: *Compressor, w: s32, h: s32, d: s32, face: s32, mipmap: s32, rgba: *float, compressionOptions: CompressionOptions, outputOptions: OutputOptions) -> bool #no_context {
        return compress(this, w, h, d, face, mipmap, rgba, *compressionOptions, *outputOptions);
    }
    estimateSize :: (this: *Compressor, w: s32, h: s32, d: s32, mipmapCount: s32, compressionOptions: *CompressionOptions) -> s32 #cpp_method #foreign libnvtt "_ZNK4nvtt10Compressor12estimateSizeEiiiiRKNS_18CompressionOptionsE";
    estimateSize :: (this: *Compressor, w: s32, h: s32, d: s32, mipmapCount: s32, compressionOptions: CompressionOptions) -> s32 #no_context {
        return estimateSize(this, w, h, d, mipmapCount, *compressionOptions);
    }

    enableCudaAcceleration :: (this: *Compressor, enable: bool) -> void #cpp_method #foreign libnvtt "_ZN4nvtt10Compressor22enableCudaAccelerationEb";
    isCudaAccelerationEnabled :: (this: *Compressor) -> bool #cpp_method #foreign libnvtt "_ZNK4nvtt10Compressor25isCudaAccelerationEnabledEv";
    process :: (this: *Compressor, inputOptions: *InputOptions, compressionOptions: *CompressionOptions, outputOptions: *OutputOptions) -> bool #cpp_method #foreign libnvtt "_ZNK4nvtt10Compressor7processERKNS_12InputOptionsERKNS_18CompressionOptionsERKNS_13OutputOptionsE";
    setTaskDispatcher :: (this: *Compressor, disp: *TaskDispatcher) -> void #cpp_method #foreign libnvtt "_ZN4nvtt10Compressor17setTaskDispatcherEPNS_14TaskDispatcherE";
}

Container :: enum u32 {
    DDS :: 0;
    DDS10 :: 1;
    KTX :: 2;

}

CubeLayout :: enum u32 {
    VerticalCross :: 0;
    HorizontalCross :: 1;
    Column :: 2;
    Row :: 3;
    LatitudeLongitude :: 4;

}

CubeSurface :: struct #type_info_procedures_are_void_pointers {
    Constructor :: (this: *CubeSurface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt11CubeSurfaceC1Ev";
    CopyConstructor :: (this: *CubeSurface, img: *CubeSurface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt11CubeSurfaceC1ERKS0_";
    CopyConstructor :: (this: *CubeSurface, img: CubeSurface) #no_context {
        CopyConstructor(this, *img);
    }
    Destructor_Base :: (this: *CubeSurface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt11CubeSurfaceD2Ev";
    Destructor :: (this: *CubeSurface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt11CubeSurfaceD1Ev";
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

    countMipmaps :: (this: *CubeSurface) -> s32 #cpp_method #foreign libnvtt "_ZNK4nvtt11CubeSurface12countMipmapsEv";
    edgeLength :: (this: *CubeSurface) -> s32 #cpp_method #foreign libnvtt "_ZNK4nvtt11CubeSurface10edgeLengthEv";
    face :: (this: *CubeSurface, face: s32) -> *Surface #cpp_method #foreign libnvtt "_ZN4nvtt11CubeSurface4faceEi";
    face :: (this: *CubeSurface, face: s32) -> *Surface #cpp_method #foreign libnvtt "_ZNK4nvtt11CubeSurface4faceEi";
    fold :: (this: *CubeSurface, img: *Surface, layout: CubeLayout) -> void #cpp_method #foreign libnvtt "_ZN4nvtt11CubeSurface4foldERKNS_7SurfaceENS_10CubeLayoutE";
    isNull :: (this: *CubeSurface) -> bool #cpp_method #foreign libnvtt "_ZNK4nvtt11CubeSurface6isNullEv";
    load :: (this: *CubeSurface, fileName: *u8, mipmap: s32) -> bool #cpp_method #foreign libnvtt "_ZN4nvtt11CubeSurface4loadEPKci";
    save :: (this: *CubeSurface, fileName: *u8) -> bool #cpp_method #foreign libnvtt "_ZNK4nvtt11CubeSurface4saveEPKc";
}

Decoder :: enum u32 {
    D3D10 :: 0;
    D3D9 :: 1;
    NV5x :: 2;

}

EdgeFixup :: enum u32 {
    None :: 0;
    Stretch :: 1;
    Warp :: 2;
    Average :: 3;

}

Error :: enum u32 {
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

    Destructor_Base :: (this: *ErrorHandler) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12ErrorHandlerD2Ev";

    virtual_Destructor :: (this: *ErrorHandler) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12ErrorHandlerD1Ev";
    virtual_Destructor_Deleting :: (this: *ErrorHandler) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12ErrorHandlerD0Ev";

}

ErrorHandler_VTable :: struct #type_info_none {
    Destructor: #type (this: *ErrorHandler) -> void #cpp_method;
    Destructor_Deleting: #type (this: *ErrorHandler) -> void #cpp_method;

    error: #type (this: *ErrorHandler, e: Error) -> void #cpp_method;

}

Format :: enum u32 {
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

InputFormat :: enum u32 {
    BGRA_8UB :: 0;
    RGBA_16F :: 1;
    RGBA_32F :: 2;
    R_32F :: 3;

}

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

MipmapFilter :: enum u32 {
    Box :: 0;
    Triangle :: 1;
    Kaiser :: 2;

}

NormalTransform :: enum u32 {
    Orthographic :: 0;
    Stereographic :: 1;
    Paraboloid :: 2;
    Quartic :: 3;

}

OutputHandler :: struct {
    vtable: *OutputHandler_VTable;

    Destructor_Base :: (this: *OutputHandler) -> void #cpp_method #foreign libnvtt "_ZN4nvtt13OutputHandlerD2Ev";

    virtual_Destructor :: (this: *OutputHandler) -> void #cpp_method #foreign libnvtt "_ZN4nvtt13OutputHandlerD1Ev";
    virtual_Destructor_Deleting :: (this: *OutputHandler) -> void #cpp_method #foreign libnvtt "_ZN4nvtt13OutputHandlerD0Ev";

}

OutputHandler_VTable :: struct #type_info_none {
    Destructor: #type (this: *OutputHandler) -> void #cpp_method;
    Destructor_Deleting: #type (this: *OutputHandler) -> void #cpp_method;

    beginImage: #type (this: *OutputHandler, size: s32, width: s32, height: s32, depth: s32, face: s32, miplevel: s32) -> void #cpp_method;

    writeData: #type (this: *OutputHandler, data: *void, size: s32) -> bool #cpp_method;

    endImage: #type (this: *OutputHandler) -> void #cpp_method;

}

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

PixelType :: enum u32 {
    UnsignedNorm :: 0;
    SignedNorm :: 1;
    UnsignedInt :: 2;
    SignedInt :: 3;
    Float :: 4;
    UnsignedFloat :: 5;
    SharedExp :: 6;

}

Quality :: enum u32 {
    Fastest :: 0;
    Normal :: 1;
    Production :: 2;
    Highest :: 3;

}

ResizeFilter :: enum u32 {
    Box :: 0;
    Triangle :: 1;
    Kaiser :: 2;
    Mitchell :: 3;

}

RoundMode :: enum u32 {
    None :: 0;
    ToNextPowerOfTwo :: 1;
    ToNearestPowerOfTwo :: 2;
    ToPreviousPowerOfTwo :: 3;
    ToNextMultipleOfFour :: 4;
    ToNearestMultipleOfFour :: 5;
    ToPreviousMultipleOfFour :: 6;

}

ShapeRestriction :: enum u32 {
    None :: 0;
    Square :: 1;

}

Surface :: struct #type_info_none {
    Constructor :: (this: *Surface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7SurfaceC1Ev";
    CopyConstructor :: (this: *Surface, img: *Surface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7SurfaceC1ERKS0_";
    CopyConstructor :: (this: *Surface, img: Surface) #no_context {
        CopyConstructor(this, *img);
    }
    Destructor_Base :: (this: *Surface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7SurfaceD2Ev";
    Destructor :: (this: *Surface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7SurfaceD1Ev";
    copyChannel :: (this: *Surface, srcImage: *Surface, srcChannel: s32, dstChannel: s32) -> bool #cpp_method #foreign libnvtt "_ZN4nvtt7Surface11copyChannelERKS0_ii";
    copyChannel :: (this: *Surface, srcImage: Surface, srcChannel: s32, dstChannel: s32) -> bool #no_context {
        return copyChannel(this, *srcImage, srcChannel, dstChannel);
    }

    addChannel :: (this: *Surface, img: *Surface, srcChannel: s32, dstChannel: s32, scale: float) -> bool #cpp_method #foreign libnvtt "_ZN4nvtt7Surface10addChannelERKS0_iif";
    addChannel :: (this: *Surface, img: Surface, srcChannel: s32, dstChannel: s32, scale: float) -> bool #no_context {
        return addChannel(this, *img, srcChannel, dstChannel, scale);
    }

    copy :: (this: *Surface, src: *Surface, xsrc: s32, ysrc: s32, zsrc: s32, xsize: s32, ysize: s32, zsize: s32, xdst: s32, ydst: s32, zdst: s32) -> bool #cpp_method #foreign libnvtt "_ZN4nvtt7Surface4copyERKS0_iiiiiiiii";
    copy :: (this: *Surface, src: Surface, xsrc: s32, ysrc: s32, zsrc: s32, xsize: s32, ysize: s32, zsize: s32, xdst: s32, ydst: s32, zdst: s32) -> bool #no_context {
        return copy(this, *src, xsrc, ysrc, zsrc, xsize, ysize, zsize, xdst, ydst, zdst);
    }

    detach :: (this: *Surface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface6detachEv";

    Private :: struct {}
    m: *Private;

    abs :: (this: *Surface, channel: s32) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface3absEi";
    alphaMode :: (this: *Surface) -> AlphaMode #cpp_method #foreign libnvtt "_ZNK4nvtt7Surface9alphaModeEv";
    alphaTestCoverage :: (this: *Surface, alphaRef: float = 0.5, alpha_channel: s32 = 3) -> float #cpp_method #foreign libnvtt "_ZNK4nvtt7Surface17alphaTestCoverageEfi";
    average :: (this: *Surface, channel: s32, alpha_channel: s32 = -1, gamma: float = 2.2) -> float #cpp_method #foreign libnvtt "_ZNK4nvtt7Surface7averageEiif";
    binarize :: (this: *Surface, channel: s32, threshold: float, dither: bool) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface8binarizeEifb";
    blend :: (this: *Surface, r: float, g: float, b: float, a: float, t: float) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface5blendEfffff";
    blockScaleCoCg :: (this: *Surface, bits: s32 = 5, threshold: float = 0.0) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface14blockScaleCoCgEif";
    buildNextMipmap :: (this: *Surface, filter: MipmapFilter, min_size: s32 = 1) -> bool #cpp_method #foreign libnvtt "_ZN4nvtt7Surface15buildNextMipmapENS_12MipmapFilterEi";
    buildNextMipmap :: (this: *Surface, filter: MipmapFilter, filterWidth: float, params: *float = null, min_size: s32 = 1) -> bool #cpp_method #foreign libnvtt "_ZN4nvtt7Surface15buildNextMipmapENS_12MipmapFilterEfPKfi";
    buildNextMipmapSolidColor :: (this: *Surface, color_components: *float) -> bool #cpp_method #foreign libnvtt "_ZN4nvtt7Surface25buildNextMipmapSolidColorEPKf";
    canMakeNextMipmap :: (this: *Surface, min_size: s32 = 1) -> bool #cpp_method #foreign libnvtt "_ZN4nvtt7Surface17canMakeNextMipmapEi";
    canvasSize :: (this: *Surface, w: s32, h: s32, d: s32) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface10canvasSizeEiii";
    channel :: (this: *Surface, i: s32) -> *float #cpp_method #foreign libnvtt "_ZNK4nvtt7Surface7channelEi";
    clamp :: (this: *Surface, channel: s32, low: float = 0.0, high: float = 1.0) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface5clampEiff";
    convolve :: (this: *Surface, channel: s32, kernelSize: s32, kernelData: *float) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface8convolveEiiPf";
    copyChannel :: (this: *Surface, srcImage: *Surface, srcChannel: s32) -> bool #cpp_method #foreign libnvtt "_ZN4nvtt7Surface11copyChannelERKS0_i";
    countMipmaps :: (this: *Surface) -> s32 #cpp_method #foreign libnvtt "_ZNK4nvtt7Surface12countMipmapsEv";
    countMipmaps :: (this: *Surface, min_size: s32) -> s32 #cpp_method #foreign libnvtt "_ZNK4nvtt7Surface12countMipmapsEi";
    createCleanMap :: (this: *Surface) -> Surface #cpp_method #cpp_return_type_is_non_pod #foreign libnvtt "_ZNK4nvtt7Surface14createCleanMapEv";
    createSubImage :: (this: *Surface, x0: s32, x1: s32, y0: s32, y1: s32, z0: s32, z1: s32) -> Surface #cpp_method #cpp_return_type_is_non_pod #foreign libnvtt "_ZNK4nvtt7Surface14createSubImageEiiiiii";
    createToksvigMap :: (this: *Surface, power: float) -> Surface #cpp_method #cpp_return_type_is_non_pod #foreign libnvtt "_ZNK4nvtt7Surface16createToksvigMapEf";
    data :: (this: *Surface) -> *float #cpp_method #foreign libnvtt "_ZNK4nvtt7Surface4dataEv";
    depth :: (this: *Surface) -> s32 #cpp_method #foreign libnvtt "_ZNK4nvtt7Surface5depthEv";
    expandNormals :: (this: *Surface, scale: float = 2.0, bias: float = -1.0) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface13expandNormalsEff";
    fill :: (this: *Surface, r: float, g: float, b: float, a: float) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface4fillEffff";
    flipX :: (this: *Surface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface5flipXEv";
    flipY :: (this: *Surface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface5flipYEv";
    flipZ :: (this: *Surface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface5flipZEv";
    fromLogScale :: (this: *Surface, channel: s32, base: float) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface12fromLogScaleEif";
    fromLUVW :: (this: *Surface, range: float = 1.0) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface8fromLUVWEf";
    fromRGBE :: (this: *Surface, mantissaBits: s32, exponentBits: s32) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface8fromRGBEEii";
    fromRGBM :: (this: *Surface, range: float = 1.0, threshold: float = 0.25) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface8fromRGBMEff";
    fromYCoCg :: (this: *Surface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface9fromYCoCgEv";
    height :: (this: *Surface) -> s32 #cpp_method #foreign libnvtt "_ZNK4nvtt7Surface6heightEv";
    histogram :: (this: *Surface, channel: s32, rangeMin: float, rangeMax: float, binCount: s32, binPtr: *s32) -> void #cpp_method #foreign libnvtt "_ZNK4nvtt7Surface9histogramEiffiPi";
    isNormalMap :: (this: *Surface) -> bool #cpp_method #foreign libnvtt "_ZNK4nvtt7Surface11isNormalMapEv";
    isNull :: (this: *Surface) -> bool #cpp_method #foreign libnvtt "_ZNK4nvtt7Surface6isNullEv";
    load :: (this: *Surface, fileName: *u8, hasAlpha: *bool = null) -> bool #cpp_method #foreign libnvtt "_ZN4nvtt7Surface4loadEPKcPb";
    normalizeNormalMap :: (this: *Surface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface18normalizeNormalMapEv";
    packNormals :: (this: *Surface, scale: float = 0.5, bias: float = 0.5) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface11packNormalsEff";
    premultiplyAlpha :: (this: *Surface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface16premultiplyAlphaEv";
    quantize :: (this: *Surface, channel: s32, bits: s32, exactEndPoints: bool, dither: bool) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface8quantizeEiibb";
    range :: (this: *Surface, channel: s32, rangeMin: *float, rangeMax: *float, alpha_channel: s32 = -1, alpha_ref: float = 0.0) -> void #cpp_method #foreign libnvtt "_ZNK4nvtt7Surface5rangeEiPfS1_if";
    reconstructNormals :: (this: *Surface, xform: NormalTransform) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface18reconstructNormalsENS_15NormalTransformE";
    resize :: (this: *Surface, w: s32, h: s32, d: s32, filter: ResizeFilter) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface6resizeEiiiNS_12ResizeFilterE";
    resize :: (this: *Surface, w: s32, h: s32, d: s32, filter: ResizeFilter, filterWidth: float, params: *float = null) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface6resizeEiiiNS_12ResizeFilterEfPKf";
    resize :: (this: *Surface, maxExtent: s32, mode: RoundMode, filter: ResizeFilter) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface6resizeEiNS_9RoundModeENS_12ResizeFilterE";
    resize :: (this: *Surface, maxExtent: s32, mode: RoundMode, filter: ResizeFilter, filterWidth: float, params: *float = null) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface6resizeEiNS_9RoundModeENS_12ResizeFilterEfPKf";
    resize_make_square :: (this: *Surface, maxExtent: s32, roundMode: RoundMode, filter: ResizeFilter) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface18resize_make_squareEiNS_9RoundModeENS_12ResizeFilterE";
    save :: (this: *Surface, fileName: *u8, hasAlpha: bool = false, hdr: bool = false) -> bool #cpp_method #foreign libnvtt "_ZNK4nvtt7Surface4saveEPKcbb";
    scaleAlphaToCoverage :: (this: *Surface, coverage: float, alphaRef: float = 0.5, alpha_channel: s32 = 3) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface20scaleAlphaToCoverageEffi";
    scaleBias :: (this: *Surface, channel: s32, scale: float, bias: float) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface9scaleBiasEiff";
    setAlphaMode :: (this: *Surface, alphaMode: AlphaMode) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface12setAlphaModeENS_9AlphaModeE";
    setAtlasBorder :: (this: *Surface, w: s32, h: s32, r: float, g: float, b: float, a: float) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface14setAtlasBorderEiiffff";
    setBorder :: (this: *Surface, r: float, g: float, b: float, a: float) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface9setBorderEffff";
    setImage :: (this: *Surface, w: s32, h: s32, d: s32) -> bool #cpp_method #foreign libnvtt "_ZN4nvtt7Surface8setImageEiii";
    setImage :: (this: *Surface, format: InputFormat, w: s32, h: s32, d: s32, data: *void) -> bool #cpp_method #foreign libnvtt "_ZN4nvtt7Surface8setImageENS_11InputFormatEiiiPKv";
    setImage :: (this: *Surface, format: InputFormat, w: s32, h: s32, d: s32, r: *void, g: *void, b: *void, a: *void) -> bool #cpp_method #foreign libnvtt "_ZN4nvtt7Surface8setImageENS_11InputFormatEiiiPKvS3_S3_S3_";
    setImage2D :: (this: *Surface, format: Format, decoder: Decoder, w: s32, h: s32, data: *void) -> bool #cpp_method #foreign libnvtt "_ZN4nvtt7Surface10setImage2DENS_6FormatENS_7DecoderEiiPKv";
    setNormalMap :: (this: *Surface, isNormalMap: bool) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface12setNormalMapEb";
    setWrapMode :: (this: *Surface, mode: WrapMode) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface11setWrapModeENS_8WrapModeE";
    swizzle :: (this: *Surface, r: s32, g: s32, b: s32, a: s32) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface7swizzleEiiii";
    toCleanNormalMap :: (this: *Surface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface16toCleanNormalMapEv";
    toGamma :: (this: *Surface, gamma: float) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface7toGammaEf";
    toGamma :: (this: *Surface, channel: s32, gamma: float) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface7toGammaEif";
    toGreyScale :: (this: *Surface, redScale: float, greenScale: float, blueScale: float, alphaScale: float) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface11toGreyScaleEffff";
    toLinear :: (this: *Surface, gamma: float) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface8toLinearEf";
    toLinear :: (this: *Surface, channel: s32, gamma: float) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface8toLinearEif";
    toLinearFromSrgb :: (this: *Surface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface16toLinearFromSrgbEv";
    toLM :: (this: *Surface, range: float = 1.0, threshold: float = 0.0) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface4toLMEff";
    toLogScale :: (this: *Surface, channel: s32, base: float) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface10toLogScaleEif";
    toLUVW :: (this: *Surface, range: float = 1.0) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface6toLUVWEf";
    toneMap :: (this: *Surface, tm: ToneMapper, parameters: *float) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface7toneMapENS_10ToneMapperEPf";
    toNormalMap :: (this: *Surface, sm: float, medium: float, big: float, large: float) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface11toNormalMapEffff";
    toRGBE :: (this: *Surface, mantissaBits: s32, exponentBits: s32) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface6toRGBEEii";
    toRGBM :: (this: *Surface, range: float = 1.0, threshold: float = 0.25) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface6toRGBMEff";
    toSrgb :: (this: *Surface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface6toSrgbEv";
    toXenonSrgb :: (this: *Surface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface11toXenonSrgbEv";
    toYCoCg :: (this: *Surface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface7toYCoCgEv";
    transform :: (this: *Surface, w0: *float, w1: *float, w2: *float, w3: *float, offset: *float) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface9transformEPKfS2_S2_S2_S2_";
    transformNormals :: (this: *Surface, xform: NormalTransform) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7Surface16transformNormalsENS_15NormalTransformE";
    type :: (this: *Surface) -> TextureType #cpp_method #foreign libnvtt "_ZNK4nvtt7Surface4typeEv";
    width :: (this: *Surface) -> s32 #cpp_method #foreign libnvtt "_ZNK4nvtt7Surface5widthEv";
    wrapMode :: (this: *Surface) -> WrapMode #cpp_method #foreign libnvtt "_ZNK4nvtt7Surface8wrapModeEv";
}

TaskDispatcher :: struct {
    vtable: *TaskDispatcher_VTable;

    Destructor_Base :: (this: *TaskDispatcher) -> void #cpp_method #foreign libnvtt "_ZN4nvtt14TaskDispatcherD2Ev";

    virtual_Destructor :: (this: *TaskDispatcher) -> void #cpp_method #foreign libnvtt "_ZN4nvtt14TaskDispatcherD1Ev";
    virtual_Destructor_Deleting :: (this: *TaskDispatcher) -> void #cpp_method #foreign libnvtt "_ZN4nvtt14TaskDispatcherD0Ev";

}

TaskDispatcher_VTable :: struct #type_info_none {
    Destructor: #type (this: *TaskDispatcher) -> void #cpp_method;
    Destructor_Deleting: #type (this: *TaskDispatcher) -> void #cpp_method;

    dispatch: #type (this: *TaskDispatcher, task: Task, _context: *void, count: s32) -> void #cpp_method;

}

TextureType :: enum u32 {
    _2D :: 0;
    Cube :: 1;
    _3D :: 2;
    Array :: 3;

}

ToneMapper :: enum u32 {
    Linear :: 0;
    Reindhart :: 1;
    Halo :: 2;
    Lightmap :: 3;

}

WrapMode :: enum u32 {
    Clamp :: 0;
    Repeat :: 1;
    Mirror :: 2;

}

Task :: #type (_context: *void, id: s32) -> void #c_call;

WarpFunction :: #type (x: *float, y: *float, d: *float) -> void #c_call;

Context :: Compressor;

NVTT_VERSION :: 20100;

angularError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "_ZN4nvtt12angularErrorERKNS_7SurfaceES2_";

cieLabError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "_ZN4nvtt11cieLabErrorERKNS_7SurfaceES2_";

diff :: (reference: *Surface, img: *Surface, scale: float) -> Surface #cpp_return_type_is_non_pod #foreign libnvtt "_ZN4nvtt4diffERKNS_7SurfaceES2_f";

errorString :: (e: Error) -> *u8 #foreign libnvtt "_ZN4nvtt11errorStringENS_5ErrorE";

histogram :: (img: *Surface, width: s32, height: s32) -> Surface #cpp_return_type_is_non_pod #foreign libnvtt "_ZN4nvtt9histogramERKNS_7SurfaceEii";

histogram :: (img: *Surface, minRange: float, maxRange: float, width: s32, height: s32) -> Surface #cpp_return_type_is_non_pod #foreign libnvtt "_ZN4nvtt9histogramERKNS_7SurfaceEffii";

operator_assign :: (this: *Surface, img: *Surface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7SurfaceaSERKS0_";

operator_assign :: (this: *CubeSurface, img: *CubeSurface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt11CubeSurfaceaSERKS0_";

rmsAlphaError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "_ZN4nvtt13rmsAlphaErrorERKNS_7SurfaceES2_";

rmsError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "_ZN4nvtt8rmsErrorERKNS_7SurfaceES2_";

rmsToneMappedError :: (reference: *Surface, img: *Surface, exposure: float) -> float #foreign libnvtt "_ZN4nvtt18rmsToneMappedErrorERKNS_7SurfaceES2_f";

version :: () -> u32 #foreign libnvtt "_ZN4nvtt7versionEv";

angularError :: (reference: Surface, img: Surface) -> float #no_context {
    return angularError(*reference, *img);
}

cieLabError :: (reference: Surface, img: Surface) -> float #no_context {
    return cieLabError(*reference, *img);
}

diff :: (reference: Surface, img: Surface, scale: float) -> Surface #no_context {
    return diff(*reference, *img, scale);
}

ErrorHandler_Destructor :: inline (this: *ErrorHandler) {
    this.vtable.Destructor(this);
}

ErrorHandler_Destructor_Deleting :: inline (this: *ErrorHandler) {
    this.vtable.Destructor_Deleting(this);
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

OutputHandler_Destructor :: inline (this: *OutputHandler) {
    this.vtable.Destructor(this);
}

OutputHandler_Destructor_Deleting :: inline (this: *OutputHandler) {
    this.vtable.Destructor_Deleting(this);
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

TaskDispatcher_Destructor :: inline (this: *TaskDispatcher) {
    this.vtable.Destructor(this);
}

TaskDispatcher_Destructor_Deleting :: inline (this: *TaskDispatcher) {
    this.vtable.Destructor_Deleting(this);
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

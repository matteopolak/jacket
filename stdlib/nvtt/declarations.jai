// Generated from docs/specs/stdlib/modules/nvtt.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

#scope_file
#if OS == .WINDOWS {
    #import "Windows";
}

#if OS == .MACOS {
    #library,system,link_always "c++";
} else #if OS == .LINUX {
    #library,system,link_always "stdc++";
}

libnvtt :: #library,system "nvtt";
#scope_export

// Types and layouts

#if OS == .WINDOWS {
    AlphaMode :: enum s32 {
        None :: 0;
        Transparency :: 1;
        Premultiplied :: 2;

    }

}

#if OS == .LINUX {
    AlphaMode :: enum u32 {
        None :: 0;
        Transparency :: 1;
        Premultiplied :: 2;

    }

}

#if OS == .MACOS {
    AlphaMode :: enum u32 {
        None :: 0;
        Transparency :: 1;
        Premultiplied :: 2;

    }

}

#if OS == .WINDOWS {
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

}

#if OS == .LINUX {
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

}

#if OS == .MACOS {
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

}

#if OS == .WINDOWS {
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

}

#if OS == .LINUX {
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

}

#if OS == .MACOS {
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

}

#if OS == .WINDOWS {
    Container :: enum s32 {
        DDS :: 0;
        DDS10 :: 1;
        KTX :: 2;

    }

}

#if OS == .LINUX {
    Container :: enum u32 {
        DDS :: 0;
        DDS10 :: 1;
        KTX :: 2;

    }

}

#if OS == .MACOS {
    Container :: enum u32 {
        DDS :: 0;
        DDS10 :: 1;
        KTX :: 2;

    }

}

#if OS == .WINDOWS {
    CubeLayout :: enum s32 {
        VerticalCross :: 0;
        HorizontalCross :: 1;
        Column :: 2;
        Row :: 3;
        LatitudeLongitude :: 4;

    }

}

#if OS == .LINUX {
    CubeLayout :: enum u32 {
        VerticalCross :: 0;
        HorizontalCross :: 1;
        Column :: 2;
        Row :: 3;
        LatitudeLongitude :: 4;

    }

}

#if OS == .MACOS {
    CubeLayout :: enum u32 {
        VerticalCross :: 0;
        HorizontalCross :: 1;
        Column :: 2;
        Row :: 3;
        LatitudeLongitude :: 4;

    }

}

#if OS == .WINDOWS {
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

}

#if OS == .LINUX {
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

}

#if OS == .MACOS {
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

}

#if OS == .WINDOWS {
    Decoder :: enum s32 {
        D3D10 :: 0;
        D3D9 :: 1;
        NV5x :: 2;

    }

}

#if OS == .LINUX {
    Decoder :: enum u32 {
        D3D10 :: 0;
        D3D9 :: 1;
        NV5x :: 2;

    }

}

#if OS == .MACOS {
    Decoder :: enum u32 {
        D3D10 :: 0;
        D3D9 :: 1;
        NV5x :: 2;

    }

}

#if OS == .WINDOWS {
    EdgeFixup :: enum s32 {
        None :: 0;
        Stretch :: 1;
        Warp :: 2;
        Average :: 3;

    }

}

#if OS == .LINUX {
    EdgeFixup :: enum u32 {
        None :: 0;
        Stretch :: 1;
        Warp :: 2;
        Average :: 3;

    }

}

#if OS == .MACOS {
    EdgeFixup :: enum u32 {
        None :: 0;
        Stretch :: 1;
        Warp :: 2;
        Average :: 3;

    }

}

#if OS == .WINDOWS {
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

}

#if OS == .LINUX {
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

}

#if OS == .MACOS {
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

}

#if OS == .WINDOWS {
    ErrorHandler :: struct {
        vtable: *ErrorHandler_VTable;

    }

}

#if OS == .LINUX {
    ErrorHandler :: struct {
        vtable: *ErrorHandler_VTable;

        Destructor_Base :: (this: *ErrorHandler) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12ErrorHandlerD2Ev";

        virtual_Destructor :: (this: *ErrorHandler) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12ErrorHandlerD1Ev";
        virtual_Destructor_Deleting :: (this: *ErrorHandler) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12ErrorHandlerD0Ev";

    }

}

#if OS == .MACOS {
    ErrorHandler :: struct {
        vtable: *ErrorHandler_VTable;

        Destructor_Base :: (this: *ErrorHandler) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12ErrorHandlerD2Ev";

        virtual_Destructor :: (this: *ErrorHandler) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12ErrorHandlerD1Ev";
        virtual_Destructor_Deleting :: (this: *ErrorHandler) -> void #cpp_method #foreign libnvtt "_ZN4nvtt12ErrorHandlerD0Ev";

    }

}

#if OS == .WINDOWS {
    ErrorHandler_VTable :: struct #type_info_none {
        Destructor: #type (this: *ErrorHandler, flags: Msvc_Destructor_Flags = .NONE) -> void #cpp_method;

        error: #type (this: *ErrorHandler, e: Error) -> void #cpp_method;

    }

}

#if OS == .LINUX {
    ErrorHandler_VTable :: struct #type_info_none {
        Destructor: #type (this: *ErrorHandler) -> void #cpp_method;
        Destructor_Deleting: #type (this: *ErrorHandler) -> void #cpp_method;

        error: #type (this: *ErrorHandler, e: Error) -> void #cpp_method;

    }

}

#if OS == .MACOS {
    ErrorHandler_VTable :: struct #type_info_none {
        Destructor: #type (this: *ErrorHandler) -> void #cpp_method;
        Destructor_Deleting: #type (this: *ErrorHandler) -> void #cpp_method;

        error: #type (this: *ErrorHandler, e: Error) -> void #cpp_method;

    }

}

#if OS == .WINDOWS {
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

}

#if OS == .LINUX {
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

}

#if OS == .MACOS {
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

}

#if OS == .WINDOWS {
    InputFormat :: enum s32 {
        BGRA_8UB :: 0;
        RGBA_16F :: 1;
        RGBA_32F :: 2;
        R_32F :: 3;

    }

}

#if OS == .LINUX {
    InputFormat :: enum u32 {
        BGRA_8UB :: 0;
        RGBA_16F :: 1;
        RGBA_32F :: 2;
        R_32F :: 3;

    }

}

#if OS == .MACOS {
    InputFormat :: enum u32 {
        BGRA_8UB :: 0;
        RGBA_16F :: 1;
        RGBA_32F :: 2;
        R_32F :: 3;

    }

}

#if OS == .WINDOWS {
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

}

#if OS == .LINUX {
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

}

#if OS == .MACOS {
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

}

#if OS == .WINDOWS {
    MipmapFilter :: enum s32 {
        Box :: 0;
        Triangle :: 1;
        Kaiser :: 2;

    }

}

#if OS == .LINUX {
    MipmapFilter :: enum u32 {
        Box :: 0;
        Triangle :: 1;
        Kaiser :: 2;

    }

}

#if OS == .MACOS {
    MipmapFilter :: enum u32 {
        Box :: 0;
        Triangle :: 1;
        Kaiser :: 2;

    }

}

#if OS == .WINDOWS {
    NormalTransform :: enum s32 {
        Orthographic :: 0;
        Stereographic :: 1;
        Paraboloid :: 2;
        Quartic :: 3;

    }

}

#if OS == .LINUX {
    NormalTransform :: enum u32 {
        Orthographic :: 0;
        Stereographic :: 1;
        Paraboloid :: 2;
        Quartic :: 3;

    }

}

#if OS == .MACOS {
    NormalTransform :: enum u32 {
        Orthographic :: 0;
        Stereographic :: 1;
        Paraboloid :: 2;
        Quartic :: 3;

    }

}

#if OS == .WINDOWS {
    OutputHandler :: struct {
        vtable: *OutputHandler_VTable;

    }

}

#if OS == .LINUX {
    OutputHandler :: struct {
        vtable: *OutputHandler_VTable;

        Destructor_Base :: (this: *OutputHandler) -> void #cpp_method #foreign libnvtt "_ZN4nvtt13OutputHandlerD2Ev";

        virtual_Destructor :: (this: *OutputHandler) -> void #cpp_method #foreign libnvtt "_ZN4nvtt13OutputHandlerD1Ev";
        virtual_Destructor_Deleting :: (this: *OutputHandler) -> void #cpp_method #foreign libnvtt "_ZN4nvtt13OutputHandlerD0Ev";

    }

}

#if OS == .MACOS {
    OutputHandler :: struct {
        vtable: *OutputHandler_VTable;

        Destructor_Base :: (this: *OutputHandler) -> void #cpp_method #foreign libnvtt "_ZN4nvtt13OutputHandlerD2Ev";

        virtual_Destructor :: (this: *OutputHandler) -> void #cpp_method #foreign libnvtt "_ZN4nvtt13OutputHandlerD1Ev";
        virtual_Destructor_Deleting :: (this: *OutputHandler) -> void #cpp_method #foreign libnvtt "_ZN4nvtt13OutputHandlerD0Ev";

    }

}

#if OS == .WINDOWS {
    OutputHandler_VTable :: struct #type_info_none {
        Destructor: #type (this: *OutputHandler, flags: Msvc_Destructor_Flags = .NONE) -> void #cpp_method;

        beginImage: #type (this: *OutputHandler, size: s32, width: s32, height: s32, depth: s32, face: s32, miplevel: s32) -> void #cpp_method;

        writeData: #type (this: *OutputHandler, data: *void, size: s32) -> bool #cpp_method;

        endImage: #type (this: *OutputHandler) -> void #cpp_method;

    }

}

#if OS == .LINUX {
    OutputHandler_VTable :: struct #type_info_none {
        Destructor: #type (this: *OutputHandler) -> void #cpp_method;
        Destructor_Deleting: #type (this: *OutputHandler) -> void #cpp_method;

        beginImage: #type (this: *OutputHandler, size: s32, width: s32, height: s32, depth: s32, face: s32, miplevel: s32) -> void #cpp_method;

        writeData: #type (this: *OutputHandler, data: *void, size: s32) -> bool #cpp_method;

        endImage: #type (this: *OutputHandler) -> void #cpp_method;

    }

}

#if OS == .MACOS {
    OutputHandler_VTable :: struct #type_info_none {
        Destructor: #type (this: *OutputHandler) -> void #cpp_method;
        Destructor_Deleting: #type (this: *OutputHandler) -> void #cpp_method;

        beginImage: #type (this: *OutputHandler, size: s32, width: s32, height: s32, depth: s32, face: s32, miplevel: s32) -> void #cpp_method;

        writeData: #type (this: *OutputHandler, data: *void, size: s32) -> bool #cpp_method;

        endImage: #type (this: *OutputHandler) -> void #cpp_method;

    }

}

#if OS == .WINDOWS {
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

}

#if OS == .LINUX {
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

}

#if OS == .MACOS {
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

}

#if OS == .WINDOWS {
    PixelType :: enum s32 {
        UnsignedNorm :: 0;
        SignedNorm :: 1;
        UnsignedInt :: 2;
        SignedInt :: 3;
        Float :: 4;
        UnsignedFloat :: 5;
        SharedExp :: 6;

    }

}

#if OS == .LINUX {
    PixelType :: enum u32 {
        UnsignedNorm :: 0;
        SignedNorm :: 1;
        UnsignedInt :: 2;
        SignedInt :: 3;
        Float :: 4;
        UnsignedFloat :: 5;
        SharedExp :: 6;

    }

}

#if OS == .MACOS {
    PixelType :: enum u32 {
        UnsignedNorm :: 0;
        SignedNorm :: 1;
        UnsignedInt :: 2;
        SignedInt :: 3;
        Float :: 4;
        UnsignedFloat :: 5;
        SharedExp :: 6;

    }

}

#if OS == .WINDOWS {
    Quality :: enum s32 {
        Fastest :: 0;
        Normal :: 1;
        Production :: 2;
        Highest :: 3;

    }

}

#if OS == .LINUX {
    Quality :: enum u32 {
        Fastest :: 0;
        Normal :: 1;
        Production :: 2;
        Highest :: 3;

    }

}

#if OS == .MACOS {
    Quality :: enum u32 {
        Fastest :: 0;
        Normal :: 1;
        Production :: 2;
        Highest :: 3;

    }

}

#if OS == .WINDOWS {
    ResizeFilter :: enum s32 {
        Box :: 0;
        Triangle :: 1;
        Kaiser :: 2;
        Mitchell :: 3;

    }

}

#if OS == .LINUX {
    ResizeFilter :: enum u32 {
        Box :: 0;
        Triangle :: 1;
        Kaiser :: 2;
        Mitchell :: 3;

    }

}

#if OS == .MACOS {
    ResizeFilter :: enum u32 {
        Box :: 0;
        Triangle :: 1;
        Kaiser :: 2;
        Mitchell :: 3;

    }

}

#if OS == .WINDOWS {
    RoundMode :: enum s32 {
        None :: 0;
        ToNextPowerOfTwo :: 1;
        ToNearestPowerOfTwo :: 2;
        ToPreviousPowerOfTwo :: 3;
        ToNextMultipleOfFour :: 4;
        ToNearestMultipleOfFour :: 5;
        ToPreviousMultipleOfFour :: 6;

    }

}

#if OS == .LINUX {
    RoundMode :: enum u32 {
        None :: 0;
        ToNextPowerOfTwo :: 1;
        ToNearestPowerOfTwo :: 2;
        ToPreviousPowerOfTwo :: 3;
        ToNextMultipleOfFour :: 4;
        ToNearestMultipleOfFour :: 5;
        ToPreviousMultipleOfFour :: 6;

    }

}

#if OS == .MACOS {
    RoundMode :: enum u32 {
        None :: 0;
        ToNextPowerOfTwo :: 1;
        ToNearestPowerOfTwo :: 2;
        ToPreviousPowerOfTwo :: 3;
        ToNextMultipleOfFour :: 4;
        ToNearestMultipleOfFour :: 5;
        ToPreviousMultipleOfFour :: 6;

    }

}

#if OS == .WINDOWS {
    ShapeRestriction :: enum s32 {
        None :: 0;
        Square :: 1;

    }

}

#if OS == .LINUX {
    ShapeRestriction :: enum u32 {
        None :: 0;
        Square :: 1;

    }

}

#if OS == .MACOS {
    ShapeRestriction :: enum u32 {
        None :: 0;
        Square :: 1;

    }

}

#if OS == .WINDOWS {
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

}

#if OS == .LINUX {
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

}

#if OS == .MACOS {
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

}

#if OS == .WINDOWS {
    TaskDispatcher :: struct {
        vtable: *TaskDispatcher_VTable;

    }

}

#if OS == .LINUX {
    TaskDispatcher :: struct {
        vtable: *TaskDispatcher_VTable;

        Destructor_Base :: (this: *TaskDispatcher) -> void #cpp_method #foreign libnvtt "_ZN4nvtt14TaskDispatcherD2Ev";

        virtual_Destructor :: (this: *TaskDispatcher) -> void #cpp_method #foreign libnvtt "_ZN4nvtt14TaskDispatcherD1Ev";
        virtual_Destructor_Deleting :: (this: *TaskDispatcher) -> void #cpp_method #foreign libnvtt "_ZN4nvtt14TaskDispatcherD0Ev";

    }

}

#if OS == .MACOS {
    TaskDispatcher :: struct {
        vtable: *TaskDispatcher_VTable;

        Destructor_Base :: (this: *TaskDispatcher) -> void #cpp_method #foreign libnvtt "_ZN4nvtt14TaskDispatcherD2Ev";

        virtual_Destructor :: (this: *TaskDispatcher) -> void #cpp_method #foreign libnvtt "_ZN4nvtt14TaskDispatcherD1Ev";
        virtual_Destructor_Deleting :: (this: *TaskDispatcher) -> void #cpp_method #foreign libnvtt "_ZN4nvtt14TaskDispatcherD0Ev";

    }

}

#if OS == .WINDOWS {
    TaskDispatcher_VTable :: struct #type_info_none {
        Destructor: #type (this: *TaskDispatcher, flags: Msvc_Destructor_Flags = .NONE) -> void #cpp_method;

        dispatch: #type (this: *TaskDispatcher, task: Task, _context: *void, count: s32) -> void #cpp_method;

    }

}

#if OS == .LINUX {
    TaskDispatcher_VTable :: struct #type_info_none {
        Destructor: #type (this: *TaskDispatcher) -> void #cpp_method;
        Destructor_Deleting: #type (this: *TaskDispatcher) -> void #cpp_method;

        dispatch: #type (this: *TaskDispatcher, task: Task, _context: *void, count: s32) -> void #cpp_method;

    }

}

#if OS == .MACOS {
    TaskDispatcher_VTable :: struct #type_info_none {
        Destructor: #type (this: *TaskDispatcher) -> void #cpp_method;
        Destructor_Deleting: #type (this: *TaskDispatcher) -> void #cpp_method;

        dispatch: #type (this: *TaskDispatcher, task: Task, _context: *void, count: s32) -> void #cpp_method;

    }

}

#if OS == .WINDOWS {
    TextureType :: enum s32 {
        _2D :: 0;
        Cube :: 1;
        _3D :: 2;
        Array :: 3;

    }

}

#if OS == .LINUX {
    TextureType :: enum u32 {
        _2D :: 0;
        Cube :: 1;
        _3D :: 2;
        Array :: 3;

    }

}

#if OS == .MACOS {
    TextureType :: enum u32 {
        _2D :: 0;
        Cube :: 1;
        _3D :: 2;
        Array :: 3;

    }

}

#if OS == .WINDOWS {
    ToneMapper :: enum s32 {
        Linear :: 0;
        Reindhart :: 1;
        Halo :: 2;
        Lightmap :: 3;

    }

}

#if OS == .LINUX {
    ToneMapper :: enum u32 {
        Linear :: 0;
        Reindhart :: 1;
        Halo :: 2;
        Lightmap :: 3;

    }

}

#if OS == .MACOS {
    ToneMapper :: enum u32 {
        Linear :: 0;
        Reindhart :: 1;
        Halo :: 2;
        Lightmap :: 3;

    }

}

#if OS == .WINDOWS {
    WrapMode :: enum s32 {
        Clamp :: 0;
        Repeat :: 1;
        Mirror :: 2;

    }

}

#if OS == .LINUX {
    WrapMode :: enum u32 {
        Clamp :: 0;
        Repeat :: 1;
        Mirror :: 2;

    }

}

#if OS == .MACOS {
    WrapMode :: enum u32 {
        Clamp :: 0;
        Repeat :: 1;
        Mirror :: 2;

    }

}

// Callback and opaque types

#if OS == .WINDOWS {
    Task :: #type (_context: *void, id: s32) -> void #c_call;
}

#if OS == .LINUX {
    Task :: #type (_context: *void, id: s32) -> void #c_call;
}

#if OS == .MACOS {
    Task :: #type (_context: *void, id: s32) -> void #c_call;
}

#if OS == .WINDOWS {
    WarpFunction :: #type (x: *float, y: *float, d: *float) -> void #c_call;
}

#if OS == .LINUX {
    WarpFunction :: #type (x: *float, y: *float, d: *float) -> void #c_call;
}

#if OS == .MACOS {
    WarpFunction :: #type (x: *float, y: *float, d: *float) -> void #c_call;
}

// Aliases and constants

#if OS == .WINDOWS {
    Context :: Compressor;
}

#if OS == .LINUX {
    Context :: Compressor;
}

#if OS == .MACOS {
    Context :: Compressor;
}

#if OS == .WINDOWS {
    NVTT_VERSION :: 20100;
}

#if OS == .LINUX {
    NVTT_VERSION :: 20100;
}

#if OS == .MACOS {
    NVTT_VERSION :: 20100;
}

// Native entry points

#if OS == .WINDOWS {
    angularError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "?angularError@nvtt@@YAMAEBUSurface@1@0@Z";
}

#if OS == .LINUX {
    angularError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "_ZN4nvtt12angularErrorERKNS_7SurfaceES2_";
}

#if OS == .MACOS {
    angularError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "_ZN4nvtt12angularErrorERKNS_7SurfaceES2_";
}

#if OS == .WINDOWS {
    cieLabError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "?cieLabError@nvtt@@YAMAEBUSurface@1@0@Z";
}

#if OS == .LINUX {
    cieLabError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "_ZN4nvtt11cieLabErrorERKNS_7SurfaceES2_";
}

#if OS == .MACOS {
    cieLabError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "_ZN4nvtt11cieLabErrorERKNS_7SurfaceES2_";
}

#if OS == .WINDOWS {
    diff :: (reference: *Surface, img: *Surface, scale: float) -> Surface #cpp_return_type_is_non_pod #foreign libnvtt "?diff@nvtt@@YA?AUSurface@1@AEBU21@0M@Z";
}

#if OS == .LINUX {
    diff :: (reference: *Surface, img: *Surface, scale: float) -> Surface #cpp_return_type_is_non_pod #foreign libnvtt "_ZN4nvtt4diffERKNS_7SurfaceES2_f";
}

#if OS == .MACOS {
    diff :: (reference: *Surface, img: *Surface, scale: float) -> Surface #cpp_return_type_is_non_pod #foreign libnvtt "_ZN4nvtt4diffERKNS_7SurfaceES2_f";
}

#if OS == .WINDOWS {
    errorString :: (e: Error) -> *u8 #foreign libnvtt "?errorString@nvtt@@YAPEBDW4Error@1@@Z";
}

#if OS == .LINUX {
    errorString :: (e: Error) -> *u8 #foreign libnvtt "_ZN4nvtt11errorStringENS_5ErrorE";
}

#if OS == .MACOS {
    errorString :: (e: Error) -> *u8 #foreign libnvtt "_ZN4nvtt11errorStringENS_5ErrorE";
}

#if OS == .WINDOWS {
    histogram :: (img: *Surface, width: s32, height: s32) -> Surface #cpp_return_type_is_non_pod #foreign libnvtt "?histogram@nvtt@@YA?AUSurface@1@AEBU21@HH@Z";
}

#if OS == .WINDOWS {
    histogram :: (img: *Surface, minRange: float, maxRange: float, width: s32, height: s32) -> Surface #cpp_return_type_is_non_pod #foreign libnvtt "?histogram@nvtt@@YA?AUSurface@1@AEBU21@MMHH@Z";
}

#if OS == .LINUX {
    histogram :: (img: *Surface, width: s32, height: s32) -> Surface #cpp_return_type_is_non_pod #foreign libnvtt "_ZN4nvtt9histogramERKNS_7SurfaceEii";
}

#if OS == .LINUX {
    histogram :: (img: *Surface, minRange: float, maxRange: float, width: s32, height: s32) -> Surface #cpp_return_type_is_non_pod #foreign libnvtt "_ZN4nvtt9histogramERKNS_7SurfaceEffii";
}

#if OS == .MACOS {
    histogram :: (img: *Surface, width: s32, height: s32) -> Surface #cpp_return_type_is_non_pod #foreign libnvtt "_ZN4nvtt9histogramERKNS_7SurfaceEii";
}

#if OS == .MACOS {
    histogram :: (img: *Surface, minRange: float, maxRange: float, width: s32, height: s32) -> Surface #cpp_return_type_is_non_pod #foreign libnvtt "_ZN4nvtt9histogramERKNS_7SurfaceEffii";
}

#if OS == .WINDOWS {
    operator_assign :: (this: *Surface, img: *Surface) -> void #cpp_method #foreign libnvtt "??4Surface@nvtt@@QEAAXAEBU01@@Z";
}

#if OS == .WINDOWS {
    operator_assign :: (this: *CubeSurface, img: *CubeSurface) -> void #cpp_method #foreign libnvtt "??4CubeSurface@nvtt@@QEAAXAEBU01@@Z";
}

#if OS == .LINUX {
    operator_assign :: (this: *Surface, img: *Surface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7SurfaceaSERKS0_";
}

#if OS == .LINUX {
    operator_assign :: (this: *CubeSurface, img: *CubeSurface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt11CubeSurfaceaSERKS0_";
}

#if OS == .MACOS {
    operator_assign :: (this: *Surface, img: *Surface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt7SurfaceaSERKS0_";
}

#if OS == .MACOS {
    operator_assign :: (this: *CubeSurface, img: *CubeSurface) -> void #cpp_method #foreign libnvtt "_ZN4nvtt11CubeSurfaceaSERKS0_";
}

#if OS == .WINDOWS {
    rmsAlphaError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "?rmsAlphaError@nvtt@@YAMAEBUSurface@1@0@Z";
}

#if OS == .LINUX {
    rmsAlphaError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "_ZN4nvtt13rmsAlphaErrorERKNS_7SurfaceES2_";
}

#if OS == .MACOS {
    rmsAlphaError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "_ZN4nvtt13rmsAlphaErrorERKNS_7SurfaceES2_";
}

#if OS == .WINDOWS {
    rmsError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "?rmsError@nvtt@@YAMAEBUSurface@1@0@Z";
}

#if OS == .LINUX {
    rmsError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "_ZN4nvtt8rmsErrorERKNS_7SurfaceES2_";
}

#if OS == .MACOS {
    rmsError :: (reference: *Surface, img: *Surface) -> float #foreign libnvtt "_ZN4nvtt8rmsErrorERKNS_7SurfaceES2_";
}

#if OS == .WINDOWS {
    rmsToneMappedError :: (reference: *Surface, img: *Surface, exposure: float) -> float #foreign libnvtt "?rmsToneMappedError@nvtt@@YAMAEBUSurface@1@0M@Z";
}

#if OS == .LINUX {
    rmsToneMappedError :: (reference: *Surface, img: *Surface, exposure: float) -> float #foreign libnvtt "_ZN4nvtt18rmsToneMappedErrorERKNS_7SurfaceES2_f";
}

#if OS == .MACOS {
    rmsToneMappedError :: (reference: *Surface, img: *Surface, exposure: float) -> float #foreign libnvtt "_ZN4nvtt18rmsToneMappedErrorERKNS_7SurfaceES2_f";
}

#if OS == .WINDOWS {
    version :: () -> u32 #foreign libnvtt "?version@nvtt@@YAIXZ";
}

#if OS == .LINUX {
    version :: () -> u32 #foreign libnvtt "_ZN4nvtt7versionEv";
}

#if OS == .MACOS {
    version :: () -> u32 #foreign libnvtt "_ZN4nvtt7versionEv";
}

// Convenience operations

#if OS == .WINDOWS {
    angularError :: (reference: Surface, img: Surface) -> float #no_context {
        return angularError(*reference, *img);
    }
}

#if OS == .LINUX {
    angularError :: (reference: Surface, img: Surface) -> float #no_context {
        return angularError(*reference, *img);
    }
}

#if OS == .MACOS {
    angularError :: (reference: Surface, img: Surface) -> float #no_context {
        return angularError(*reference, *img);
    }
}

#if OS == .WINDOWS {
    cieLabError :: (reference: Surface, img: Surface) -> float #no_context {
        return cieLabError(*reference, *img);
    }
}

#if OS == .LINUX {
    cieLabError :: (reference: Surface, img: Surface) -> float #no_context {
        return cieLabError(*reference, *img);
    }
}

#if OS == .MACOS {
    cieLabError :: (reference: Surface, img: Surface) -> float #no_context {
        return cieLabError(*reference, *img);
    }
}

#if OS == .WINDOWS {
    diff :: (reference: Surface, img: Surface, scale: float) -> Surface #no_context {
        return diff(*reference, *img, scale);
    }
}

#if OS == .LINUX {
    diff :: (reference: Surface, img: Surface, scale: float) -> Surface #no_context {
        return diff(*reference, *img, scale);
    }
}

#if OS == .MACOS {
    diff :: (reference: Surface, img: Surface, scale: float) -> Surface #no_context {
        return diff(*reference, *img, scale);
    }
}

#if OS == .WINDOWS {
    ErrorHandler_Destructor :: inline (this: *ErrorHandler, flags: Msvc_Destructor_Flags = .NONE) {
        this.vtable.Destructor(this, flags);
    }
}

#if OS == .LINUX {
    ErrorHandler_Destructor :: inline (this: *ErrorHandler) {
        this.vtable.Destructor(this);
    }
}

#if OS == .MACOS {
    ErrorHandler_Destructor :: inline (this: *ErrorHandler) {
        this.vtable.Destructor(this);
    }
}

#if OS == .LINUX {
    ErrorHandler_Destructor_Deleting :: inline (this: *ErrorHandler) {
        this.vtable.Destructor_Deleting(this);
    }
}

#if OS == .MACOS {
    ErrorHandler_Destructor_Deleting :: inline (this: *ErrorHandler) {
        this.vtable.Destructor_Deleting(this);
    }
}

#if OS == .WINDOWS {
    ErrorHandler_error :: inline (this: *ErrorHandler, e: Error) {
        this.vtable.error(this, e);
    }
}

#if OS == .LINUX {
    ErrorHandler_error :: inline (this: *ErrorHandler, e: Error) {
        this.vtable.error(this, e);
    }
}

#if OS == .MACOS {
    ErrorHandler_error :: inline (this: *ErrorHandler, e: Error) {
        this.vtable.error(this, e);
    }
}

#if OS == .WINDOWS {
    histogram :: (img: Surface, width: s32, height: s32) -> Surface #no_context {
        return histogram(*img, width, height);
    }
}

#if OS == .WINDOWS {
    histogram :: (img: Surface, minRange: float, maxRange: float, width: s32, height: s32) -> Surface #no_context {
        return histogram(*img, minRange, maxRange, width, height);
    }
}

#if OS == .LINUX {
    histogram :: (img: Surface, width: s32, height: s32) -> Surface #no_context {
        return histogram(*img, width, height);
    }
}

#if OS == .LINUX {
    histogram :: (img: Surface, minRange: float, maxRange: float, width: s32, height: s32) -> Surface #no_context {
        return histogram(*img, minRange, maxRange, width, height);
    }
}

#if OS == .MACOS {
    histogram :: (img: Surface, width: s32, height: s32) -> Surface #no_context {
        return histogram(*img, width, height);
    }
}

#if OS == .MACOS {
    histogram :: (img: Surface, minRange: float, maxRange: float, width: s32, height: s32) -> Surface #no_context {
        return histogram(*img, minRange, maxRange, width, height);
    }
}

#if OS == .WINDOWS {
    operator_assign :: (this: *Surface, img: Surface) #no_context {
        operator_assign(this, *img);
    }
}

#if OS == .WINDOWS {
    operator_assign :: (this: *CubeSurface, img: CubeSurface) #no_context {
        operator_assign(this, *img);
    }
}

#if OS == .LINUX {
    operator_assign :: (this: *Surface, img: Surface) #no_context {
        operator_assign(this, *img);
    }
}

#if OS == .LINUX {
    operator_assign :: (this: *CubeSurface, img: CubeSurface) #no_context {
        operator_assign(this, *img);
    }
}

#if OS == .MACOS {
    operator_assign :: (this: *Surface, img: Surface) #no_context {
        operator_assign(this, *img);
    }
}

#if OS == .MACOS {
    operator_assign :: (this: *CubeSurface, img: CubeSurface) #no_context {
        operator_assign(this, *img);
    }
}

#if OS == .WINDOWS {
    OutputHandler_beginImage :: inline (this: *OutputHandler, size: s32, width: s32, height: s32, depth: s32, face: s32, miplevel: s32) {
        this.vtable.beginImage(this, size, width, height, depth, face, miplevel);
    }
}

#if OS == .LINUX {
    OutputHandler_beginImage :: inline (this: *OutputHandler, size: s32, width: s32, height: s32, depth: s32, face: s32, miplevel: s32) {
        this.vtable.beginImage(this, size, width, height, depth, face, miplevel);
    }
}

#if OS == .MACOS {
    OutputHandler_beginImage :: inline (this: *OutputHandler, size: s32, width: s32, height: s32, depth: s32, face: s32, miplevel: s32) {
        this.vtable.beginImage(this, size, width, height, depth, face, miplevel);
    }
}

#if OS == .WINDOWS {
    OutputHandler_Destructor :: inline (this: *OutputHandler, flags: Msvc_Destructor_Flags = .NONE) {
        this.vtable.Destructor(this, flags);
    }
}

#if OS == .LINUX {
    OutputHandler_Destructor :: inline (this: *OutputHandler) {
        this.vtable.Destructor(this);
    }
}

#if OS == .MACOS {
    OutputHandler_Destructor :: inline (this: *OutputHandler) {
        this.vtable.Destructor(this);
    }
}

#if OS == .LINUX {
    OutputHandler_Destructor_Deleting :: inline (this: *OutputHandler) {
        this.vtable.Destructor_Deleting(this);
    }
}

#if OS == .MACOS {
    OutputHandler_Destructor_Deleting :: inline (this: *OutputHandler) {
        this.vtable.Destructor_Deleting(this);
    }
}

#if OS == .WINDOWS {
    OutputHandler_endImage :: inline (this: *OutputHandler) {
        this.vtable.endImage(this);
    }
}

#if OS == .LINUX {
    OutputHandler_endImage :: inline (this: *OutputHandler) {
        this.vtable.endImage(this);
    }
}

#if OS == .MACOS {
    OutputHandler_endImage :: inline (this: *OutputHandler) {
        this.vtable.endImage(this);
    }
}

#if OS == .WINDOWS {
    OutputHandler_writeData :: inline (this: *OutputHandler, data: *void, size: s32) -> bool {
        return this.vtable.writeData(this, data, size);
    }
}

#if OS == .LINUX {
    OutputHandler_writeData :: inline (this: *OutputHandler, data: *void, size: s32) -> bool {
        return this.vtable.writeData(this, data, size);
    }
}

#if OS == .MACOS {
    OutputHandler_writeData :: inline (this: *OutputHandler, data: *void, size: s32) -> bool {
        return this.vtable.writeData(this, data, size);
    }
}

#if OS == .WINDOWS {
    rmsAlphaError :: (reference: Surface, img: Surface) -> float #no_context {
        return rmsAlphaError(*reference, *img);
    }
}

#if OS == .LINUX {
    rmsAlphaError :: (reference: Surface, img: Surface) -> float #no_context {
        return rmsAlphaError(*reference, *img);
    }
}

#if OS == .MACOS {
    rmsAlphaError :: (reference: Surface, img: Surface) -> float #no_context {
        return rmsAlphaError(*reference, *img);
    }
}

#if OS == .WINDOWS {
    rmsError :: (reference: Surface, img: Surface) -> float #no_context {
        return rmsError(*reference, *img);
    }
}

#if OS == .LINUX {
    rmsError :: (reference: Surface, img: Surface) -> float #no_context {
        return rmsError(*reference, *img);
    }
}

#if OS == .MACOS {
    rmsError :: (reference: Surface, img: Surface) -> float #no_context {
        return rmsError(*reference, *img);
    }
}

#if OS == .WINDOWS {
    rmsToneMappedError :: (reference: Surface, img: Surface, exposure: float) -> float #no_context {
        return rmsToneMappedError(*reference, *img, exposure);
    }
}

#if OS == .LINUX {
    rmsToneMappedError :: (reference: Surface, img: Surface, exposure: float) -> float #no_context {
        return rmsToneMappedError(*reference, *img, exposure);
    }
}

#if OS == .MACOS {
    rmsToneMappedError :: (reference: Surface, img: Surface, exposure: float) -> float #no_context {
        return rmsToneMappedError(*reference, *img, exposure);
    }
}

#if OS == .WINDOWS {
    TaskDispatcher_Destructor :: inline (this: *TaskDispatcher, flags: Msvc_Destructor_Flags = .NONE) {
        this.vtable.Destructor(this, flags);
    }
}

#if OS == .LINUX {
    TaskDispatcher_Destructor :: inline (this: *TaskDispatcher) {
        this.vtable.Destructor(this);
    }
}

#if OS == .MACOS {
    TaskDispatcher_Destructor :: inline (this: *TaskDispatcher) {
        this.vtable.Destructor(this);
    }
}

#if OS == .LINUX {
    TaskDispatcher_Destructor_Deleting :: inline (this: *TaskDispatcher) {
        this.vtable.Destructor_Deleting(this);
    }
}

#if OS == .MACOS {
    TaskDispatcher_Destructor_Deleting :: inline (this: *TaskDispatcher) {
        this.vtable.Destructor_Deleting(this);
    }
}

#if OS == .WINDOWS {
    TaskDispatcher_dispatch :: inline (this: *TaskDispatcher, task: Task, _context: *void, count: s32) {
        this.vtable.dispatch(this, task, _context, count);
    }
}

#if OS == .LINUX {
    TaskDispatcher_dispatch :: inline (this: *TaskDispatcher, task: Task, _context: *void, count: s32) {
        this.vtable.dispatch(this, task, _context, count);
    }
}

#if OS == .MACOS {
    TaskDispatcher_dispatch :: inline (this: *TaskDispatcher, task: Task, _context: *void, count: s32) {
        this.vtable.dispatch(this, task, _context, count);
    }
}

#if OS == .WINDOWS {
    vtable :: (obj: *OutputHandler) -> *OutputHandler_VTable {
        return obj.vtable;
    }
}

#if OS == .WINDOWS {
    vtable :: (obj: *ErrorHandler) -> *ErrorHandler_VTable {
        return obj.vtable;
    }
}

#if OS == .WINDOWS {
    vtable :: (obj: *TaskDispatcher) -> *TaskDispatcher_VTable {
        return obj.vtable;
    }
}

#if OS == .LINUX {
    vtable :: (obj: *OutputHandler) -> *OutputHandler_VTable {
        return obj.vtable;
    }
}

#if OS == .LINUX {
    vtable :: (obj: *ErrorHandler) -> *ErrorHandler_VTable {
        return obj.vtable;
    }
}

#if OS == .LINUX {
    vtable :: (obj: *TaskDispatcher) -> *TaskDispatcher_VTable {
        return obj.vtable;
    }
}

#if OS == .MACOS {
    vtable :: (obj: *OutputHandler) -> *OutputHandler_VTable {
        return obj.vtable;
    }
}

#if OS == .MACOS {
    vtable :: (obj: *ErrorHandler) -> *ErrorHandler_VTable {
        return obj.vtable;
    }
}

#if OS == .MACOS {
    vtable :: (obj: *TaskDispatcher) -> *TaskDispatcher_VTable {
        return obj.vtable;
    }
}

// Generated from docs/specs/stdlib/modules/lz4.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

#scope_file

liblz4 :: #library,system "lz4";
#scope_export

// Types and layouts

LZ4_stream_t_internal :: struct {
    hashTable: [4096] LZ4_u32;
    dictionary: *LZ4_byte;
    dictCtx: *LZ4_stream_t_internal;
    currentOffset: LZ4_u32;
    tableType: LZ4_u32;
    dictSize: LZ4_u32;
}

LZ4_stream_u :: union {
    minStateSize: [16416] u8;
    internal_donotuse: LZ4_stream_t_internal;
}

LZ4_streamDecode_t_internal :: struct {
    externalDict: *LZ4_byte;
    prefixEnd: *LZ4_byte;
    extDictSize: u64;
    prefixSize: u64;
}

LZ4_streamDecode_u :: union {
    minStateSize: [32] u8;
    internal_donotuse: LZ4_streamDecode_t_internal;
}

LZ4_streamHC_u :: union {
    minStateSize: [262200] u8;
    internal_donotuse: LZ4HC_CCtx_internal;
}

LZ4HC_CCtx_internal :: struct {
    hashTable: [32768] LZ4_u32;
    chainTable: [65536] LZ4_u16;
    end: *LZ4_byte;
    prefixStart: *LZ4_byte;
    dictStart: *LZ4_byte;
    dictLimit: LZ4_u32;
    lowLimit: LZ4_u32;
    nextToUpdate: LZ4_u32;
    compressionLevel: s16;
    favorDecSpeed: LZ4_i8;
    dirty: LZ4_i8;
    dictCtx: *LZ4HC_CCtx_internal;
}

// Aliases and constants

LZ4_byte :: u8;

LZ4_HASH_SIZE_U32 :: 1 << LZ4_HASHLOG;

LZ4_HASHLOG :: LZ4_MEMORY_USAGE - 2;

LZ4_HASHTABLESIZE :: 1 << LZ4_MEMORY_USAGE;

LZ4_i8 :: s8;

LZ4_MAX_INPUT_SIZE :: 0x7E000000;

LZ4_MEMORY_USAGE :: LZ4_MEMORY_USAGE_DEFAULT;

LZ4_MEMORY_USAGE_DEFAULT :: 14;

LZ4_MEMORY_USAGE_MAX :: 20;

LZ4_MEMORY_USAGE_MIN :: 10;

LZ4_STREAM_MINSIZE :: (1 << LZ4_MEMORY_USAGE) + 32;

LZ4_stream_t :: LZ4_stream_u;

LZ4_STREAMDECODE_MINSIZE :: 32;

LZ4_streamDecode_t :: LZ4_streamDecode_u;

LZ4_STREAMHC_MINSIZE :: 262200;

LZ4_streamHC_t :: LZ4_streamHC_u;

LZ4_u16 :: u16;

LZ4_u32 :: u32;

LZ4_VERSION_MAJOR :: 1;

LZ4_VERSION_MINOR :: 9;

LZ4_VERSION_NUMBER :: LZ4_VERSION_MAJOR * 100 * 100 + LZ4_VERSION_MINOR * 100 + LZ4_VERSION_RELEASE;

LZ4_VERSION_RELEASE :: 4;

LZ4HC_CLEVEL_DEFAULT :: 9;

LZ4HC_CLEVEL_MAX :: 12;

LZ4HC_CLEVEL_MIN :: 3;

LZ4HC_CLEVEL_OPT_MIN :: 10;

LZ4HC_DICTIONARY_LOGSIZE :: 16;

LZ4HC_HASH_LOG :: 15;

LZ4HC_HASH_MASK :: LZ4HC_HASHTABLESIZE - 1;

LZ4HC_HASHTABLESIZE :: 1 << LZ4HC_HASH_LOG;

LZ4HC_MAXD :: 1 << LZ4HC_DICTIONARY_LOGSIZE;

LZ4HC_MAXD_MASK :: LZ4HC_MAXD - 1;

// Native entry points

LZ4_compress :: (src: *u8, dest: *u8, srcSize: s32) -> s32 #foreign liblz4;

LZ4_compress_continue :: (LZ4_streamPtr: *LZ4_stream_t, source: *u8, dest: *u8, inputSize: s32) -> s32 #foreign liblz4;

LZ4_compress_default :: (src: *u8, dst: *u8, srcSize: s32, dstCapacity: s32) -> s32 #foreign liblz4;

LZ4_compress_destSize :: (src: *u8, dst: *u8, srcSizePtr: *s32, targetDstSize: s32) -> s32 #foreign liblz4;

LZ4_compress_fast :: (src: *u8, dst: *u8, srcSize: s32, dstCapacity: s32, acceleration: s32) -> s32 #foreign liblz4;

LZ4_compress_fast_continue :: (streamPtr: *LZ4_stream_t, src: *u8, dst: *u8, srcSize: s32, dstCapacity: s32, acceleration: s32) -> s32 #foreign liblz4;

LZ4_compress_fast_extState :: (state: *void, src: *u8, dst: *u8, srcSize: s32, dstCapacity: s32, acceleration: s32) -> s32 #foreign liblz4;

LZ4_compress_HC :: (src: *u8, dst: *u8, srcSize: s32, dstCapacity: s32, compressionLevel: s32) -> s32 #foreign liblz4;

LZ4_compress_HC_continue :: (streamHCPtr: *LZ4_streamHC_t, src: *u8, dst: *u8, srcSize: s32, maxDstSize: s32) -> s32 #foreign liblz4;

LZ4_compress_HC_continue_destSize :: (LZ4_streamHCPtr: *LZ4_streamHC_t, src: *u8, dst: *u8, srcSizePtr: *s32, targetDstSize: s32) -> s32 #foreign liblz4;

LZ4_compress_HC_destSize :: (stateHC: *void, src: *u8, dst: *u8, srcSizePtr: *s32, targetDstSize: s32, compressionLevel: s32) -> s32 #foreign liblz4;

LZ4_compress_HC_extStateHC :: (stateHC: *void, src: *u8, dst: *u8, srcSize: s32, maxDstSize: s32, compressionLevel: s32) -> s32 #foreign liblz4;

LZ4_compress_limitedOutput :: (src: *u8, dest: *u8, srcSize: s32, maxOutputSize: s32) -> s32 #foreign liblz4;

LZ4_compress_limitedOutput_continue :: (LZ4_streamPtr: *LZ4_stream_t, source: *u8, dest: *u8, inputSize: s32, maxOutputSize: s32) -> s32 #foreign liblz4;

LZ4_compress_limitedOutput_withState :: (state: *void, source: *u8, dest: *u8, inputSize: s32, maxOutputSize: s32) -> s32 #foreign liblz4;

LZ4_compress_withState :: (state: *void, source: *u8, dest: *u8, inputSize: s32) -> s32 #foreign liblz4;

LZ4_compressBound :: (inputSize: s32) -> s32 #foreign liblz4;

LZ4_compressHC :: (source: *u8, dest: *u8, inputSize: s32) -> s32 #foreign liblz4;

LZ4_compressHC2 :: (source: *u8, dest: *u8, inputSize: s32, compressionLevel: s32) -> s32 #foreign liblz4;

LZ4_compressHC2_continue :: (LZ4HC_Data: *void, source: *u8, dest: *u8, inputSize: s32, compressionLevel: s32) -> s32 #foreign liblz4;

LZ4_compressHC2_limitedOutput :: (source: *u8, dest: *u8, inputSize: s32, maxOutputSize: s32, compressionLevel: s32) -> s32 #foreign liblz4;

LZ4_compressHC2_limitedOutput_continue :: (LZ4HC_Data: *void, source: *u8, dest: *u8, inputSize: s32, maxOutputSize: s32, compressionLevel: s32) -> s32 #foreign liblz4;

LZ4_compressHC2_limitedOutput_withStateHC :: (state: *void, source: *u8, dest: *u8, inputSize: s32, maxOutputSize: s32, compressionLevel: s32) -> s32 #foreign liblz4;

LZ4_compressHC2_withStateHC :: (state: *void, source: *u8, dest: *u8, inputSize: s32, compressionLevel: s32) -> s32 #foreign liblz4;

LZ4_compressHC_continue :: (LZ4_streamHCPtr: *LZ4_streamHC_t, source: *u8, dest: *u8, inputSize: s32) -> s32 #foreign liblz4;

LZ4_compressHC_limitedOutput :: (source: *u8, dest: *u8, inputSize: s32, maxOutputSize: s32) -> s32 #foreign liblz4;

LZ4_compressHC_limitedOutput_continue :: (LZ4_streamHCPtr: *LZ4_streamHC_t, source: *u8, dest: *u8, inputSize: s32, maxOutputSize: s32) -> s32 #foreign liblz4;

LZ4_compressHC_limitedOutput_withStateHC :: (state: *void, source: *u8, dest: *u8, inputSize: s32, maxOutputSize: s32) -> s32 #foreign liblz4;

LZ4_compressHC_withStateHC :: (state: *void, source: *u8, dest: *u8, inputSize: s32) -> s32 #foreign liblz4;

LZ4_create :: (inputBuffer: *u8) -> *void #foreign liblz4;

LZ4_createHC :: (inputBuffer: *u8) -> *void #foreign liblz4;

LZ4_createStream :: () -> *LZ4_stream_t #foreign liblz4;

LZ4_createStreamDecode :: () -> *LZ4_streamDecode_t #foreign liblz4;

LZ4_createStreamHC :: () -> *LZ4_streamHC_t #foreign liblz4;

LZ4_decoderRingBufferSize :: (maxBlockSize: s32) -> s32 #foreign liblz4;

LZ4_decompress_fast :: (src: *u8, dst: *u8, originalSize: s32) -> s32 #foreign liblz4;

LZ4_decompress_fast_continue :: (LZ4_streamDecode: *LZ4_streamDecode_t, src: *u8, dst: *u8, originalSize: s32) -> s32 #foreign liblz4;

LZ4_decompress_fast_usingDict :: (src: *u8, dst: *u8, originalSize: s32, dictStart: *u8, dictSize: s32) -> s32 #foreign liblz4;

LZ4_decompress_fast_withPrefix64k :: (src: *u8, dst: *u8, originalSize: s32) -> s32 #foreign liblz4;

LZ4_decompress_safe :: (src: *u8, dst: *u8, compressedSize: s32, dstCapacity: s32) -> s32 #foreign liblz4;

LZ4_decompress_safe_continue :: (LZ4_streamDecode: *LZ4_streamDecode_t, src: *u8, dst: *u8, srcSize: s32, dstCapacity: s32) -> s32 #foreign liblz4;

LZ4_decompress_safe_partial :: (src: *u8, dst: *u8, srcSize: s32, targetOutputSize: s32, dstCapacity: s32) -> s32 #foreign liblz4;

LZ4_decompress_safe_partial_usingDict :: (src: *u8, dst: *u8, compressedSize: s32, targetOutputSize: s32, maxOutputSize: s32, dictStart: *u8, dictSize: s32) -> s32 #foreign liblz4;

LZ4_decompress_safe_usingDict :: (src: *u8, dst: *u8, srcSize: s32, dstCapacity: s32, dictStart: *u8, dictSize: s32) -> s32 #foreign liblz4;

LZ4_decompress_safe_withPrefix64k :: (src: *u8, dst: *u8, compressedSize: s32, maxDstSize: s32) -> s32 #foreign liblz4;

LZ4_freeHC :: (LZ4HC_Data: *void) -> s32 #foreign liblz4;

LZ4_freeStream :: (streamPtr: *LZ4_stream_t) -> s32 #foreign liblz4;

LZ4_freeStreamDecode :: (LZ4_stream: *LZ4_streamDecode_t) -> s32 #foreign liblz4;

LZ4_freeStreamHC :: (streamHCPtr: *LZ4_streamHC_t) -> s32 #foreign liblz4;

LZ4_initStream :: (buffer: *void, size: u64) -> *LZ4_stream_t #foreign liblz4;

LZ4_initStreamHC :: (buffer: *void, size: u64) -> *LZ4_streamHC_t #foreign liblz4;

LZ4_loadDict :: (streamPtr: *LZ4_stream_t, dictionary: *u8, dictSize: s32) -> s32 #foreign liblz4;

LZ4_loadDictHC :: (streamHCPtr: *LZ4_streamHC_t, dictionary: *u8, dictSize: s32) -> s32 #foreign liblz4;

LZ4_resetStream :: (streamPtr: *LZ4_stream_t) -> void #foreign liblz4;

LZ4_resetStream_fast :: (streamPtr: *LZ4_stream_t) -> void #foreign liblz4;

LZ4_resetStreamHC :: (streamHCPtr: *LZ4_streamHC_t, compressionLevel: s32) -> void #foreign liblz4;

LZ4_resetStreamHC_fast :: (streamHCPtr: *LZ4_streamHC_t, compressionLevel: s32) -> void #foreign liblz4;

LZ4_resetStreamState :: (state: *void, inputBuffer: *u8) -> s32 #foreign liblz4;

LZ4_resetStreamStateHC :: (state: *void, inputBuffer: *u8) -> s32 #foreign liblz4;

LZ4_saveDict :: (streamPtr: *LZ4_stream_t, safeBuffer: *u8, maxDictSize: s32) -> s32 #foreign liblz4;

LZ4_saveDictHC :: (streamHCPtr: *LZ4_streamHC_t, safeBuffer: *u8, maxDictSize: s32) -> s32 #foreign liblz4;

LZ4_setStreamDecode :: (LZ4_streamDecode: *LZ4_streamDecode_t, dictionary: *u8, dictSize: s32) -> s32 #foreign liblz4;

LZ4_sizeofState :: () -> s32 #foreign liblz4;

LZ4_sizeofStateHC :: () -> s32 #foreign liblz4;

LZ4_sizeofStreamState :: () -> s32 #foreign liblz4;

LZ4_sizeofStreamStateHC :: () -> s32 #foreign liblz4;

LZ4_slideInputBuffer :: (state: *void) -> *u8 #foreign liblz4;

LZ4_slideInputBufferHC :: (LZ4HC_Data: *void) -> *u8 #foreign liblz4;

LZ4_uncompress :: (source: *u8, dest: *u8, outputSize: s32) -> s32 #foreign liblz4;

LZ4_uncompress_unknownOutputSize :: (source: *u8, dest: *u8, isize: s32, maxOutputSize: s32) -> s32 #foreign liblz4;

LZ4_versionNumber :: () -> s32 #foreign liblz4;

LZ4_versionString :: () -> *u8 #foreign liblz4;

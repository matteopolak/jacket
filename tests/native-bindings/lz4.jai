L :: #import "lz4";

#assert L.LZ4_VERSION_NUMBER == 10904;
#assert size_of(L.LZ4_stream_u) == 16416;
#assert size_of(L.LZ4_streamDecode_u) == 32;
#assert size_of(L.LZ4_streamHC_u) == 262200;
#assert L.LZ4_stream_t == L.LZ4_stream_u;
#assert L.LZ4_streamHC_t == L.LZ4_streamHC_u;

probe :: #library "../../.work/native/abi_probe";
check_size :: (which: s32) -> u64 #foreign probe "binding_size";
check_offset :: (which: s32) -> u64 #foreign probe "binding_offset";
libc :: #library,system "c";
abort :: () #foreign libc;

check :: (value: bool) {
    if !value abort();
}

main :: () {
    check(type_info(L.LZ4_stream_t).name == "LZ4_stream_u");
    check(type_info(L.LZ4HC_CCtx_internal).name == "LZ4HC_CCtx_internal");
    check(L.LZ4_versionNumber() == L.LZ4_VERSION_NUMBER);
    check(size_of(L.LZ4_stream_u) == check_size(0));
    check(size_of(L.LZ4_streamDecode_u) == check_size(1));
    check(size_of(L.LZ4_streamHC_u) == check_size(2));
    check(size_of(L.LZ4_stream_t_internal) == check_size(3));
    check(size_of(L.LZ4HC_CCtx_internal) == check_size(4));
    s: L.LZ4_stream_t_internal;
    h: L.LZ4HC_CCtx_internal;

    check(cast(u64) *s.dictionary - cast(u64) *s == check_offset(0));
    check(cast(u64) *h.compressionLevel - cast(u64) *h == check_offset(1));
    check(cast(u64) *h.dictCtx - cast(u64) *h == check_offset(2));
    input := "repeated input repeated input repeated input repeated input";
    compressed: [256] u8;
    decoded: [256] u8;
    n := L.LZ4_compress_default(input.data, compressed.data, cast(s32) input.count, 256);

    check(n > 0);
    count := L.LZ4_decompress_safe(compressed.data, decoded.data, n, 256);

    check(count == input.count);
    for i: 0..input.count - 1 check(input[i] == decoded[i]);

    check(L.LZ4_decompress_safe(compressed.data, decoded.data, n, 1) < 0);
    hc := L.LZ4_createStreamHC();

    check(hc != null);
    defer check(L.LZ4_freeStreamHC(hc) == 0);
    L.LZ4_resetStreamHC_fast(hc, L.LZ4HC_CLEVEL_DEFAULT);
    n = L.LZ4_compress_HC_continue(hc, input.data, compressed.data, cast(s32) input.count, 256);
    check(n > 0);
    count = L.LZ4_decompress_safe(compressed.data, decoded.data, n, 256);
    check(count == input.count);
    for i: 0..input.count - 1 check(input[i] == decoded[i]);
}

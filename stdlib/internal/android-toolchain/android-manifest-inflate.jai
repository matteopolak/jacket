// Private system-zlib ABI, from the platform SDK's zlib.h. ZIP parsing stays
// in the canonical APK library. Android ZIP entries use raw DEFLATE (-15).
#scope_module
#if OS == .WINDOWS {
    AM_Z_Long :: u32;
    am_zlib :: #library,system "zlib1";
} else {
    AM_Z_Long :: u64;
    am_zlib :: #library,system "z";
}

AM_Z_Stream :: struct {
    next_in: *u8;
    avail_in: u32;
    total_in: AM_Z_Long;
    next_out: *u8;
    avail_out: u32;
    total_out: AM_Z_Long;
    msg: *u8;
    state: *void;
    zalloc: *void;
    zfree: *void;
    opaque: *void;
    data_type: s32;
    adler: AM_Z_Long;
    reserved: AM_Z_Long;
}

am_z_version :: () -> *u8 #foreign am_zlib "zlibVersion";
am_z_init :: (stream: *AM_Z_Stream, window_bits: s32, version: *u8, stream_size: s32) -> s32 #foreign am_zlib "inflateInit2_";
am_z_inflate :: (stream: *AM_Z_Stream, flush: s32) -> s32 #foreign am_zlib "inflate";
am_z_end :: (stream: *AM_Z_Stream) -> s32 #foreign am_zlib "inflateEnd";

am_inflate :: (compressed: string, expected: u32) -> bool, string {
    if compressed.count > 0xffffffff || expected == 0 return false, "";

    output := alloc_string(expected);
    if !output.data return false, "";

    stream: AM_Z_Stream;
    stream.next_in = compressed.data;
    stream.avail_in = cast(u32) compressed.count;
    stream.next_out = output.data;
    stream.avail_out = expected;
    if am_z_init(*stream, -15, am_z_version(), cast(s32) size_of(AM_Z_Stream)) != 0 {
        free(output);
        return false, "";
    }

    defer am_z_end(*stream);
    status := am_z_inflate(*stream, 4);
    if status != 1 || stream.total_out != expected || stream.total_in != cast(AM_Z_Long) compressed.count {
        free(output);
        return false, "";
    }

    return true, output;
}

#import "Basic";

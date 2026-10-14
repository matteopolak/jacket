I :: #import "stb_image";
W :: #import "stb_image_write";
R :: #import "stb_image_resize";
V :: #import "stb_vorbis";

libc :: #library,system "c";
abort :: () #foreign libc;
free :: (p: *void) #foreign libc;

check :: (value: bool) {
    if !value abort();
}

Sink :: struct {
    bytes: [2048] u8;
    count: s32;
}

write_bytes :: (user: *void, data: *void, size: s32) #c_call {
    sink := cast(*Sink) user;
    if sink.count + size > 2048 abort();

    p := cast(*u8) data;
    for i: 0..size - 1 sink.bytes[sink.count + i] = p[i];

    sink.count += size;
}

main :: () {
    pixels: [12] u8 = .[255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255];
    length: s32;
    png := W.stbi_write_png_to_mem(pixels.data, 6, 2, 2, 3, *length);

    check(png != null && length > 0);
    defer free(png);
    x, y, channels: s32;
    decoded := I.stbi_load_from_memory(png, length, *x, *y, *channels, 3);

    check(decoded != null && x == 2 && y == 2 && channels == 3);
    defer I.stbi_image_free(decoded);
    for i: 0..11 check(pixels[i] == decoded[i]);

    sink: Sink;

    check(W.stbi_write_png_to_func(write_bytes, *sink, 2, 2, 3, pixels.data, 6) == 1);
    check(sink.count > 0);
    roundtrip := I.stbi_load_from_memory(sink.bytes.data, sink.count, *x, *y, *channels, 3);

    check(roundtrip != null);
    defer I.stbi_image_free(roundtrip);
    for i: 0..11 check(pixels[i] == roundtrip[i]);

    single: [3] u8 = .[47, 83, 129];
    resized: [48] u8;

    check(R.stbir_resize_uint8(single.data, 1, 1, 0, resized.data, 4, 4, 0, 3) == 1);
    for i: 0..47 check(resized[i] == single[i % 3]);

    invalid := "invalid image";

    check(I.stbi_load_from_memory(invalid.data, cast(s32) invalid.count, *x, *y, *channels, 3) == null);
    check(I.stbi_failure_reason() != null);
    error: s32;

    check(V.stb_vorbis_open_memory(invalid.data, cast(s32) invalid.count, *error, null) == null);
    check(error != 0);
    original := W.stbi_write_png_compression_level;

    W.stbi_write_png_compression_level = 4;
    check(W.stbi_write_png_compression_level == 4);
    W.stbi_write_png_compression_level = original;
}

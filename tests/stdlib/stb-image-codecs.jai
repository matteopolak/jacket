// stb_image, stb_image_write and stb_image_resize against the pinned C libraries: encode a small
// picture to PNG, BMP, TGA, JPEG and HDR in memory, decode each back, inspect headers, resize, and
// check that corrupt input fails with a reason instead of crashing.
#import "Basic";
#import "Math";
Load :: #import "stb_image";
Write :: #import "stb_image_write";
Resize :: #import "stb_image_resize";

Sink :: struct {
    bytes: [..] u8;
}

collect :: (user: *void, data: *void, size: s32) #c_call {
    sink := cast(*Sink) user;
    base_context: #Context;
    push_context base_context {
        for 0..size - 1 array_add(*sink.bytes, (cast(*u8) data)[it]);
    }
}

WIDTH :: 8;
HEIGHT :: 6;

make_picture :: () -> [] u8 {
    pixels := NewArray(WIDTH * HEIGHT * 4, u8);
    for y: 0..HEIGHT - 1 for x : 0..WIDTH - 1 {
        texel := pixels.data + (y * WIDTH + x) * 4;
        texel[0] = cast(u8) (x * 30);
        texel[1] = cast(u8) (y * 40);
        texel[2] = cast(u8) (x * 10 + y * 10);
        texel[3] = 255;
    }
    return pixels;
}

decode :: (sink: Sink, channels: s32) -> [] u8, width: s32, height: s32 {
    w, h, found: s32;
    loaded := Load.stbi_load_from_memory(sink.bytes.data, cast(s32) sink.bytes.count, *w, *h, *found, channels);
    assert(loaded != null, "decode failed: %", to_string(Load.stbi_failure_reason()));
    result := NewArray(w * h * channels, u8);
    memcpy(result.data, loaded, w * h * channels);
    Load.stbi_image_free(loaded);
    return result, w, h;
}

main :: () {
    picture := make_picture();

    // PNG is lossless: every byte comes back, and the header reports the size and channels.
    png: Sink;
    assert(Write.stbi_write_png_to_func(collect, *png, WIDTH, HEIGHT, 4, picture.data, WIDTH * 4) != 0);
    assert(png.bytes.count > 8 && png.bytes[1] == #char "P" && png.bytes[2] == #char "N" && png.bytes[3] == #char "G");
    w, h, comp: s32;
    assert(Load.stbi_info_from_memory(png.bytes.data, cast(s32) png.bytes.count, *w, *h, *comp) != 0);
    assert(w == WIDTH && h == HEIGHT && comp == 4, "png info % % %", w, h, comp);
    assert(Load.stbi_is_16_bit_from_memory(png.bytes.data, cast(s32) png.bytes.count) == 0);
    back, bw, bh := decode(png, 4);
    assert(bw == WIDTH && bh == HEIGHT && back.count == picture.count);
    for picture assert(back[it_index] == it, "png byte % differs", it_index);
    // Asking for one channel converts to luminance; three drops alpha.
    gray := decode(png, 1);
    assert(gray.count == WIDTH * HEIGHT);
    rgb := decode(png, 3);
    assert(rgb.count == WIDTH * HEIGHT * 3 && rgb[0] == picture[0] && rgb[3] == picture[4]);

    // Flipping on load reverses the rows.
    Load.stbi_set_flip_vertically_on_load(1);
    flipped := decode(png, 4);
    Load.stbi_set_flip_vertically_on_load(0);
    for x: 0..WIDTH * 4 - 1 assert(flipped[x] == picture[(HEIGHT - 1) * WIDTH * 4 + x], "flipped row at %", x);

    // BMP and TGA are lossless too (their stored channel order is handled by the codecs).
    bmp: Sink;
    assert(Write.stbi_write_bmp_to_func(collect, *bmp, WIDTH, HEIGHT, 4, picture.data) != 0);
    from_bmp := decode(bmp, 4);
    for from_bmp if it_index % 4 != 3 assert(it == picture[it_index], "bmp byte % differs", it_index);
    tga: Sink;
    assert(Write.stbi_write_tga_to_func(collect, *tga, WIDTH, HEIGHT, 4, picture.data) != 0);
    from_tga := decode(tga, 4);
    for from_tga assert(it == picture[it_index], "tga byte % differs", it_index);

    // JPEG is lossy: the decoded picture is close, not equal.
    jpg: Sink;
    assert(Write.stbi_write_jpg_to_func(collect, *jpg, WIDTH, HEIGHT, 4, picture.data, 95) != 0);
    from_jpg, jw, jh := decode(jpg, 3);
    assert(jw == WIDTH && jh == HEIGHT);
    worst: s32;
    for y: 0..HEIGHT - 1 for x : 0..WIDTH - 1 for c : 0..2 {
        error := abs(cast(s32) from_jpg[(y * WIDTH + x) * 3 + c] - cast(s32) picture[(y * WIDTH + x) * 4 + c]);
        if error > worst worst = error;
    }
    assert(worst < 40, "jpeg error %", worst);

    // HDR round trip reports itself as HDR.
    floats := NewArray(WIDTH * HEIGHT * 3, float);
    for *floats it.* = cast(float) (it_index % 7) * 0.25 + 0.1;
    hdr: Sink;
    assert(Write.stbi_write_hdr_to_func(collect, *hdr, WIDTH, HEIGHT, 3, floats.data) != 0);
    assert(Load.stbi_is_hdr_from_memory(hdr.bytes.data, cast(s32) hdr.bytes.count) != 0);
    assert(Load.stbi_is_hdr_from_memory(png.bytes.data, cast(s32) png.bytes.count) == 0);
    hw, hh, hc: s32;
    decoded_floats := Load.stbi_loadf_from_memory(hdr.bytes.data, cast(s32) hdr.bytes.count, *hw, *hh, *hc, 3);
    assert(decoded_floats != null && hw == WIDTH && hh == HEIGHT);
    for 0..floats.count - 1 assert(abs(decoded_floats[it] - floats[it]) < 0.02 * floats[it] + 0.01, "hdr value % is % not %", it, decoded_floats[it], floats[it]);
    Load.stbi_image_free(decoded_floats);

    // Resizing a flat picture keeps it flat; halving averages neighbours.
    flat := NewArray(8 * 8, u8);
    for *flat it.* = 100;
    small := NewArray(4 * 4, u8);
    assert(Resize.stbir_resize_uint8(flat.data, 8, 8, 8, small.data, 4, 4, 4, 1) != 0);
    for small assert(it == 100, "flat pixel % became %", it_index, it);
    stripes := NewArray(8 * 8, u8);
    for *stripes it.* = ifx (it_index % 8) < 4 then cast(u8) 0 else cast(u8) 200;
    halved := NewArray(4 * 4, u8);
    assert(Resize.stbir_resize_uint8(stripes.data, 8, 8, 8, halved.data, 4, 4, 4, 1) != 0);
    assert(halved[0] < 20 && halved[3] > 180, "edges stay dark and light: % %", halved[0], halved[3]);
    bigger := NewArray(16 * 16, u8);
    assert(Resize.stbir_resize_uint8(flat.data, 8, 8, 8, bigger.data, 16, 16, 16, 1) != 0);
    assert(bigger[17] == 100 && bigger[255] == 100);

    // Corrupt or empty input fails with a reason.
    junk := u8.[1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    jw2, jh2, jc2: s32;
    assert(Load.stbi_load_from_memory(junk.data, junk.count, *jw2, *jh2, *jc2, 4) == null);
    assert(to_string(Load.stbi_failure_reason()).count > 0);
    assert(Load.stbi_info_from_memory(junk.data, junk.count, *jw2, *jh2, *jc2) == 0);
    truncated := png;
    truncated.bytes.count = png.bytes.count / 2;
    assert(Load.stbi_load_from_memory(truncated.bytes.data, cast(s32) truncated.bytes.count, *jw2, *jh2, *jc2, 4) == null, "a truncated png does not decode");
    assert(Load.stbi_load("no-such-file.png".data, *jw2, *jh2, *jc2, 4) == null);

    print("ok\n");
}

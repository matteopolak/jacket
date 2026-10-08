// Calls into each prebuilt third-party library a release archive ships (stb_image,
// stb_image_write, stb_image_resize, stb_vorbis) through Simp and the bindings, without a window.
// Exits with 0 when every call worked; release.yml runs it from the unpacked archive with
// `jaic run` and as a `jaic build` executable (docs/tools/releases.md).
#import "Basic";
Simp :: #import "Simp";
Resize :: #import "stb_image_resize";
Vorbis :: #import "stb_vorbis";

fail :: (message: string) {
    print("simp-image: %\n", message);
    exit(1);
}

main :: () {
    // A 4x2 RGBA gradient, written as PNG by stb_image_write and read back by stb_image.
    image: Simp.Bitmap;
    Simp.bitmap_alloc(*image, 4, 2, .RGBA8);
    for y: 0..image.height - 1 {
        for x: 0..image.width - 1 {
            texel := image.data.data + y * image.stride + x * 4;
            texel[0] = cast(u8) (x * 60);
            texel[1] = cast(u8) (y * 200);
            texel[2] = 7;
            texel[3] = 255;
        }
    }
    path :: "simp-image.png";
    if !Simp.bitmap_save(*image, path, save_alpha = true) fail("stbi_write_png failed");

    loaded: Simp.Bitmap;
    if !Simp.bitmap_load(*loaded, path) fail("stbi_load failed");
    if loaded.width != 4 || loaded.height != 2 fail(tprint("loaded %x%, expected 4x2", loaded.width, loaded.height));
    for y: 0..1 {
        for x: 0..3 {
            want := image.data.data + y * image.stride + x * 4;
            got := loaded.data.data + y * loaded.stride + x * 4;
            if memcmp(want, got, 4) != 0 fail(tprint("texel (%, %) changed in the round trip", x, y));
        }
    }

    // stb_image_resize: 4x2 down to 2x1.
    small: [2 * 1 * 4] u8;
    if !Resize.stbir_resize_uint8(image.data.data, 4, 2, image.stride, small.data, 2, 1, 0, 4) fail("stbir_resize_uint8 failed");
    if small[3] != 255 fail(tprint("resized alpha is %", small[3]));

    // stb_vorbis: garbage is rejected with an error code, which proves the library answered.
    junk: [64] u8;
    error: s32;
    stream := Vorbis.stb_vorbis_open_memory(junk.data, junk.count, *error, null);
    if stream fail("stb_vorbis accepted garbage");
    if error == 0 fail("stb_vorbis reported no error");

    print("simp-image: ok\n");
}

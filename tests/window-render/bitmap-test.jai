main :: () {
    bitmap: S.Bitmap;

    S.bitmap_alloc(*bitmap, 3, 2, .RGBA8);
    defer S.deinit(*bitmap);

    for i: 0..5 {
        bitmap.data[i * 4] = cast(u8) (i * 40);
        bitmap.data[i * 4 + 1] = cast(u8) (255 - i * 30);
        bitmap.data[i * 4 + 2] = cast(u8) (i * 20);
        bitmap.data[i * 4 + 3] = cast(u8) (200 + i);
    }

    path := sprint("%../../target/window-bitmap-test.png", #filepath);

    assert(S.bitmap_save(*bitmap, path, true));
    decoded: S.Bitmap;

    defer S.deinit(*decoded);
    assert(S.bitmap_load(*decoded, path));
    assert(decoded.width == 3 && decoded.height == 2 && decoded.format == .RGBA8);
    assert(decoded.data.count == bitmap.data.count);
    for byte, i: bitmap.data assert(decoded.data[i] == byte);

    invalid := u8.[0, 1, 2, 3];

    assert(!S.bitmap_load_from_memory(*decoded, invalid));
    empty: [] u8;

    assert(!S.bitmap_load_from_memory(*decoded, empty));
    assert(decoded.width == 3 && decoded.data[3] == 200);
    print("ok\n");
}

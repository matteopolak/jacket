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

    path := sprint("%../../../../target/window-bitmap-test.png", #filepath);

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
    assert(S.bitmap_save(*bitmap, path, false));
    assert(S.bitmap_load(*decoded, path));
    assert(decoded.format == .RGB8 && decoded.data.count == 18);
    for i: 0..5 for channel : 0..2 assert(decoded.data[i * 3 + channel] == bitmap.data[i * 4 + channel]);

    unsupported: S.Bitmap;

    S.bitmap_alloc(*unsupported, 2, 2, .R8);
    defer S.deinit(*unsupported);
    assert(!S.bitmap_save(*unsupported, path));
    // Original two-pixel PGM asset exercises the decoder's single-channel result.
    gray := u8.[80, 53, 10, 50, 32, 49, 10, 50, 53, 53, 10, 17, 99];

    assert(S.bitmap_load_from_memory(*decoded, gray));
    assert(decoded.width == 2 && decoded.height == 1 && decoded.format == .R8);
    assert(decoded.data.count == 2 && decoded.data[0] == 17 && decoded.data[1] == 99);
    print("ok\n");
}

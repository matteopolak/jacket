#import "Basic";
S :: #import "Simp";

main :: () {
    bitmap: S.Bitmap;

    S.bitmap_alloc(*bitmap, 3, 3, .R8);
    defer S.deinit(*bitmap);
    for *byte: bitmap.data byte.* = 100;

    original := bitmap.data.data;

    S.bitmap_build_mipmaps(*bitmap);
    assert(bitmap.mipmap_count == 2 && bitmap.stride == 4 && bitmap.data.count == 16);
    assert(bitmap.data.data != original);

    for row: 0..2 {
        for col: 0..2 assert(bitmap.data[row * 4 + col] == 100);
        assert(bitmap.data[row * 4 + 3] == 0);
    }

    assert(bitmap.data[12] == 100);
    S.deinit(*bitmap);
    S.bitmap_alloc(*bitmap, 4, 2, .RGBA8);

    for pixel: 0..7 {
        bitmap.data[pixel * 4] = 10;
        bitmap.data[pixel * 4 + 1] = 20;
        bitmap.data[pixel * 4 + 2] = 30;
        bitmap.data[pixel * 4 + 3] = 200;
    }

    bitmap.flags |= .IS_SRGB;
    S.bitmap_build_mipmaps(*bitmap);
    assert(bitmap.mipmap_count == 3 && bitmap.data.count == 44);
    assert((bitmap.flags & .IS_SRGB) != 0 && (bitmap.flags & .FREE_DATA_ON_DEINIT) != 0);

    for pixel: 0..10 {
        assert(bitmap.data[pixel * 4] == 10);
        assert(bitmap.data[pixel * 4 + 1] == 20);
        assert(bitmap.data[pixel * 4 + 2] == 30);
        assert(bitmap.data[pixel * 4 + 3] == 200);
    }

    print("ok\n");
}

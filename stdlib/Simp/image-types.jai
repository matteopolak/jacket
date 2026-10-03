Texture_Format :: enum u16 {
    Unknown :: 0;

    R8 :: 1;
    RG8 :: 2;
    RGB8 :: 3;
    RGBA8 :: 4;
    RGBX8;
    BGRA8;
    B5G6R5;
    RGB10A2;
    R16;

    R16F :: 0x10;
    RG16F;
    RGBA16F;
    R32F;
    RG32F;
    RGBA32F;
    RG11B10F;
    RGB9E5F;

    R16I;
    RG16I;
    RGBA16I;
    R32I;
    RG32I;
    RGBA32I;

    BC1 :: 0x100;
    BC3;
    BC4;
    BC5;
    BC6;
    BC7;

    ETC_RGB;
    ETC_RGBA;
    ETC_R;
    ETC_RG;
    ETC_RGB_A1;

    D16;
    D24S8;
    D32F;
    D32FS8;
}

Bitmap :: struct {
    width: s32;
    height: s32;

    mipmap_count: s32 = 1;

    data: [] u8;
    stride: s32;

    flags: enum_flags u32 {
        FREE_DATA_ON_DEINIT :: 0x1;
        IS_SRGB :: 0x2;
    }

    format := Texture_Format.BGRA8;
}

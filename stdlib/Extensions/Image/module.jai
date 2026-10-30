// Decode, encode and transform raster images, in plain Jai (natively, in the playground and
// in -os wasm programs). A jaic extension, not official Jai. See docs/extensions/image.md.
//
// ```jai,no_run
// #import "Extensions/Image";
// #import "File";
//
// file, found := read_entire_file("photo.png");
// image, ok, error := decode_image_as(cast([] u8) file, .RGBA8);
// if !ok { print("%\n", error); return; }
// defer deinit(*image);
// small := resize(image, 64, 64, .BOX);
// png := encode_png(small);
// ```
//
// Every procedure that returns an Image or a []u8 allocates it with context.allocator; the caller
// owns it. Errors are static strings.

// How the bytes of one pixel are laid out. Channels are 8 bits; decoders reduce deeper samples.
Pixel_Format :: enum u8 {
    GRAY8;
    GRAY_ALPHA8;
    RGB8;
    RGBA8;
    BGRA8;
}

// Rows are packed top to bottom with no padding: pixel (x, y) starts at
// (y * width + x) * bytes_per_pixel(format).
Image :: struct {
    width: s32;
    height: s32;
    format: Pixel_Format;
    pixels: [] u8;
}

Image_File_Format :: enum u8 {
    UNKNOWN;
    PNG;
    JPEG;
    QOI;
    BMP;
    PNM; // PPM (P6), PGM (P5) and their text forms (P3, P2)
}

// How resize samples the source; see resize.
Resize_Filter :: enum u8 {
    NEAREST;
    BILINEAR;
    BOX;
}

// Decoders refuse images with more pixels than this, before allocating them.
DEFAULT_MAX_PIXELS :: 1 << 26;

bytes_per_pixel :: (format: Pixel_Format) -> s64 {
    if format == {
        case .GRAY8; return 1;
        case .GRAY_ALPHA8; return 2;
        case .RGB8; return 3;
        case .RGBA8; return 4;
        case .BGRA8; return 4;
    }

    return 0; // not a Pixel_Format
}

has_alpha :: (format: Pixel_Format) -> bool {
    return format == .GRAY_ALPHA8 || format == .RGBA8 || format == .BGRA8;
}

// A zeroed image of the given size; empty when the size or format is invalid.
make_image :: (width: s32, height: s32, format: Pixel_Format) -> Image {
    image: Image;
    if width <= 0 || height <= 0 || bytes_per_pixel(format) == 0 return image;

    image.width = width;
    image.height = height;
    image.format = format;
    image.pixels = NewArray(cast(s64) width * height * bytes_per_pixel(format), u8);

    return image;
}

deinit :: (image: *Image) {
    array_free(image.pixels);
    image.* = .{};
}

// Which file format the bytes start like, from their signature alone.
detect_image_format :: (data: [] u8) -> Image_File_Format {
    starts :: (data: [] u8, prefix: string) -> bool {
        if data.count < prefix.count return false;
        for 0..prefix.count - 1 if data[it] != prefix[it] return false;
        return true;
    }

    if starts(data, "\x89PNG\r\n\x1a\n") return .PNG;
    if starts(data, "\xff\xd8\xff") return .JPEG;
    if starts(data, "qoif") return .QOI;
    if starts(data, "BM") return .BMP;
    if data.count >= 2 && data[0] == #char "P" && data[1] >= #char "2" && data[1] <= #char "6" && data[1] != #char "4" return .PNM;

    return .UNKNOWN;
}

// Decodes a PNG, JPEG, QOI, BMP or PNM file in the format it is stored in (the nearest
// Pixel_Format to it).
decode_image :: (data: [] u8, max_pixels: s64 = DEFAULT_MAX_PIXELS) -> image: Image, ok: bool, error: string {
    if detect_image_format(data) == {
        case .PNG; return decode_png(data, max_pixels);
        case .JPEG; return decode_jpeg(data, max_pixels);
        case .QOI; return decode_qoi(data, max_pixels);
        case .BMP; return decode_bmp(data, max_pixels);
        case .PNM; return decode_pnm(data, max_pixels);
    }

    return .{}, false, "not a PNG, JPEG, QOI, BMP or PNM image";
}

// decode_image, then convert to `format` if the file holds something else.
decode_image_as :: (data: [] u8, format: Pixel_Format, max_pixels: s64 = DEFAULT_MAX_PIXELS) -> image: Image, ok: bool, error: string {
    image, ok, error := decode_image(data, max_pixels);
    if !ok || image.format == format return image, ok, error;

    converted := convert(image, format);

    deinit(*image);
    if converted.pixels.count == 0 return .{}, false, "not a pixel format";

    return converted, true, "";
}

#scope_module
#import "Basic";
#import "Extensions/Deflate";
#import "Extensions/SIMD";

#scope_export
#load "convert.jai";
#load "transform.jai";
#load "png.jai";
#load "jpeg.jai";
#load "qoi.jai";
#load "bmp.jai";
#load "pnm.jai";

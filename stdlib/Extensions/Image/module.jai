// Decodes, encodes and transforms raster images (PNG, JPEG, QOI, BMP and PNM) in plain Jai, on every
// target including wasm.
//
// `decode_image` reads any supported file; `encode_png`, `encode_qoi`, `encode_bmp` and `encode_ppm`
// write one. `resize`, `crop`, `convert` and the flips and rotation change an `Image`. Procedures that
// return an `Image` or a `[] u8` allocate with `context.allocator`; free the image with `deinit`. Errors
// are static strings.
//
// ```jai
// #import "Extensions/Image";
//
// image := make_image(2, 2, .RGBA8);
// defer deinit(*image);
// image.pixels[0] = 255; // top left pixel: red
// image.pixels[3] = 255;
//
// png := encode_png(image);
// defer free(png.data);
//
// decoded, ok, error := decode_image(png);
// defer deinit(*decoded);
// assert(ok && decoded.width == 2 && decoded.pixels[0] == 255);
// ```

// How the bytes of one pixel are laid out. Channels are 8 bits; decoders reduce deeper samples.
Pixel_Format :: enum u8 {
    GRAY8;
    GRAY_ALPHA8;
    RGB8;
    RGBA8;
    BGRA8;
}

// A decoded image. Rows are packed top to bottom with no padding: pixel `(x, y)` starts at
// `(y * width + x) * bytes_per_pixel(format)` in `pixels`.
//
// ```jai
// image := make_image(3, 2, .RGB8);
// defer deinit(*image);
// assert(image.pixels.count == 3 * 2 * 3);
//
// x, y := 2, 1;
// offset := (y * image.width + x) * bytes_per_pixel(image.format);
// image.pixels[offset] = 200; // red channel of the bottom right pixel
// ```
Image :: struct {
    width: s32;
    height: s32;
    format: Pixel_Format;
    pixels: [] u8;
}

// A file format `detect_image_format` can name. `PNM` covers PPM (P6), PGM (P5) and their text forms (P3, P2).
Image_File_Format :: enum u8 {
    UNKNOWN;
    PNG;
    JPEG;
    QOI;
    BMP;
    PNM; // PPM (P6), PGM (P5) and their text forms (P3, P2)
}

// What `read_image_info` learns from a file's header.
Image_Info :: struct {
    format: Image_File_Format;
    width: s32;
    height: s32;
    channels: s32; // samples per pixel as stored: 1 gray, 2 gray and alpha, 3 colour, 4 colour and alpha or CMYK
}

// How `resize` samples the source: `NEAREST` takes the closest pixel, `BILINEAR` blends the four around a
// point, `BOX` averages the whole area under each output pixel.
Resize_Filter :: enum u8 {
    NEAREST;
    BILINEAR;
    BOX;
}

// Decoders refuse images with more pixels than this, before allocating them.
DEFAULT_MAX_PIXELS :: 1 << 26;

// The size of one pixel in bytes.
//
// ```jai
// assert(bytes_per_pixel(.GRAY8) == 1);
// assert(bytes_per_pixel(.RGBA8) == 4);
// ```
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

// Whether the format has an alpha channel.
//
// ```jai
// assert(has_alpha(.RGBA8));
// assert(!has_alpha(.RGB8));
// ```
has_alpha :: (format: Pixel_Format) -> bool {
    return format == .GRAY_ALPHA8 || format == .RGBA8 || format == .BGRA8;
}

// A zeroed image of the given size, or an empty one when the size or format is invalid. Free it with `deinit`.
//
// ```jai
// image := make_image(16, 8, .RGBA8);
// defer deinit(*image);
// assert(image.width == 16 && image.pixels.count == 16 * 8 * 4);
// assert(make_image(0, 8, .RGBA8).pixels.count == 0);
// ```
make_image :: (width: s32, height: s32, format: Pixel_Format) -> Image {
    image: Image;
    if width <= 0 || height <= 0 || bytes_per_pixel(format) == 0 return image;

    image.width = width;
    image.height = height;
    image.format = format;
    image.pixels = NewArray(cast(s64) width * height * bytes_per_pixel(format), u8);

    return image;
}

// Frees the pixels and empties the image.
deinit :: (image: *Image) {
    array_free(image.pixels);
    image.* = .{};
}

// Which file format the bytes start like. It looks at the signature only, not the rest of the file.
//
// ```jai
// png := encode_png(make_image(1, 1, .RGB8));
// defer free(png.data);
// assert(detect_image_format(png) == .PNG);
// assert(detect_image_format(cast([] u8) "hello") == .UNKNOWN);
// ```
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

// Decodes a PNG, JPEG, QOI, BMP or PNM file into the nearest pixel format to the one it's stored in.
// Returns the image, whether it worked and, if not, why. Files with more than `max_pixels` pixels are
// refused before anything is allocated.
//
// ```jai
// qoi := encode_qoi(make_image(4, 4, .RGB8));
// defer free(qoi.data);
//
// image, ok, error := decode_image(qoi);
// defer deinit(*image);
// assert(ok && image.format == .RGB8);
//
// _, ok, error = decode_image(cast([] u8) "not an image");
// assert(!ok);
// ```
decode_image :: (data: [] u8, max_pixels: s64 = DEFAULT_MAX_PIXELS) -> image: Image, ok: bool, error: string {
    if detect_image_format(data) == {
        case .PNG; return decode_png(data, max_pixels);
        case .JPEG; return decode_jpeg(data, max_pixels);
        case .QOI; return decode_qoi(data, max_pixels);
        case .BMP; return decode_bmp(data, max_pixels);
        case .PNM; return decode_pnm(data, max_pixels);
        case; // Other members deliberately do nothing.
    }

    return .{}, false, "not a PNG, JPEG, QOI, BMP or PNM image";
}

// Reads the format, size and channel count from the header of a PNG or JPEG without decoding the
// pixels or allocating. It checks the header only: later chunks, tables and image data may still
// be damaged, so use `decode_image` when you need the pixels. Files with more than `max_pixels` pixels
// are refused, as in `decode_image`. Other formats are reported as not supported.
//
// ```jai
// png := encode_png(make_image(5, 3, .RGBA8));
// defer free(png.data);
//
// info, ok, error := read_image_info(png);
// assert(ok && info.format == .PNG && info.width == 5 && info.height == 3 && info.channels == 4);
//
// _, ok, error = read_image_info(cast([] u8) "not an image");
// assert(!ok);
// ```
read_image_info :: (data: [] u8, max_pixels: s64 = DEFAULT_MAX_PIXELS) -> info: Image_Info, ok: bool, error: string {
    format := detect_image_format(data);
    if format == .PNG return png_read_info(data, max_pixels);
    if format == .JPEG return jpeg_read_info(data, max_pixels);
    if format == .UNKNOWN return .{}, false, "not a PNG, JPEG, QOI, BMP or PNM image";

    return .{}, false, "reading the header of this format is not supported";
}

// Like `decode_image`, then converts to `format` if the file holds something else.
//
// ```jai
// qoi := encode_qoi(make_image(4, 4, .RGB8));
// defer free(qoi.data);
//
// image, ok := decode_image_as(qoi, .RGBA8);
// defer deinit(*image);
// assert(ok && image.format == .RGBA8 && image.pixels[3] == 255);
// ```
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
Byte_Search :: #import "Extensions/Byte_Search";

#scope_export
#load "convert.jai";
#load "transform.jai";
#load "png.jai";
#load "jpeg.jai";
#load "qoi.jai";
#load "bmp.jai";
#load "pnm.jai";
#load "ico.jai";

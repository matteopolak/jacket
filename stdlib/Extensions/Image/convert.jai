// jailint: allow-file(magic_number) // the Rec. 709 luma weights
// Pixel format conversion. One loop per (source, destination) pair, instantiated from a single
// template so each is straight-line code over the pixels.

// Rec. 709 luma in 16-bit fixed point; the weights sum to 65536 so white stays white.
convert_luma :: inline (r: s32, g: s32, b: s32) -> u8 {
    return cast,trunc(u8) ((13933 * r + 46871 * g + 4732 * b + 32768) >> 16);
}

// The same pixels in another format. Gray is made from colour with Rec. 709 luma, colour from
// gray by repeating it, alpha is added as 255 and dropped without compositing. Always returns a
// new image (a copy when the format is already right); an invalid image gives an empty one.
convert :: (image: Image, format: Pixel_Format) -> Image {
    if !image_is_valid(image) || bytes_per_pixel(format) == 0 return .{};

    result := make_image(image.width, image.height, format);
    count := cast(s64) image.width * image.height;

    if image.format == format {
        memcpy(result.pixels.data, image.pixels.data, image.pixels.count);
        return result;
    }

    src := image.pixels.data;
    dst := result.pixels.data;

    if #complete image.format == {
        case .GRAY8;
            if #complete format == {
                case .GRAY8;
                case .GRAY_ALPHA8; convert_pixels(.GRAY8, .GRAY_ALPHA8, src, dst, count);
                case .RGB8; convert_pixels(.GRAY8, .RGB8, src, dst, count);
                case .RGBA8; convert_pixels(.GRAY8, .RGBA8, src, dst, count);
                case .BGRA8; convert_pixels(.GRAY8, .BGRA8, src, dst, count);
            }
        case .GRAY_ALPHA8;
            if #complete format == {
                case .GRAY8; convert_pixels(.GRAY_ALPHA8, .GRAY8, src, dst, count);
                case .GRAY_ALPHA8;
                case .RGB8; convert_pixels(.GRAY_ALPHA8, .RGB8, src, dst, count);
                case .RGBA8; convert_pixels(.GRAY_ALPHA8, .RGBA8, src, dst, count);
                case .BGRA8; convert_pixels(.GRAY_ALPHA8, .BGRA8, src, dst, count);
            }
        case .RGB8;
            if #complete format == {
                case .GRAY8; convert_pixels(.RGB8, .GRAY8, src, dst, count);
                case .GRAY_ALPHA8; convert_pixels(.RGB8, .GRAY_ALPHA8, src, dst, count);
                case .RGB8;
                case .RGBA8; convert_pixels(.RGB8, .RGBA8, src, dst, count);
                case .BGRA8; convert_pixels(.RGB8, .BGRA8, src, dst, count);
            }
        case .RGBA8;
            if #complete format == {
                case .GRAY8; convert_pixels(.RGBA8, .GRAY8, src, dst, count);
                case .GRAY_ALPHA8; convert_pixels(.RGBA8, .GRAY_ALPHA8, src, dst, count);
                case .RGB8; convert_pixels(.RGBA8, .RGB8, src, dst, count);
                case .RGBA8;
                case .BGRA8; convert_pixels(.RGBA8, .BGRA8, src, dst, count);
            }
        case .BGRA8;
            if #complete format == {
                case .GRAY8; convert_pixels(.BGRA8, .GRAY8, src, dst, count);
                case .GRAY_ALPHA8; convert_pixels(.BGRA8, .GRAY_ALPHA8, src, dst, count);
                case .RGB8; convert_pixels(.BGRA8, .RGB8, src, dst, count);
                case .RGBA8; convert_pixels(.BGRA8, .RGBA8, src, dst, count);
                case .BGRA8;
            }
    }

    return result;
}

#scope_module

// data[from ..< from + count] as a view, with no bounds check of its own.
byte_range :: (data: [] u8, from: s64, count: s64) -> [] u8 {
    result: [] u8;
    result.data = data.data + from;
    result.count = count;
    return result;
}

empty_bytes :: () -> [] u8 {
    empty: [] u8;
    return empty;
}

// Whether the pixel array holds exactly width * height pixels of the format.
image_is_valid :: (image: Image) -> bool {
    if image.width <= 0 || image.height <= 0 return false;
    bpp := bytes_per_pixel(image.format);
    return bpp != 0 && image.pixels.count == cast(s64) image.width * image.height * bpp;
}

// Whether a decoder may allocate width * height pixels of this format.
image_size_ok :: (width: s64, height: s64, max_pixels: s64) -> bool {
    if width <= 0 || height <= 0 || width > 0x7fffffff || height > 0x7fffffff return false;
    return width * height <= max_pixels && width * height <= 0x7fffffff;
}

#scope_file

convert_pixels :: ($from: Pixel_Format, $to: Pixel_Format, src: *u8, dst: *u8, count: s64) {
    SB :: #run bytes_per_pixel(from);
    DB :: #run bytes_per_pixel(to);

    for i: 0..count - 1 {
        s := src + i * SB;
        d := dst + i * DB;

        r, g, b, a: u8;

        #if from == .GRAY8 {
            r = s[0];
            g = r;
            b = r;
            a = 255;
        } else #if from == .GRAY_ALPHA8 {
            r = s[0];
            g = r;
            b = r;
            a = s[1];
        } else #if from == .RGB8 {
            r = s[0];
            g = s[1];
            b = s[2];
            a = 255;
        } else #if from == .RGBA8 {
            r = s[0];
            g = s[1];
            b = s[2];
            a = s[3];
        } else {
            r = s[2];
            g = s[1];
            b = s[0];
            a = s[3];
        }

        #if to == .GRAY8 {
            #if from == .GRAY8 || from == .GRAY_ALPHA8 {
                d[0] = r;
            } else {
                d[0] = convert_luma(r, g, b);
            }
        } else #if to == .GRAY_ALPHA8 {
            #if from == .GRAY8 || from == .GRAY_ALPHA8 {
                d[0] = r;
            } else {
                d[0] = convert_luma(r, g, b);
            }

            d[1] = a;
        } else #if to == .RGB8 {
            d[0] = r;
            d[1] = g;
            d[2] = b;
        } else #if to == .RGBA8 {
            d[0] = r;
            d[1] = g;
            d[2] = b;
            d[3] = a;
        } else {
            d[0] = b;
            d[1] = g;
            d[2] = r;
            d[3] = a;
        }
    }
}

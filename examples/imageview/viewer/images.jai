// File signatures win over extensions except TGA, which has no fixed signature.
format_name :: (bytes: [] u8, path: string) -> string {
    detected := Image.detect_image_format(bytes);
    if detected != .UNKNOWN return tprint("%", detected);

    if bytes.count >= 6 {
        if bytes[0] == 0 && bytes[1] == 0 && bytes[2] == 1 && bytes[3] == 0 return "ICO";
        if bytes[0] == 71 && bytes[1] == 73 && bytes[2] == 70 return "GIF";
    }

    lower := to_lower_copy(path);

    defer free(lower);
    if ends_with(lower, ".tga") return "TGA";

    return "UNKNOWN";
}

Picture :: struct {
    image: Image.Image;
    frames: [] u8; // animated GIF: contiguous RGBA frames, includes frame zero
    delays: [] s32; // milliseconds
    format: string;
    bits: s32 = 8; // source channel depth, decoded samples are always 8 bits
    file_bytes: s64;
    error: string;
}

release_picture :: (picture: *Picture) {
    Image.deinit(*picture.image);
    array_free(picture.frames);
    array_free(picture.delays);
    free(picture.format);
    free(picture.error);
    picture.* = .{};
}

decode_picture :: (path: string, animation: bool = false) -> Picture {
    result: Picture;
    bytes, read_ok := read_entire_file(path);

    if !read_ok {
        result.error = copy_string("cannot read file");
        return result;
    }

    defer free(bytes);
    result.file_bytes = bytes.count;
    data := cast([] u8) bytes;

    result.format = copy_string(format_name(data, path));
    if result.format == "ICO" {
        image, ok, error := Image.decode_ico(data);
        result.image = image;
        if !ok result.error = copy_string(error);
        return result;
    }

    // Plain-Jai codecs cover PNG/JPEG/BMP/PNM/QOI, the native fallback adds TGA/GIF.
    if Image.detect_image_format(data) != .UNKNOWN {
        image, ok, error := Image.decode_image_as(data, .RGBA8);

        result.image = image;
        if !ok result.error = copy_string(error);
        if bytes.count <= 0x7fffffff && Load.stbi_is_16_bit_from_memory(bytes.data, cast(s32) bytes.count) != 0 result.bits = 16;

        return result;
    }

    if bytes.count > 0x7fffffff {
        result.error = copy_string("file too large for native decoder");
        return result;
    }

    w, h, channels: s32;
    if Load.stbi_info_from_memory(bytes.data, cast(s32) bytes.count, *w, *h, *channels) == 0 || w <= 0 || h <= 0 || cast(s64) w * h > Image.DEFAULT_MAX_PIXELS {
        result.error = copy_string("invalid or oversized image");
        return result;
    }

    pixels := Load.stbi_load_from_memory(bytes.data, cast(s32) bytes.count, *w, *h, *channels, 4);
    if !pixels {
        result.error = copy_string("native decode failed");
        return result;
    }

    result.image = Image.make_image(w, h, .RGBA8);
    memcpy(result.image.pixels.data, pixels, result.image.pixels.count);
    Load.stbi_image_free(pixels);

    if animation && result.format == "GIF" {
        delays: *s32;
        count: s32;
        frames := Load.stbi_load_gif_from_memory(bytes.data, cast(s32) bytes.count, *delays, *w, *h, *count, *channels, 4);

        if frames && count > 0 {
            result.frames = NewArray(cast(s64) w * h * 4 * count, u8);
            result.delays = NewArray(count, s32);
            memcpy(result.frames.data, frames, result.frames.count);
            memcpy(result.delays.data, delays, count * size_of(s32));
        }

        if frames Load.stbi_image_free(frames);
        if delays Load.stbi_image_free(delays);
    }

    return result;
}

thumbnail_size :: (w: s32, h: s32, edge: s32) -> s32, s32 {
    if w <= 0 || h <= 0 || edge <= 0 return 0, 0;
    longest := max(w, h);
    if longest <= edge return w, h;
    return cast(s32) max(1, cast(s64) w * edge / longest), cast(s32) max(1, cast(s64) h * edge / longest);
}

thumbnail :: (image: Image.Image, edge: s32 = 112) -> Image.Image {
    w, h := thumbnail_size(image.width, image.height, edge);
    if w == 0 return .{};

    result := Image.make_image(w, h, .RGBA8);
    if Scale.stbir_resize_uint8_srgb(image.pixels.data, image.width, image.height, image.width * 4, result.pixels.data, w, h, w * 4, 4, 3, .{}) == 0 Image.deinit(*result);

    return result;
}

export_picture :: (image: Image.Image, path: string) -> bool {
    if image.pixels.count == 0 return false;

    name := temp_c_string(path);
    if ends_with(path, ".png") return Write.stbi_write_png(name, image.width, image.height, 4, image.pixels.data, image.width * 4) != 0;
    if ends_with(path, ".jpg") || ends_with(path, ".jpeg") return Write.stbi_write_jpg(name, image.width, image.height, 4, image.pixels.data, 95) != 0;

    return false;
}

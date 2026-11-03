// Decode one ICO directory entry (largest area by default) into owned RGBA8 pixels. Supports
// PNG payloads and uncompressed 24/32-bit DIBs with an AND mask. Free with deinit.
// Unsupported palette/compressed DIBs fail explicitly; entry offsets are checked before access.
//
// ```jai,skip_on(.WASM, reason = "Ico_File resizes with a native library")
// Icons :: #import "Ico_File";
// bytes := Icons.create_ico_file(1, 1, 4, u8.[0, 0, 255, 255].data);
// defer free(bytes);
// image, ok := decode_ico(cast([] u8) bytes);
// defer deinit(*image);
// assert(ok && image.width == 256);
// ```
decode_ico :: (data: [] u8, entry: s64 = -1) -> image: Image, ok: bool, error: string {
    if data.count < 6 return .{}, false, "truncated ICO header";
    if ico_word(data, 0) != 0 || ico_word(data, 2) != 1 return .{}, false, "not an ICO";

    count := ico_word(data, 4);
    if count == 0 || data.count < 6 + count * 16 return .{}, false, "truncated ICO directory";

    chosen := entry;
    if chosen < 0 {
        best := 0;

        for i: 0..count - 1 {
            at := 6 + 16 * i;
            w := ifx data[at] == 0 then 256 else cast(int) data[at];
            h := ifx data[at + 1] == 0 then 256 else cast(int) data[at + 1];

            if w * h > best {
                best = w * h;
                chosen = i;
            }
        }
    }

    if chosen >= count return .{}, false, "ICO entry out of range";

    at := 6 + chosen * 16;
    length := ico_dword(data, at + 8);
    offset := ico_dword(data, at + 12);

    if offset > data.count || length > data.count - offset return .{}, false, "ICO payload out of range";

    payload := [] u8.{ length, data.data + offset };
    if detect_image_format(payload) == .PNG return decode_image_as(payload, .RGBA8, 256 * 256);
    if payload.count < 40 || ico_dword(payload, 0) != 40 return .{}, false, "unsupported ICO DIB header";

    w := ico_dword(payload, 4);
    doubled := ico_dword(payload, 8);
    bits := ico_word(payload, 14);

    if w < 1 || w > 256 || doubled < 2 || doubled > 512 || doubled % 2 != 0 return .{}, false, "invalid ICO dimensions";
    if ico_word(payload, 12) != 1 || (bits != 24 && bits != 32) || ico_dword(payload, 16) != 0 return .{}, false, "unsupported ICO pixel encoding";

    h := doubled / 2;
    stride := ((w * bits + 31) / 32) * 4;
    mask_stride := ((w + 31) / 32) * 4;
    mask_at := 40 + stride * h;

    if payload.count < mask_at + mask_stride * h return .{}, false, "truncated ICO pixels or mask";

    image = make_image(cast(s32) w, cast(s32) h, .RGBA8);
    any_alpha := false;
    if bits == 32 for y: 0..h - 1 for x : 0..w - 1 if payload[40 + y * stride + x * 4 + 3] != 0 any_alpha = true;

    for y: 0..h - 1 for x : 0..w - 1 {
        source := 40 + (h - 1 - y) * stride + x * (bits / 8);
        target := (y * w + x) * 4;
        image.pixels[target] = payload[source + 2];
        image.pixels[target + 1] = payload[source + 1];
        image.pixels[target + 2] = payload[source];
        alpha: u8 = 255;
        if bits == 32 && any_alpha alpha = payload[source + 3];
        if (payload[mask_at + (h - 1 - y) * mask_stride + x / 8] & (128 >> (x % 8))) != 0 alpha = 0;

        image.pixels[target + 3] = alpha;
    }

    return image, true, "";
}

#scope_file

ico_word :: (data: [] u8, at: s64) -> s64 {
    return cast(s64) data[at] | ((cast(s64) data[at + 1]) << 8);
}

ico_dword :: (data: [] u8, at: s64) -> s64 {
    return ico_word(data, at) | (ico_word(data, at + 2) << 16);
}

// test: only(.LINUX,.MACOS,.WASM) args("$DIR/ico-fixtures","$TEMP")
#import "Basic";
I :: #import "Ico_File";
F :: #import "File";
#load "../../Extensions/test-support/counting-allocator.jai";

errors: int;

capture :: (message: string, data: *void, info: Log_Info) {
    assert(message.count > 0 && data == null && (info.common_flags & .ERROR) != 0);
    errors += 1;
}

u32_at :: (bytes: string, offset: int) -> u32 {
    value: u32;
    for byte: 0..3 value |= (cast(u32) bytes[offset + byte]) << (8 * byte);
    return value;
}

check_rgba :: (folder: string, name: string, with_alpha: bool = true) {
    path := sprint("%/%", folder, name);

    defer free(path);
    encoded := I.create_ico_file_from_bitmap_filename(path);

    defer free(encoded);
    assert(encoded.count >= 86 && encoded[4] == 5 && encoded[5] == 0);
    sizes: [5] int = .[256, 128, 48, 32, 16];
    cursor := 86;

    for size, index: sizes {
        directory := 6 + 16 * index;

        assert(u32_at(encoded, directory + 12) == cursor);
        length := u32_at(encoded, directory + 8);

        // The existing wire layout reserves two pixel planes and two mask planes.
        assert(length == 40 + size * size * 8 + size * size / 2);
        assert(cursor + length <= encoded.count);
        color := cursor + 40;
        mask := color + size * size * 4;
        mask_stride := ((size + 31) / 32) * 4;

        for row: 0..size - 1 for column : 0..mask_stride * 8 - 1 {
            bit := (encoded[mask + row * mask_stride + column / 8] & (128 >> (column % 8))) != 0;
            transparent := column < size && encoded[color + (row * size + column) * 4 + 3] == 0;
            assert(bit == transparent);
        }

        cursor += length;
    }

    assert(encoded.count == cursor);
    offset := cast(int) u32_at(encoded, 18 + 16 * 4) + 40;

    // Independent same-size bottom-up BGRA oracle; includes alpha 0, 128 and 255.
    for row: 0..15 for column : 0..15 {
        at := offset + (row * 16 + column) * 4;
        // Alpha-weighted resizing canonicalizes fully transparent colors to black.
        if with_alpha && column == 0 assert(encoded[at] == 0 && encoded[at + 1] == 0 && encoded[at + 2] == 0);
        else assert(encoded[at] == row * 10 && encoded[at + 1] == column * 10 && encoded[at + 2] == 60 + row);

        alpha := cast(u8) 255;
        if with_alpha && column == 0 alpha = 0;
        if with_alpha && column == 1 alpha = 128;

        assert(encoded[at + 3] == alpha);
    }
}

check_ownership :: (path: string, accepted: bool) {
    tally: Tally;

    {
        push_allocator(tally_proc, *tally);
        encoded := I.create_ico_file_from_bitmap_filename(path);
        defer free(encoded);
        assert((encoded.count > 0) == accepted);
    }

    if tally.outstanding != 0 print("ownership %: % outstanding after % allocations\n", path, tally.outstanding, tally.allocations);

    assert(tally.outstanding == 0);
}

main :: () {
    args := get_command_line_arguments();

    // args[1] holds the input bitmaps (read-only, in the repository); args[2] is a folder to write to.
    assert(args.count == 3);
    bitmap := sprint("%/bitmap-ü.bmp", args[1]);

    defer free(bitmap);
    encoded := I.create_ico_file_from_bitmap_filename(bitmap);

    defer free(encoded);
    assert(encoded[4] == 5);
    assert(encoded[126] == 30 && encoded[127] == 20 && encoded[128] == 10 && encoded[129] == 255);
    path := sprint("%/filename.ico", args[2]);

    defer free(path);
    assert(F.write_entire_file(path, encoded));
    gradient_path := sprint("%/gradient.bmp", args[1]);

    defer free(gradient_path);
    gradient := I.create_ico_file_from_bitmap_filename(gradient_path);

    defer free(gradient);
    // Same-size scaling must keep the converted pixels and their orientation exactly.
    offset: u32;
    for byte: 0..3 offset |= (cast(u32) gradient[18 + 16 * 4 + byte]) << (8 * byte);

    expected_path := sprint("%/gradient-resized.bgra", args[1]);

    defer free(expected_path);
    expected, expected_ok := F.read_entire_file(expected_path);

    assert(expected_ok && expected.count == 16 * 16 * 4);
    defer free(expected);
    for byte, index: expected assert(gradient[offset + 40 + index] == byte);

    check_rgba(args[1], "rgba.png");
    check_rgba(args[1], "palette.png");
    rgba_path := sprint("%/rgba.png", args[1]);

    defer free(rgba_path);
    check_ownership(rgba_path, true);
    check_rgba(args[1], "rgb.ppm", false);
    #if OS == .WASM check_rgba(args[1], "rgba.qoi");
    jpeg_path := sprint("%/rgb.jpg", args[1]);

    defer free(jpeg_path);
    jpeg := I.create_ico_file_from_bitmap_filename(jpeg_path);

    defer free(jpeg);
    assert(jpeg.count > 86 && jpeg[4] == 5);
    #if OS != .WASM check_rgba(args[1], "rgba.tga");

    missing := sprint("%/absent-ü.bmp", args[1]);
    malformed := sprint("%/invalid.bmp", args[2]);

    defer free(missing);
    defer free(malformed);
    assert(F.write_entire_file(malformed, "not a bitmap"));

    push_context {
        context.logger = capture;
        context.logger_data = null;
        assert(I.create_ico_file_from_bitmap_filename(missing).count == 0);
        assert(I.create_ico_file_from_bitmap_filename(malformed).count == 0);
    }

    rejected: [4] string = .["gray.pgm", "gray-alpha.png", "truncated.png", "oversized.png"];

    push_context {
        context.logger = capture;
        context.logger_data = null;

        for rejected {
            rejected_path := sprint("%/%", args[1], it);
            defer free(rejected_path);
            check_ownership(rejected_path, false);
        }
    }

    assert(errors == 6);
    print("ok\n");
}

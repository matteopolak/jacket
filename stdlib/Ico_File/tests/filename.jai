#import "Basic";
I :: #import "Ico_File";
F :: #import "File";

errors: int;

capture :: (message: string, data: *void, info: Log_Info) {
    assert(message.count > 0 && data == null && (info.common_flags & .ERROR) != 0);
    errors += 1;
}

main :: () {
    args := get_command_line_arguments();

    assert(args.count == 2);
    bitmap := sprint("%/bitmap-ü.bmp", args[1]);

    defer free(bitmap);
    encoded := I.create_ico_file_from_bitmap_filename(bitmap);

    defer free(encoded);
    assert(encoded[4] == 5);
    assert(encoded[126] == 30 && encoded[127] == 20 && encoded[128] == 10 && encoded[129] == 255);
    path := sprint("%/filename.ico", args[1]);

    defer free(path);
    assert(F.write_entire_file(path, encoded));
    gradient_path := sprint("%/gradient.bmp", args[1]);

    defer free(gradient_path);
    gradient := I.create_ico_file_from_bitmap_filename(gradient_path);

    defer free(gradient);
    // Compare conversion/orientation against an independent native SDK call.
    offset: u32;
    for byte: 0..3 offset |= (cast(u32) gradient[18 + 16 * 4 + byte]) << (8 * byte);

    expected_path := sprint("%/gradient-resized.bgra", args[1]);

    defer free(expected_path);
    expected, expected_ok := F.read_entire_file(expected_path);

    assert(expected_ok && expected.count == 16 * 16 * 4);
    defer free(expected);
    for byte, index: expected assert(gradient[offset + 40 + index] == byte);

    missing := sprint("%/absent-ü.bmp", args[1]);
    malformed := sprint("%/invalid.bmp", args[1]);

    defer free(missing);
    defer free(malformed);
    assert(F.write_entire_file(malformed, "not a bitmap"));

    push_context {
        context.logger = capture;
        context.logger_data = null;
        assert(I.create_ico_file_from_bitmap_filename(missing).count == 0);
        assert(I.create_ico_file_from_bitmap_filename(malformed).count == 0);
    }

    assert(errors == 2);
    print("ok\n");
}

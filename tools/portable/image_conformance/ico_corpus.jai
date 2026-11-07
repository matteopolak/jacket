#import "Basic";
#import "File";
Image :: #import "Extensions/Image";
#load "ico_cases.jai";

failures: int;

check :: (ok: bool, name: string, reason: string) {
    if !ok {
        print("FAIL %: %\n", name, reason);
        failures += 1;
    }
}

main :: () {
    arguments := get_command_line_arguments();

    assert(arguments.count == 3);

    for CASES {
        defer print("DONE %\n", it.name);
        path := sprint("%/%", arguments[1], it.file);

        defer free(path);
        data, read_ok := read_entire_file(path);

        defer free(data);
        check(read_ok, it.name, "read input");
        if !read_ok continue;

        image, ok, error := Image.decode_ico(cast([] u8) data, it.entry);

        defer Image.deinit(*image);
        check(ok == (it.required == 1), it.name, "acceptance policy");
        print("RESULT % % %\n", it.name, ok, error);

        if !ok {
            check(image.pixels.count == 0, it.name, "empty rejected image");
            continue;
        }

        check(image.width == it.width && image.height == it.height && image.format == .RGBA8, it.name, "dimensions/format");
        expected_path := sprint("%/%.rgba", arguments[2], it.name);

        defer free(expected_path);
        expected, expected_ok := read_entire_file(expected_path);

        defer free(expected);
        check(expected_ok && expected.count == image.pixels.count, it.name, "independent oracle length");

        if expected_ok && expected.count == image.pixels.count {
            same := true;
            for byte, index: image.pixels if byte != expected[index] same = false;
            check(same, it.name, "independent Pillow pixels");
        }
    }

    print("ICO corpus: % cases, % failures\n", CASES.count, failures);
    assert(failures == 0);
}

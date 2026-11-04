// test: args("$TEMP")
#import "Basic";
#import "File";
Icons :: #import "Ico_File";

// Writes asymmetric alpha cases for the independent native Win32 consumer.
main :: () {
    arguments := get_command_line_arguments();

    assert(arguments.count == 2);

    for name: string.["clear", "mixed", "opaque", "faint"] {
        size: s32 = 16;
        if name == "faint" size = 256;

        pixels := NewArray(size * size * 4, u8);

        defer array_free(pixels);

        for row: 0..15 for column : 0..15 {
            offset := (row * 16 + column) * 4;
            pixels[offset] = cast(u8) (10 + row * 4);
            pixels[offset + 1] = cast(u8) (20 + column * 4);
            pixels[offset + 2] = cast(u8) (30 + row + column);
            pixels[offset + 3] = 0;
            if name == "opaque" pixels[offset + 3] = 255;

            if name == "mixed" {
                if column >= 5 pixels[offset + 3] = 128;
                if column >= 10 pixels[offset + 3] = 255;

                if row >= 8 {
                    pixels[offset + 3] = 255;
                    if column >= 5 pixels[offset + 3] = 0;
                    if column >= 10 pixels[offset + 3] = 128;
                }
            }
        }

        if name == "faint" {
            memset(pixels.data, 0, pixels.count);
            pixels[0] = 120;
            pixels[1] = 80;
            pixels[2] = 40;
            pixels[3] = 1;
        }

        bytes := Icons.create_ico_file(size, size, 4, pixels.data);

        defer free(bytes);
        path := sprint("%/gdi-%.ico", arguments[1], name);

        defer free(path);
        assert(write_entire_file(path, bytes));
    }

    print("ok\n");
}

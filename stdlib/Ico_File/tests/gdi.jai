// test: args("$TEMP")
#import "Basic";
#import "File";
Icons :: #import "Ico_File";

// Writes asymmetric alpha cases for the independent native Win32 consumer.
main :: () {
    arguments := get_command_line_arguments();

    assert(arguments.count == 2);
    pixels: [16 * 16 * 4] u8;

    for name: string.["clear", "mixed", "opaque"] {
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

        bytes := Icons.create_ico_file(16, 16, 4, pixels.data);

        defer free(bytes);
        path := sprint("%/gdi-%.ico", arguments[1], name);

        defer free(path);
        assert(write_entire_file(path, bytes));
    }

    print("ok\n");
}

// Black-box read/write adapter using only the public README calls of sjorsdonkers/toml-jai.
// Compile and execute only through tools/bench Sandbox. Never inspect its implementation.
#import "Basic";
F :: #import "File";
P :: #import "POSIX";
T :: #import "toml-jai";

main :: () {
    input := F.File.{ P.stdin };
    bytes: [..] u8;

    defer array_free(bytes);
    buffer: [4096] u8;

    while true {
        ok, count := F.file_read(input, buffer.data, buffer.count);
        if !ok exit(74);
        if !count break;
        for 0..count - 1 array_add(*bytes, buffer[it]);
        if bytes.count > 64 << 20 exit(75);
    }

    push_allocator(temp);
    ok, value := T.string_to_type(string.{ bytes.count, bytes.data }, T.Value);
    if !ok {
        write_string("TOML_READ_REJECTED\n", true);
        exit(1);
    }

    args := get_command_line_arguments();
    output_ok: bool;
    text: string;

    if args.count > 1 && args[1] == "shortest" {
        output_ok, text = T.type_to_string(value,, toml = .{ default_format_float = .{ mode = .SHORTEST, zero_removal = .ONE_ZERO_AFTER_DECIMAL } });
    } else if args.count > 1 && args[1] == "scientific" {
        output_ok, text = T.type_to_string(value,, toml = .{ default_format_float = .{ mode = .SCIENTIFIC, trailing_width = 17, zero_removal = .ONE_ZERO_AFTER_DECIMAL } });
    } else if args.count > 1 && args[1] == "scientific-compact" {
        output_ok, text = T.type_to_string(value,, toml = .{ default_format_float = .{ mode = .SCIENTIFIC, trailing_width = 16, zero_removal = .NO } });
    } else {
        output_ok, text = T.type_to_string(value);
    }

    if !output_ok exit(70);

    write_string(text);
}

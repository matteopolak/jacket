// Runs the repository's pure Jai_Format engine while its full CLI dependencies
// are unavailable in the fresh foundation. Python supplies source on stdin.
// This is not reported as a successful build/check of jaifmt/main.jai.
#import "Basic";
#import "Extensions/Jai_Format";
#import "File";

main :: () -> int {
    config_text, read_ok := read_entire_file("jaifmt.toml");
    if !read_ok return 2;

    defer free(config_text);
    config, ok, message := parse_config(config_text);
    if !ok {
        print("%\n", message, to_standard_error = true);
        return 2;
    }

    input: [..] u8;

    defer array_free(input);

    while true {
        byte := getchar();
        if byte < 0 break;
        array_add(*input, cast(u8) byte);
    }

    output: string;
    output, ok, message = format_source(string.{ input.count, input.data }, config);
    if !ok {
        print("%\n", message, to_standard_error = true);
        return 2;
    }

    defer free(output);
    write_string(output);

    return 0;
}

#scope_file
libc :: #system_library "libc";
getchar :: () -> s32 #foreign libc;

// Projects the actual formatter consumer for comparison with the independent Rust schema.
// Example: echo '[format]' | settings-adapter
#import "Basic";
F :: #import "File";
J :: #import "Extensions/Json";

#if OS == .WINDOWS {
    W :: #import "Windows";
} else {
    P :: #import "POSIX";
}

Fmt :: #import "Extensions/Jai_Format";
#load "../../../benchmarks/compare/toml/settings_projection.jai";

main :: () {
    input: F.File;

    #if OS == .WINDOWS {
        input = .{ W.GetStdHandle(-10) };
    } else {
        input = .{ P.stdin };
    }

    bytes: [..] u8;

    defer array_free(bytes);
    buffer: [4096] u8;

    while true {
        ok, count := F.file_read(input, buffer.data, buffer.count);
        if !ok exit(1);
        if !count break;
        for 0..count - 1 array_add(*bytes, buffer[it]);
        if bytes.count > 64 << 20 exit(1);
    }

    text := string.{ bytes.count, bytes.data };
    config, ok := Fmt.parse_config(text);

    if !ok exit(1);

    result := project_settings(config);

    defer J.free_json(*result);
    output := J.to_json(result);

    defer free(output);
    write_string(output);
}

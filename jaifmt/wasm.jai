// jaifmt as a WASI command, built to jaifmt.wasm with `jaic build -os wasm` (see
// docs/tools/jaifmt.md). It formats the Jai source read from stdin and writes the result to
// stdout, exiting 0. The `jaifmt.toml` text comes from `--config <text>`, else the
// JAIFMT_CONFIG environment variable, else the defaults apply. `--name <file>` names the input
// in messages (default main.jai). On an error it prints `jaifmt: ...` to stderr, prints nothing
// to stdout and exits 1, like the playground driver (playground.jai).

main :: () -> s32 {
    name := "main.jai";
    config_text: string;
    have_config := false;
    args := get_command_line_arguments();
    i := 1;
    while i < args.count {
        flag := args[i];
        if i + 1 >= args.count return fail("% needs a value", flag);
        if flag == {
            case "--config";
                config_text = args[i + 1];
                have_config = true;
            case "--name";
                name = args[i + 1];
            case;
                return fail("unknown option %", flag);
        }
        i += 2;
    }
    if !have_config {
        from_env := c_getenv("JAIFMT_CONFIG");
        if from_env {
            config_text = to_string(from_env);
            have_config = true;
        }
    }

    config: Format_Config;
    if have_config {
        parsed, ok, error := parse_config(config_text);
        if !ok return fail("%: %", FORMAT_CONFIG_FILE_NAME, error);
        config = parsed;
    }

    source, read_ok := read_standard_input();
    if !read_ok return fail("%: cannot read standard input", name);
    formatted, ok, error := format_source(source, config);
    if !ok return fail("%:%", name, error);
    write_string(formatted);
    return 0;
}

#scope_file

#import "Basic";
#import "Extensions/Jai_Format";

libc :: #system_library "libc";
c_read :: (fd: s32, buffer: *void, count: u64) -> s64 #foreign libc "read";
c_getenv :: (name: *u8) -> *u8 #foreign libc "getenv";

read_standard_input :: () -> string, bool {
    builder: String_Builder;
    chunk: [16384] u8;
    while true {
        got := c_read(0, chunk.data, chunk.count);
        if got < 0 return "", false;
        if got == 0 break;
        append(*builder, chunk.data, got);
    }
    return builder_to_string(*builder), true;
}

fail :: (format: string, args: ..Any) -> s32 {
    print("jaifmt: %\n", tprint(format, ..args), to_standard_error = true);
    return 1;
}

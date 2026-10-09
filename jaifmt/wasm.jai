// jaifmt as a WASI command, built to jaifmt.wasm with `jaic build -os wasm` (see
// docs/tools/jaifmt.md). It formats the Jai source read from stdin and writes the result to
// stdout, exiting 0. Options are parsed by Extensions/Args (see `HELP`): a bad command line
// exits 2. On a config or format error it prints `jaifmt: ...` to stderr, prints nothing to
// stdout and exits 1, like the playground driver (playground.jai).

main :: () -> s32 {
    options, set := Args.parse(Options, HELP);

    config: Format_Config;
    if set.config {
        parsed, ok, error := parse_config(options.config);
        if !ok return fail("%: %", FORMAT_CONFIG_FILE_NAME, error);
        config = parsed;
    }

    source, read_ok := read_standard_input();
    if !read_ok return fail("%: cannot read standard input", options.name);
    formatted, ok, error := format_source(source, config);
    if !ok return fail("%:%", options.name, error);
    write_string(formatted);
    return 0;
}

#scope_file

#import "Basic";
#import "Extensions/Jai_Format";
Args :: #import "Extensions/Args";

Options :: struct {
    config: string; @value=TEXT @env=JAIFMT_CONFIG
    name: string = "main.jai"; @value=FILE
}

HELP :: Args.Help(Options).{
    about = "Formats the Jai source on standard input and writes it to standard output.",
    config = "The jaifmt.toml text, not a path (default: the built-in defaults)",
    name = "The input's name in messages",
};

libc :: #system_library "libc";
c_read :: (fd: s32, buffer: *void, count: u64) -> s64 #foreign libc "read";

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

// Generates Bindings_Generator output for one C library at one clang target.
//
// generate --out FILE --triple TRIPLE [--os linux|none] [--include DIR]... [--system DIR]...
//          [--clang ARG]... [--define NAME=VALUE]... -- HEADER...
//
// `--triple` becomes `--target=TRIPLE` for clang, which decides pointer width, alignment and the C
// type widths the generated `#assert`s check. `--os` names the OS the library belongs to: a bare-metal
// triple passes `none`, so no host SDK is searched for libc. Exits 1 when generation reports an error.
#import "Basic";
B :: #import "Bindings_Generator";

main :: () {
    args := get_command_line_arguments();
    out: string;
    triple: string;
    options: B.Generate_Bindings_Options;
    options.add_generator_command = false;
    options.log_stripped_declarations = false;

    headers_started := false;
    i := 1;

    while i < args.count {
        arg := args[i];

        i += 1;

        if arg == "--" {
            headers_started = true;
            continue;
        }

        if headers_started {
            array_add(*options.source_files, arg);
            continue;
        }

        if i >= args.count {
            print("missing value after %\n", arg);
            exit(2);
        }

        value := args[i];

        i += 1;

        if arg == {
            case "--out"; out = value;
            case "--triple"; triple = value;
            case "--os";
                if value == "linux" options.os = .LINUX;
                else if value == "none" options.os = .NONE;
                else {
                    print("unknown --os %\n", value);
                    exit(2);
                }
            case "--include"; array_add(*options.include_paths, value);
            case "--system"; array_add(*options.system_include_paths, value);
            case "--clang"; array_add(*options.extra_clang_arguments, value);
            case "--define"; array_add(*options.extra_clang_arguments, tprint("-D%", value));
            case;
                print("unknown option %\n", arg);
                exit(2);
        }
    }

    if out == "" || triple == "" || options.source_files.count == 0 {
        print("usage: generate --out FILE --triple TRIPLE [--os linux|none] [--include DIR]... -- HEADER...\n");
        exit(2);
    }

    array_add(*options.extra_clang_arguments, tprint("--target=%", triple));
    ok := B.generate_bindings(options, out);
    if !ok exit(1);
}

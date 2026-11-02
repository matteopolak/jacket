// Times Schema.protobuf_generate on one schema set and writes the emitted Jai.
//
//   jaic -interp gen_probe.jai -- --path file.proto|--dir folder [--import-paths a;b] [--out emitted.jai]
//
// Prints one line: `gen_ms=<float> bytes=<n>`, or the schema error and exit code 1.
#import "Basic";
#import "File";
Schema :: #import "Extensions/Protobuf/Schema";

main :: () {
    args := get_command_line_arguments();
    path, dir, import_paths, out: string;

    i := 1;

    while i + 1 < args.count {
        if args[i] == "--path" path = args[i + 1];
        else if args[i] == "--dir" dir = args[i + 1];
        else if args[i] == "--import-paths" import_paths = args[i + 1];
        else if args[i] == "--out" out = args[i + 1];

        i += 2;
    }

    options := Schema.Generate_Options.{ import_paths = import_paths, dir = dir };
    text: string;

    if path.count {
        ok: bool;
        text, ok = read_entire_file(path);
        if !ok {
            print("cannot read %\n", path);
            exit(2);
        }

        options.path = path;
    }

    start := current_time_monotonic();
    output, error := Schema.protobuf_generate(text, options);
    elapsed := to_float64_seconds(current_time_monotonic() - start) * 1000.0;

    if error.message.count {
        print("error %:%:%: %\n", error.path, error.line, error.column, error.message);
        exit(1);
    }

    if out.count write_entire_file(out, output);

    print("gen_ms=% bytes=%\n", elapsed, output.count);
}

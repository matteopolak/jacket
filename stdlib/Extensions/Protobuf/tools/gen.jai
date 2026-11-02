// Print generated Jai: jaic tools/gen.jai -interp -- schema.proto
#import "Basic";
#import "File";
Schema :: #import "Extensions/Protobuf/Schema";

// Prints the generated source for the schema path given on the command line.
main :: () {
    args := get_command_line_arguments();

    defer free(args.data);

    if args.count != 2 {
        print("usage: gen schema.proto\n");
        exit(2);
    }

    text, ok := read_entire_file(args[1]);
    if !ok {
        print("cannot read %\n", args[1]);
        exit(1);
    }

    defer free(text);
    output, error := Schema.protobuf_generate(text, .{ path = args[1] });
    if error.message.count {
        print("%:%:%: %\n", error.path, error.line, error.column, error.message);
        free(error.message);
        exit(1);
    }

    defer free(output);
    print("%", output);
}

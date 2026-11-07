// Print generated Jai: jaic tools/gen.jai -interp -- schema.proto
#import "Basic";
#import "File";
Schema :: #import "Extensions/Protobuf/Schema";

// Prints the generated source for the schema path given on the command line.
main :: () {
    args := get_command_line_arguments();

    defer free(args.data);

    grpc := args.count == 3 && args[1] == "--grpc";
    if args.count != 2 && !grpc {
        print("usage: gen [--grpc] schema.proto\n");
        exit(2);
    }

    path := args[ifx grpc then 2 else 1];
    text, ok := read_entire_file(path);

    if !ok {
        print("cannot read %\n", path);
        exit(1);
    }

    defer free(text);
    output, error := Schema.protobuf_generate(text, .{ path = path, grpc = grpc });
    if error.message.count {
        print("%:%:%: %\n", error.path, error.line, error.column, error.message);
        free(error.message);
        exit(1);
    }

    defer free(output);
    print("%", output);
}

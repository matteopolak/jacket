// test: args("one two", "Three", "$TEMP/data")
#import "Basic";
#import "String";

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);

    // The program's own name may come first; the arguments of the note come last.
    offset := args.count - 3;

    print("% %\n", args[offset], args[offset + 1]);
    print("%\n", ends_with_data(args[offset + 2]));
}

ends_with_data :: (path: string) -> bool {
    return path.count > 5 && slice(path, path.count - 5, 5) == "/data";
}

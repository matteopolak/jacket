#import "Basic";
#import "File";

taken_as_string :: (n: int) -> string {
    builder: String_Builder;
    append(*builder, "total: ");
    print_to_builder(*builder, "%", n);

    return builder_to_string(*builder);
}

freed_by_defer :: (n: int) -> int {
    builder: String_Builder;
    defer free_buffers(*builder);

    print_to_builder(*builder, "%", n);

    return builder_string_length(*builder);
}

written_out :: (n: int) {
    builder: String_Builder;
    print_to_builder(*builder, "%", n);
    write_builder(*builder);
}

closed_file :: (path: string) -> bool {
    file, ok := file_open(path);
    if !ok return false;
    defer file_close(*file);

    return true;
}

handed_on :: (path: string) -> File {
    file, ok := file_open(path);

    return file;
}

filled_by_helper :: (n: int) -> string {
    builder: String_Builder;
    fill(*builder, n);

    return builder_to_string(*builder);
}

fill :: (builder: *String_Builder, n: int) {
    print_to_builder(builder, "%", n);
}

main :: () {
    print("%\n", taken_as_string(1));
    print("%\n", freed_by_defer(2));
    written_out(3);
    print("%\n", closed_file("missing.txt"));
    file := handed_on("missing.txt");

    file_close(*file);
    print("%\n", filled_by_helper(4));
}

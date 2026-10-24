#import "Basic";
#import "File";

writes_only :: (n: int) {
    builder: String_Builder;
    append(*builder, "total: ");
    print_to_builder(*builder, "%", n);
}

opens_and_checks :: (path: string) -> bool {
    file, ok := file_open(path);
    if !ok return false;

    return true;
}

started_only :: () {
    builder: String_Builder;
    init_string_builder(*builder);

    if builder.failed print("failed\n");
}

main :: () {
    writes_only(1);
    print("%\n", opens_and_checks("missing.txt"));
    started_only();
}

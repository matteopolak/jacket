#import "Basic";
R :: #import "Windows_Resources";
F :: #import "File";
CSV :: #import "Subtitles";

#assert OS == .WINDOWS;

main :: () {
    args := get_command_line_arguments();

    assert(args.count == 2);
    root10 := R.find_windows_kit_root("KitsRoot10");
    root81 := R.find_windows_kit_root("KitsRoot81");
    selected, version := R.find_windows_kit_root();

    defer free(selected);
    library, linker := R.find_visual_studio_in_a_ridiculous_garbage_way();
    values: [6] string = .[root10, root81, selected, tprint("%", version), library, linker];
    builder: String_Builder;

    defer free_buffers(*builder);

    for value, index: values {
        if index > 0 append(*builder, ",");
        quoted := CSV.csv_quote(value);
        append(*builder, quoted);
        free(quoted);
    }

    append(*builder, "\n");
    path := sprint("%/windows-discovery.csv", args[1]);

    defer free(path);
    assert(F.write_entire_file(path, *builder));
    print("discovery report written for independent Windows SDK comparison\n");
}

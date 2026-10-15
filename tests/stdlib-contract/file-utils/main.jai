// Whole-file ownership, truncating writes, and reduced path component behavior.
#import "Basic";
File :: #import "File";
File_Utilities :: #import "File_Utilities";

main :: () {
    scratch_name := "stdlib-contract-file.tmp";
    builder: String_Builder;

    init_string_builder(*builder);
    append(*builder, "first");
    append(*builder, "-second");
    assert(File.write_entire_file(scratch_name, *builder));
    assert(builder_string_length(*builder) == 0);

    data, read_ok := File.read_entire_file(scratch_name, zero_terminated = true);

    assert(read_ok && data == "first-second");
    assert(data.data[data.count] == 0);
    free(data);
    assert(File.write_entire_file(scratch_name, "oak"));
    data, read_ok = File.read_entire_file(scratch_name);
    assert(read_ok && data == "oak");
    free(data);

    opened, open_ok := File.file_open(scratch_name);

    assert(open_ok && File.is_valid(opened));
    byte: u8;
    bytes_read: s64;
    read_ok, bytes_read = File.file_read(opened, *byte, 1);
    assert(read_ok && bytes_read == 1 && byte == cast(u8) #char "o");
    tail, tail_ok := File.read_entire_file(opened);

    assert(tail_ok && tail == "ak");
    free(tail);
    read_ok, bytes_read = File.file_read(opened, *byte, 1);
    assert(read_ok && bytes_read == 0);
    File.file_close(*opened);

    assert(File.file_delete(scratch_name));

    path := File_Utilities.parse_path("..//orchard/./maple/../oak/");

    assert(path.number_of_leading_slashes == 0 && path.trailing_slash);
    assert(path.words.count == 3);
    assert(path.words[0] == ".." && path.words[1] == "orchard" && path.words[2] == "oak");
    found, position := File_Utilities.path_contains(path, .["orchard", "oak"]);

    assert(found && position == 1);
    found, position = File_Utilities.path_contains(path, .["oak", "orchard"]);
    assert(!found && position == 0);
    assert(File_Utilities.path_to_string(path) == "../orchard/oak/");
    print("ok\n");
}

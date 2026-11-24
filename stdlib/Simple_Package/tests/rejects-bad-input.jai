// A package with a bad signature, a newer version, or a cut-off table of contents is refused by init_from_memory.
#import "Basic";
P :: #import "Simple_Package";
F :: #import "File";

reports: int;

count_errors :: (message: string, data: *void, info: Log_Info) {
    if (info.common_flags & .ERROR) != 0 reports += 1;
}

// Returns a fresh copy of a one-entry package, from a writer run in memory.
make_valid :: () -> [] u8 {
    creator: P.Create_Package;

    P.init(*creator);
    defer P.deinit(*creator);
    P.add(*creator, "only", .[7, 8, 9]);

    // The writer only writes to files, so build the bytes by writing and reading back.
    path := "simple-package-reject.simp";

    assert(P.write(*creator, path));
    bytes, ok := F.read_entire_file(path);

    assert(ok);
    assert(F.file_delete(path));

    return .{ bytes.count, bytes.data };
}

try_load :: (label: string, data: [] u8) {
    loaded: P.Load_Package;
    before := reports;

    context.logger = count_errors;
    ok := P.init_from_memory(*loaded, data, label);

    context.logger = null;
    print("% loaded=% errors=%\n", label, ok, reports - before);
}

main :: () {
    valid := make_valid();

    defer free(valid.data);

    loaded: P.Load_Package;

    assert(P.init_from_memory(*loaded, valid, "valid"));
    print("valid entries=% first=% size=%\n", loaded.table_of_contents.number_of_entries, loaded.entries[0].name, loaded.entries[0].size_of_entry);
    assert(loaded.entries[0].name == "only" && loaded.entries[0].size_of_entry == 3);

    // The magic is the first four bytes; "simp" is 0x706d6973 little-endian.
    bad_magic := copy_bytes(valid);

    defer free(bad_magic.data);
    bad_magic[0] = 0;
    try_load("bad magic", bad_magic);

    // Version is the next u32: 2 is newer than this reader supports.
    newer := copy_bytes(valid);

    defer free(newer.data);
    newer[4] = 2;
    try_load("newer version", newer);

    // A file cut off inside the table of contents.
    cut := valid;
    cut.count = valid.count - 6;
    try_load("cut short", cut);

    // Nothing at all.
    try_load("empty", .[]);
    print("ok\n");
}

copy_bytes :: (source: [] u8) -> [] u8 {
    result: [] u8;
    result.data = alloc(source.count);
    result.count = source.count;
    for source result[it_index] = it;

    return result;
}

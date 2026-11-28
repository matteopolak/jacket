// A package written sorted and in add order, read back from memory and from a file, with every payload compared byte for byte.
#import "Basic";
P :: #import "Simple_Package";
F :: #import "File";
H :: #import "Hash_Table";

zeta_bytes: [3] u8 = .[1, 2, 255];
mid_bytes: [300] u8;

same_bytes :: (a: [] u8, b: [] u8) -> bool {
    if a.count != b.count return false;
    for a if it != b[it_index] return false;
    return true;
}

source_for :: (name: string) -> [] u8 {
    if name == "zeta" return zeta_bytes;
    if name == "mid" return mid_bytes;
    return .[];
}

check_entries :: (label: string, package: *P.Load_Package) -> bool {
    all_ok := true;

    for package.entries {
        expected := source_for(it.name);
        same := same_bytes(it.data, expected) && it.size_of_entry == expected.count;
        print("% entry % size=% match=%\n", label, it.name, it.size_of_entry, same);
        all_ok &= same;
    }

    return all_ok;
}

release :: (package: *P.Load_Package) {
    H.deinit(*package.lookup);
    array_free(package.entries);
    package.* = .{};
}

main :: () {
    for *mid_bytes it.* = cast(u8) ((it_index * 7) % 256);

    creator: P.Create_Package;

    P.init(*creator);
    defer P.deinit(*creator);
    P.add(*creator, "zeta", zeta_bytes);
    P.add(*creator, "alpha", .[]);
    P.add(*creator, "mid", mid_bytes);

    sorted_path := "simple-package-sorted.simp";
    ordered_path := "simple-package-ordered.simp";

    assert(P.write(*creator, sorted_path));
    assert(P.write(*creator, ordered_path, sort_contents = false));

    sorted_bytes, read_ok := F.read_entire_file(sorted_path);

    assert(read_ok);
    defer free(sorted_bytes);
    // Header 64 + payloads 0 + 300 + 3, then the table of contents: 64 + 26 ("alpha") + 24 ("mid") + 25 ("zeta").
    assert(sorted_bytes.count == 506);

    memory: P.Load_Package;

    assert(P.init_from_memory(*memory, .{ sorted_bytes.count, sorted_bytes.data }, "sorted memory"));
    print("sorted file size=% table_offset=%\n", sorted_bytes.count, memory.header.table_of_contents_offset);
    assert(memory.header.table_of_contents_offset == 367);
    assert(memory.table_of_contents.number_of_entries == 3);
    all_ok := check_entries("memory", *memory);

    found, mid := H.table_find(*memory.lookup, "mid");

    assert(found && mid.size_of_entry == 300);
    release(*memory);

    file_package: P.Load_Package;

    assert(P.init_from_file(*file_package, sorted_path));
    all_ok &= check_entries("file", *file_package);
    P.free_package(*file_package);

    ordered_bytes, ordered_ok := F.read_entire_file(ordered_path);

    assert(ordered_ok);
    defer free(ordered_bytes);
    // Add order is kept: the entries are zeta, alpha, mid.
    ordered: P.Load_Package;

    assert(P.init_from_memory(*ordered, .{ ordered_bytes.count, ordered_bytes.data }));
    print("ordered names=%,%,%\n", ordered.entries[0].name, ordered.entries[1].name, ordered.entries[2].name);
    assert(ordered.entries[0].name == "zeta" && ordered.entries[1].name == "alpha" && ordered.entries[2].name == "mid");
    all_ok &= check_entries("ordered", *ordered);
    release(*ordered);

    assert(F.file_delete(sorted_path));
    assert(F.file_delete(ordered_path));
    assert(all_ok);
    print("ok\n");
}

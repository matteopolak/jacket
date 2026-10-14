#import "Basic";
F :: #import "File";
U :: #import "File_Utilities";

main :: () {
    assert(F.make_directory_if_it_does_not_exist("portable-tree/sub", true));
    assert(F.write_entire_file("portable-tree/sub/leaf", "copper"));
    assert(U.file_exists("portable-tree/sub/leaf"));
    assert(!U.copy_file("portable-tree/sub/leaf", "portable-tree/sub/leaf"));
    assert(!U.copy_directory("portable-tree", "portable-tree/copy"));
    stamp, size, ok := U.file_modtime_and_size("portable-tree/sub/leaf");

    assert(ok && size == 6);
    stamp_only, stamp_ok := U.file_modtime("portable-tree/sub/leaf");

    assert(stamp_ok && compare_apollo_times(stamp, stamp_only) == 0);
    list := U.file_list("portable-tree", true);

    assert(list.count == 1 && list[0] == "portable-tree/sub/leaf");
    for path: list free(path);

    array_free(list);
    assert(U.copy_directory("portable-tree", "portable-tree-copy"));
    data, valid := F.read_entire_file("portable-tree-copy/sub/leaf");

    assert(valid && data == "copper");
    free(data);
    assert(F.file_delete("portable-tree-copy/sub/leaf"));
    assert(F.delete_directory("portable-tree-copy/sub"));
    assert(F.delete_directory("portable-tree-copy"));
    assert(F.file_delete("portable-tree/sub/leaf"));
    assert(F.delete_directory("portable-tree/sub"));
    assert(F.delete_directory("portable-tree"));
    print("ok\n");
}

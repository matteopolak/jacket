#import "Basic";
F :: #import "File";
U :: #import "File_Utilities";

main :: () {
    root := "foundation-filesystem-test";

    assert(F.make_directory_if_it_does_not_exist(root));
    defer assert(F.delete_directory(root));
    assert(F.make_directory_if_it_does_not_exist("foundation-filesystem-test/source/empty", true));

    defer {
        assert(F.file_delete("foundation-filesystem-test/source/content"));
        assert(F.delete_directory("foundation-filesystem-test/source/empty"));
        assert(F.delete_directory("foundation-filesystem-test/source"));
    }

    assert(F.write_entire_file("foundation-filesystem-test/source/content", "bytes\0inside"));
    assert(F.file_link("foundation-filesystem-test/hardlink", "foundation-filesystem-test/source/content"));
    defer assert(F.file_delete("foundation-filesystem-test/hardlink"));
    assert(!U.copy_file("foundation-filesystem-test/source/content", "foundation-filesystem-test/hardlink"));
    original, read_ok := F.read_entire_file("foundation-filesystem-test/hardlink");

    assert(read_ok && original == "bytes\0inside");
    free(original);
    assert(!U.copy_directory("foundation-filesystem-test/source", "foundation-filesystem-test/source/empty/../copy"));
    assert(!U.file_exists("foundation-filesystem-test/source/copy"));
    assert(U.copy_directory("foundation-filesystem-test/source", "foundation-filesystem-test/dest"));

    defer {
        assert(F.file_delete("foundation-filesystem-test/dest/content"));
        assert(F.delete_directory("foundation-filesystem-test/dest/empty"));
        assert(F.delete_directory("foundation-filesystem-test/dest"));
    }

    copied, copy_ok := F.read_entire_file("foundation-filesystem-test/dest/content");

    assert(copy_ok && copied == "bytes\0inside");
    free(copied);
    assert(U.file_exists("foundation-filesystem-test/dest/empty"));
    assert(F.write_entire_file("foundation-filesystem-test/dest/content", "longer destination contents to truncate"));
    assert(!U.copy_file("foundation-filesystem-test/source", "foundation-filesystem-test/dest/content"));
    preserved, preserved_ok := F.read_entire_file("foundation-filesystem-test/dest/content");

    assert(preserved_ok && preserved == "longer destination contents to truncate");
    free(preserved);
    assert(U.copy_file("foundation-filesystem-test/source/content", "foundation-filesystem-test/dest/content"));
    copied, copy_ok = F.read_entire_file("foundation-filesystem-test/dest/content");
    assert(copy_ok && copied == "bytes\0inside");
    free(copied);
    assert(U.atomic_file_replace_or_just_rename_if_destination_file_does_not_exist("foundation-filesystem-test/dest/content", "foundation-filesystem-test/replaced"));
    assert(!U.file_exists("foundation-filesystem-test/dest/content"));
    assert(U.file_exists("foundation-filesystem-test/replaced"));
    assert(U.atomic_file_replace_or_just_rename_if_destination_file_does_not_exist("foundation-filesystem-test/replaced", "foundation-filesystem-test/dest/content"));
    print("ok\n");
}

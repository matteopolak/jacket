// test: skip_on(.WASM, reason = "File_Utilities has no wasm backend")
#import "Basic";
F :: #import "File";
U :: #import "File_Utilities";

main :: () {
    root := "delete-tree-test";
    keep := "delete-tree-keep";

    // A folder that must survive: a link inside the tree points at it.
    assert(F.make_directory_if_it_does_not_exist(keep));
    assert(F.write_entire_file("delete-tree-keep/precious", "stay"));

    assert(F.make_directory_if_it_does_not_exist("delete-tree-test/a/b/c", recursive = true));
    assert(F.make_directory_if_it_does_not_exist("delete-tree-test/empty"));
    assert(F.write_entire_file("delete-tree-test/top", "1"));
    assert(F.write_entire_file("delete-tree-test/a/mid", "2"));
    assert(F.write_entire_file("delete-tree-test/a/b/c/leaf", "3"));

    #if OS_IS_UNIX {
        assert(symlink_to("../delete-tree-keep", "delete-tree-test/a/link"));
    }

    assert(!F.delete_directory(root)); // not empty: the plain call is unchanged
    assert(!U.delete_directory_tree("delete-tree-test/top")); // a file
    assert(U.file_exists("delete-tree-test/top"));

    assert(U.delete_directory_tree("delete-tree-test/"));
    assert(!U.file_exists(root));
    assert(!U.delete_directory_tree(root)); // gone already

    precious, ok := F.read_entire_file("delete-tree-keep/precious");

    assert(ok && precious == "stay");
    free(precious);

    #if OS_IS_UNIX {
        // A link to a folder is removed as a link, not emptied.
        assert(symlink_to("delete-tree-keep", "delete-tree-link"));
        assert(U.delete_directory_tree("delete-tree-link"));
        assert(!U.file_exists("delete-tree-link"));
        assert(U.file_exists("delete-tree-keep/precious"));
    }

    assert(U.delete_directory_tree(keep));
    assert(!U.file_exists(keep));
    print("ok\n");
}

#if OS_IS_UNIX {
    P :: #import "POSIX";

    symlink_to :: (target: string, link: string) -> bool {
        ctarget := to_c_string(target);

        defer free(ctarget);
        clink := to_c_string(link);

        defer free(clink);

        return P.symlink(ctarget, clink) == 0;
    }
}

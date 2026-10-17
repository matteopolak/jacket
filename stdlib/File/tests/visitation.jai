#import "Basic";
U :: #import "File_Utilities";
F :: #import "File";

State :: struct {
    files: int;
    directories: int;
    saw_leaf: bool;
    prune: bool;
    links: int;
}

main :: () {
    assert(F.make_directory_if_it_does_not_exist("foundation-visit/sub", true));

    defer {
        assert(F.file_delete("foundation-visit/top"));
        assert(F.file_delete("foundation-visit/sub/leaf"));
        assert(F.delete_directory("foundation-visit/sub"));
        assert(F.delete_directory("foundation-visit"));
    }

    assert(F.write_entire_file("foundation-visit/top", "top"));
    assert(F.write_entire_file("foundation-visit/sub/leaf", "leaf"));
    directory, ok := U.is_directory("foundation-visit");

    assert(ok && directory);
    directory, ok = U.is_directory("foundation-visit/top");
    assert(ok && !directory);
    directory, ok = U.is_directory("foundation-visit/missing");
    assert(!ok && !directory);
    state: State;

    assert(U.visit_files("foundation-visit", true, *state, visitor, visit_directories = true));
    assert(state.files == 2 && state.directories == 1 && state.saw_leaf);
    state = .{ prune = true };
    assert(U.visit_files("foundation-visit", true, *state, visitor, visit_directories = true));
    assert(state.files == 1 && state.directories == 1 && !state.saw_leaf);
    state = .{};
    assert(U.visit_files("foundation-visit", true, *state, visitor, visit_files = false, visit_directories = true));
    assert(state.files == 0 && state.directories == 1);
    assert(!U.visit_files("foundation-visit/missing", false, *state, visitor));
    info: U.File_Visit_Info;
    info.full_name = "foundation-visit/top";
    info.modification_time = APOLLO_TIME_INVALID;
    assert(U.add_modtime_and_size_if_missing(*info));
    assert(info.size == 3 && !info.had_error);
    assert(make_symlink("sub", "foundation-visit/link"));
    defer assert(F.file_delete("foundation-visit/link"));
    assert(make_symlink("..", "foundation-visit/sub/back"));
    defer assert(F.file_delete("foundation-visit/sub/back"));
    state = .{};
    assert(U.visit_files("foundation-visit", true, *state, visitor, follow_directory_symlinks = false));
    assert(state.files == 2 && state.links == 2);
    state = .{};
    assert(U.visit_files("foundation-visit", true, *state, visitor));
    assert(state.files == 3 && state.links == 3);
    // Distinct paths to sub are walked; links to ancestors do not recurse forever.
    state = .{};
    assert(U.visit_files("foundation-visit", true, *state, visitor, visit_symlinks = false, follow_directory_symlinks = false));
    assert(state.links == 0 && state.files == 2);
    list := U.file_list("foundation-visit", true);

    assert(list.count == 3);
    for path: list free(path);

    array_free(list);
    print("ok\n");
}

visitor :: (info: *U.File_Visit_Info, state: *State) {
    assert(!info.had_error);
    assert(info.full_name.count >= info.short_name.count);
    if info.is_symlink state.links += 1;

    if info.is_directory {
        state.directories += 1;
        if state.prune info.descend_into_directory = false;
    } else {
        state.files += 1;

        if info.short_name == "leaf" {
            state.saw_leaf = true;
            assert(info.size == 4);
        }
    }
}

make_symlink :: (target: string, path: string) -> bool {
    ctarget := to_c_string(target);

    defer free(ctarget);
    cpath := to_c_string(path);

    defer free(cpath);

    return P.symlink(ctarget, cpath) == 0;
}

P :: #import "POSIX";

// File_Utilities on a real tree: visit_files and its flags, file_list, copy_file and
// copy_directory, is_directory, modification times and sizes, atomic replacement, and
// directory symlinks (including a loop) on POSIX hosts.
#import "Basic";
#import "File";
#import "File_Utilities";
#import "String";
#if OS == .MACOS || OS == .LINUX {
    POSIX :: #import "POSIX";
}

ROOT :: ".scratch/file-utilities-tree";

path :: (name: string) -> string {
    return tprint("%/%", ROOT, name);
}

contents :: (name: string) -> string {
    text, ok := read_entire_file(name);
    assert(ok, "read %", name);
    return text;
}

// The names in a listing, relative to the root, sorted, joined with "|".
relative :: (names: [] string) -> string {
    prefix := tprint("%/", ROOT);
    sorted: [..] string;
    for names {
        assert(begins_with(it, prefix), "'%' is under the root", it);
        array_add(*sorted, slice(it, prefix.count, it.count - prefix.count));
    }
    // Insertion sort: the order of directory entries is up to the OS.
    for i: 1..sorted.count - 1 {
        j := i;
        while j > 0 && compare(sorted[j - 1], sorted[j]) > 0 {
            sorted[j - 1], sorted[j] = sorted[j], sorted[j - 1];
            j -= 1;
        }
    }
    return join(..sorted, "|");
}

Visit_Log :: struct {
    files: [..] string;
    directories: [..] string;
    infos_ok: bool = true;
}

record_visit :: (info: *File_Visit_Info, log: *Visit_Log) {
    if info.is_directory array_add(*log.directories, copy_string(info.full_name));
    else array_add(*log.files, copy_string(info.full_name));
    if !ends_with(info.full_name, info.short_name) || info.had_error log.infos_ok = false;
    if info.modification_time == .{} log.infos_ok = false;
}

main :: () {
    delete_directory(ROOT);
    assert(make_directory_if_it_does_not_exist(path("src/sub/deeper"), recursive = true));
    assert(make_directory_if_it_does_not_exist(path("src/empty")));
    assert(write_entire_file(path("src/a.txt"), "alpha"));
    assert(write_entire_file(path("src/b-ü.txt"), "beta ü"));
    assert(write_entire_file(path("src/sub/c.bin"), ""));
    assert(write_entire_file(path("src/sub/deeper/d.txt"), "delta"));

    {
        listed := file_list(path("src"));
        assert(relative(listed) == "src/a.txt|src/b-ü.txt", "flat listing '%'", relative(listed));
        listed = file_list(path("src"), recursive = true);
        assert(relative(listed) == "src/a.txt|src/b-ü.txt|src/sub/c.bin|src/sub/deeper/d.txt", "recursive listing '%'", relative(listed));
        // A missing directory lists nothing.
        assert(file_list(path("missing")).count == 0);
    }
    {
        log: Visit_Log;
        ok := visit_files(path("src"), true, *log, record_visit, visit_files = true, visit_directories = true);
        assert(ok && log.infos_ok);
        assert(relative(log.directories) == "src/empty|src/sub|src/sub/deeper", "directories '%'", relative(log.directories));
        assert(log.files.count == 4);

        only_dirs: Visit_Log;
        ok = visit_files(path("src"), false, *only_dirs, record_visit, visit_files = false, visit_directories = true);
        assert(ok && only_dirs.files.count == 0);
        assert(relative(only_dirs.directories) == "src/empty|src/sub");

        missing: Visit_Log;
        ok = visit_files(path("missing"), true, *missing, record_visit);
        assert(!ok && missing.files.count == 0, "visiting a missing directory fails");

        // A trailing separator on the root still finds everything.
        trailing: Visit_Log;
        ok = visit_files(tprint("%/src/", ROOT), true, *trailing, record_visit);
        assert(ok && trailing.files.count == 4);
    }
    {
        dir, ok := is_directory(path("src/sub"));
        assert(dir && ok);
        dir, ok = is_directory(path("src/a.txt"));
        assert(!dir && ok);
        dir, ok = is_directory(path("src/none"));
        assert(!dir && !ok);
        assert(file_exists(path("src/sub/c.bin")) && !file_exists(path("src/sub/none")));

        modtime, size, found := file_modtime_and_size(path("src/b-ü.txt"));
        assert(found && size == "beta ü".count, "size %", size);
        now := current_time_consensus();
        // Within a day of now: catches unit and epoch mistakes.
        assert(modtime > now - seconds_to_apollo(86400) && modtime < now + seconds_to_apollo(86400), "modification time % vs now %", modtime, now);
        _, size, found = file_modtime_and_size(path("src/sub/c.bin"));
        assert(found && size == 0);
        _, _, found = file_modtime_and_size(path("src/none"));
        assert(!found);
    }
    {
        // copy_file: contents, an empty file, a large file, overwrite, and refusals.
        assert(copy_file(path("src/a.txt"), path("copy.txt")));
        assert(contents(path("copy.txt")) == "alpha");
        assert(copy_file(path("src/b-ü.txt"), path("copy.txt")));
        assert(contents(path("copy.txt")) == "beta ü", "overwritten copy");
        assert(copy_file(path("src/sub/c.bin"), path("empty-copy.bin")));
        assert(file_exists(path("empty-copy.bin")) && contents(path("empty-copy.bin")) == "");
        big := alloc_string(3 * 65536 + 17);
        for i: 0..big.count - 1 big[i] = cast,trunc(u8) (i * 13 + i / 65536);
        assert(write_entire_file(path("big.bin"), big));
        assert(copy_file(path("big.bin"), path("big-copy.bin")));
        assert(contents(path("big-copy.bin")) == big, "large copy");
        // A buffer-sized file ends exactly on a read boundary.
        exact := alloc_string(65536);
        memset(exact.data, #char "e", exact.count);
        assert(write_entire_file(path("exact.bin"), exact));
        assert(copy_file(path("exact.bin"), path("exact-copy.bin")));
        assert(contents(path("exact-copy.bin")) == exact);

        assert(!copy_file(path("src/a.txt"), path("src/a.txt")), "copying onto itself");
        assert(!copy_file(path("src/a.txt"), tprint("%/src/./a.txt", ROOT)), "copying onto itself by another name");
        assert(contents(path("src/a.txt")) == "alpha");
        assert(!copy_file(path("src/none"), path("x")));
        assert(!file_exists(path("x")));
        assert(!copy_file(path("src/a.txt"), path("no-dir/x")));
    }
    {
        copy_directory(path("src"), path("dst/nested"));
        listed := file_list(path("dst/nested"), recursive = true);
        assert(relative(listed) == "dst/nested/a.txt|dst/nested/b-ü.txt|dst/nested/sub/c.bin|dst/nested/sub/deeper/d.txt", "copied tree '%'", relative(listed));
        assert(contents(path("dst/nested/sub/deeper/d.txt")) == "delta");
        // Empty directories come along too.
        dir := is_directory(path("dst/nested/empty"));
        assert(dir, "the empty directory was copied");
    }
    {
        // Atomic replacement: over an existing file, and to a new name.
        assert(write_entire_file(path("new.txt"), "new"));
        assert(write_entire_file(path("target.txt"), "old"));
        assert(atomic_file_replace_or_just_rename_if_destination_file_does_not_exist(path("new.txt"), path("target.txt")));
        assert(!file_exists(path("new.txt")));
        assert(contents(path("target.txt")) == "new");
        assert(atomic_file_replace_or_just_rename_if_destination_file_does_not_exist(path("target.txt"), path("fresh.txt")));
        assert(contents(path("fresh.txt")) == "new" && !file_exists(path("target.txt")));
        assert(!atomic_file_replace_or_just_rename_if_destination_file_does_not_exist(path("gone.txt"), path("fresh.txt")));
        assert(contents(path("fresh.txt")) == "new");
    }
    #if OS == .MACOS || OS == .LINUX {
        // A directory symlink, and one that points back up (a loop visit_files must not follow forever).
        assert(POSIX.symlink(temp_c_string("sub"), temp_c_string(path("src/link"))) == 0);
        assert(POSIX.symlink(temp_c_string(".."), temp_c_string(path("src/sub/up"))) == 0);
        followed := file_list(path("src"), recursive = true);
        assert(followed.count >= 4 && followed.count < 50, "followed listing has % names", followed.count);
        found_through_link := false;
        for followed if ends_with(it, "link/c.bin") found_through_link = true;
        // Through the link or the real path: each directory is listed once.
        unfollowed := file_list(path("src"), recursive = true, follow_directory_symlinks = false);
        assert(relative(unfollowed) == "src/a.txt|src/b-ü.txt|src/sub/c.bin|src/sub/deeper/d.txt", "unfollowed '%'", relative(unfollowed));
        log: Visit_Log;
        assert(visit_files(path("src"), false, *log, record_visit, visit_directories = true, visit_symlinks = false));
        assert(relative(log.directories) == "src/empty|src/sub", "symlinks skipped: '%'", relative(log.directories));
    }

    assert(delete_directory(ROOT));
    assert(!file_exists(ROOT));
    print("ok\n");
}

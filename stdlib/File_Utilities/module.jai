// Asking about files, walking and copying folders, and taking paths apart.
//
// Where `File` opens and reads single files, this module looks at the file system as a whole: `file_exists`,
// `is_directory` and `file_modtime_and_size` examine a path, `visit_files` and `file_list` walk a folder,
// `copy_file` and `copy_directory` copy, `delete_directory_tree` removes a folder with everything in it, and `parse_path` turns text into a `Path` of names that `reduce_path`,
// `path_contains` and `path_to_string` work on without touching the disk. The procedures that take a path
// answer `false` for one with a NUL byte in it. Strings that a procedure returns come from `context.allocator` and
// are the caller's to free. There is no wasm backend.
//
// ```jai,skip_on(wasm)
// #import "Extensions/Environment";
// #import "File";
// base := environment_variable("TMPDIR");
// if base == "" base = environment_variable("TEMP");
// if base == "" base = "/tmp";
// path := tprint("%/jai-doc-file-utilities.txt", base);
// defer file_delete(path);
//
// assert(!file_exists(path));
// assert(write_entire_file(path, "x"));
// assert(file_exists(path));
// ```

Path :: Paths.Path;
parse_path :: Paths.parse_path;
reduce_path :: Paths.reduce_path;
path_contains :: Paths.path_contains;
path_to_string :: Paths.path_to_string;
copy_to_permanent :: Paths.copy_to_permanent;
free_permanent :: Paths.free_permanent;
File_Visit_Info :: Visit.File_Visit_Info;

#if OS_IS_UNIX {
    visit_files :: Files.visit_files;
    is_directory :: Files.is_directory;
    add_modtime_and_size_if_missing :: Files.add_modtime_and_size_if_missing;
    file_exists :: Files.file_exists;
    file_modtime :: Files.file_modtime;
    file_modtime_and_size :: Files.file_modtime_and_size;
    copy_file :: Files.copy_file;
    copy_directory :: Files.copy_directory;
    file_list :: Files.file_list;
    how_much_of_path_exists_on_the_local_filesystem :: Files.how_much_of_path_exists_on_the_local_filesystem;
    atomic_file_replace_or_just_rename_if_destination_file_does_not_exist :: Files.atomic_file_replace_or_just_rename_if_destination_file_does_not_exist;
}

#if OS == .WINDOWS {
    visit_files :: Files.visit_files;
    file_list :: Files.file_list;
    copy_directory :: Files.copy_directory;
    how_much_of_path_exists_on_the_local_filesystem :: Files.how_much_of_path_exists_on_the_local_filesystem;
    file_exists :: Files.file_exists;
    is_directory :: Files.is_directory;
    file_modtime :: Files.file_modtime;
    file_modtime_and_size :: Files.file_modtime_and_size;
    add_modtime_and_size_if_missing :: Files.add_modtime_and_size_if_missing;
    copy_file :: Files.copy_file;
    atomic_file_replace_or_just_rename_if_destination_file_does_not_exist :: Files.atomic_file_replace_or_just_rename_if_destination_file_does_not_exist;
}

#if OS_IS_UNIX || OS == .WINDOWS {
    // Deletes the folder `path` and everything in it. Returns whether the folder is gone.
    //
    // `File.delete_directory` only removes an empty folder; this walks down first and removes the files, then the folders,
    // deepest first. A link is deleted as a link and never followed, so what it points to is left alone, also when `path` itself
    // is a link to a folder. `false` for a missing path, a path that is a file, or one with a NUL byte in it. If something cannot
    // be removed (no permission, a read-only file on Windows), the rest is still tried and the answer is `false`, so a failure
    // may leave part of the tree behind.
    //
    // ```jai,skip_on(wasm)
    // #import "Extensions/Environment";
    // #import "File";
    // base := environment_variable("TMPDIR");
    // if base == "" base = environment_variable("TEMP");
    // if base == "" base = "/tmp";
    // root := tprint("%/jai-doc-delete-tree", base);
    //
    // assert(make_directory_if_it_does_not_exist(tprint("%/a/b", root), recursive = true));
    // assert(write_entire_file(tprint("%/a/b/leaf", root), "x"));
    // assert(!delete_directory(root)); // not empty
    //
    // assert(delete_directory_tree(root));
    // assert(!file_exists(root));
    // assert(!delete_directory_tree(root)); // already gone
    // ```
    delete_directory_tree :: (path: string) -> bool {
        trimmed := path;
        while trimmed.count > 1 && (trimmed[trimmed.count - 1] == #char "/" || trimmed[trimmed.count - 1] == #char "\\") trimmed.count -= 1;

        if is_symlink(trimmed) return F.file_delete(trimmed) || F.delete_directory(trimmed);

        directory, found := is_directory(trimmed);
        if !found || !directory return false;

        return remove_tree(trimmed);
    }
}

#scope_file
F :: #import "File";
S :: #import "String";
#import "Basic";

#if OS_IS_UNIX || OS == .WINDOWS {
    Tree_Entry :: struct {
        name: string;
        is_directory: bool;
        is_symlink: bool;
    }

    collect_entry :: (info: *File_Visit_Info, entries: *[..] Tree_Entry) {
        array_add(entries, .{ copy_string(info.full_name), info.is_directory, info.is_symlink });
    }

    Link_Search :: struct {
        name: string;
        found: bool;
    }

    note_link :: (info: *File_Visit_Info, search: *Link_Search) {
        if info.short_name == search.name search.found = true;
    }

    // Whether `path` is itself a link, found among the links of its parent folder.
    is_symlink :: (path: string) -> bool {
        search := Link_Search.{ name = S.path_filename(path) };
        if search.name.count == 0 return false;

        parent := S.path_strip_filename(path);
        if parent.count == 0 parent = ".";

        visit_files(parent, false, *search, note_link, visit_files = false, visit_directories = false, visit_symlinks = true, follow_directory_symlinks = false);

        return search.found;
    }

    remove_tree :: (path: string) -> bool {
        entries: [..] Tree_Entry;

        defer {
            for entries free(it.name);
            array_free(entries);
        }

        ok := visit_files(path, false, *entries, collect_entry, visit_files = true, visit_directories = true, visit_symlinks = true, follow_directory_symlinks = false);

        for entries {
            if it.is_symlink {
                if !(F.file_delete(it.name) || F.delete_directory(it.name)) ok = false;
            } else if it.is_directory {
                if !remove_tree(it.name) ok = false;
            } else if !F.file_delete(it.name) {
                ok = false;
            }
        }

        return F.delete_directory(path) && ok;
    }
}

Paths :: #import "internal/files/paths";
Visit :: #import "internal/files/visit-info";
#if OS_IS_UNIX Files :: #import "internal/files/filesystem";
#if OS == .WINDOWS Files :: #import "internal/files/filesystem-windows";

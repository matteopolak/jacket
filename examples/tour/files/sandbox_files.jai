// Private tour adapter for the existing sandbox/WASI C bridge. This is not a
// File_Utilities WASM export. Both runtimes supply the same directory-entry ABI.
Tour_File_Visit_Info :: struct {
    full_name: string;
    short_name: string;
}

tour_visit_files :: (directory: string, recursive: bool, user_data: $T, visitor: (info: *Tour_File_Visit_Info, user_data: T)) -> bool {
    name := to_c_string(directory);

    defer free(name);
    stream := tour_directory_open(name);
    if !stream return false;

    success := true;

    while true {
        error := tour_file_errno();
        error.* = 0;
        entry := tour_directory_read(stream);
        if !entry {
            success = error.* == 0;
            break;
        }

        short_name := to_string(entry.name.data);
        if short_name == "." || short_name == ".." continue;

        full_name := tprint("%/%", directory, short_name);

        if entry.kind == 4 {
            if recursive && !tour_visit_files(full_name, recursive, user_data, visitor) {
                success = false;
                break;
            }
        } else if entry.kind == 8 {
            info := Tour_File_Visit_Info.{ full_name, short_name };
            visitor(*info, user_data);
        } else {
            // This private workspace walk does not follow symlinks or guess an
            // unknown entry's kind. Report unsupported traversal to its caller.
            success = false;
            break;
        }
    }

    if tour_directory_close(stream) != 0 success = false;

    return success;
}

tour_file_exists :: (path: string) -> bool {
    name := to_c_string(path);
    defer free(name);
    return tour_file_access(name, 0) == 0;
}

#scope_file
Tour_Directory_Entry :: struct {
    inode: u64;
    next_offset: s64;
    record_bytes: u16;
    kind: u8;
    name: [256] u8;
}

#assert size_of(Tour_Directory_Entry) == 280;
tour_file_libc :: #system_library "libc";
tour_directory_open :: (path: *u8) -> *void #foreign tour_file_libc "opendir";
tour_directory_read :: (stream: *void) -> *Tour_Directory_Entry #foreign tour_file_libc "readdir";
tour_directory_close :: (stream: *void) -> s32 #foreign tour_file_libc "closedir";
tour_file_access :: (path: *u8, mode: s32) -> s32 #foreign tour_file_libc "access";
tour_file_errno :: () -> *s32 #foreign tour_file_libc "__errno_location";

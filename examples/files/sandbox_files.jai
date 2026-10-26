// Private adapter for the existing sandbox/WASI C bridge. This is not a
// File_Utilities WASM export. Both runtimes supply the same directory-entry ABI.
Demo_File_Visit_Info :: struct {
    full_name: string;
    short_name: string;
}

demo_visit_files :: (directory: string, recursive: bool, user_data: $T, visitor: (info: *Demo_File_Visit_Info, user_data: T)) -> bool {
    name := to_c_string(directory);

    defer free(name);
    stream := demo_directory_open(name);
    if !stream return false;

    success := true;

    while true {
        error := demo_file_errno();
        error.* = 0;
        entry := demo_directory_read(stream);
        if !entry {
            success = error.* == 0;
            break;
        }

        short_name := to_string(entry.name.data);
        if short_name == "." || short_name == ".." continue;

        full_name := tprint("%/%", directory, short_name);

        if entry.kind == 4 {
            if recursive && !demo_visit_files(full_name, recursive, user_data, visitor) {
                success = false;
                break;
            }
        } else if entry.kind == 8 {
            info := Demo_File_Visit_Info.{ full_name, short_name };
            visitor(*info, user_data);
        } else {
            // This private workspace walk does not follow symlinks or guess an
            // unknown entry's kind. Report unsupported traversal to its caller.
            success = false;
            break;
        }
    }

    if demo_directory_close(stream) != 0 success = false;

    return success;
}

demo_file_exists :: (path: string) -> bool {
    name := to_c_string(path);
    defer free(name);
    return demo_file_access(name, 0) == 0;
}

#scope_file
Demo_Directory_Entry :: struct {
    inode: u64;
    next_offset: s64;
    record_bytes: u16;
    kind: u8;
    name: [256] u8;
}

#assert size_of(Demo_Directory_Entry) == 280;
demo_file_libc :: #system_library "libc";
demo_directory_open :: (path: *u8) -> *void #foreign demo_file_libc "opendir";
demo_directory_read :: (stream: *void) -> *Demo_Directory_Entry #foreign demo_file_libc "readdir";
demo_directory_close :: (stream: *void) -> s32 #foreign demo_file_libc "closedir";
demo_file_access :: (path: *u8, mode: s32) -> s32 #foreign demo_file_libc "access";
demo_file_errno :: () -> *s32 #foreign demo_file_libc "__errno_location";

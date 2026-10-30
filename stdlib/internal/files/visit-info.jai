// What `visit_files` reports about one directory entry, and what `add_modtime_and_size_if_missing` fills in.
//
// - `short_name` is the entry's name and `full_name` the directory given to `visit_files` joined with it (with `/`). Both are
//   valid only during the callback; copy them to keep them.
// - `modification_time` and `size` are those of the file itself (of the target, for a link). `modification_time` is
//   `APOLLO_TIME_INVALID` when unknown.
// - `is_directory` and `is_symlink` say what it is. A link to a directory has both set.
// - `had_error` is set when the entry's details could not be read.
// - `descend_into_directory` is `true` for a directory the walk will enter. A callback that sets it to `false` skips that directory's contents.
//
// A zero `File_Visit_Info` has empty names, size `0` and every flag `false`.
//
// ```jai
// info: File_Visit_Info;
// assert(info.short_name.count == 0 && info.size == 0);
// assert(!info.had_error && !info.is_symlink && !info.is_directory && !info.descend_into_directory);
// ```
File_Visit_Info :: struct {
    short_name: string;
    full_name: string;
    modification_time: Apollo_Time;
    size: s64;
    had_error: bool;
    is_symlink: bool;
    is_directory: bool;
    descend_into_directory: bool;
}

#scope_file
#import "Basic";

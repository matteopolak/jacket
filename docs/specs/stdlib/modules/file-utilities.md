# File_Utilities

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `copy_directory :: (source: string, dest: string) -> bool`
- `file_list :: (path: string, recursive := false, follow_directory_symlinks := true) -> [] string`
- `File_Visit_Info :: struct`
- `Path :: struct`
- `parse_path :: (_s: string, reduce := true) -> Path`
- `reduce_path :: (path: Path) -> Path`
- `path_contains :: (path: Path, words: [] string) -> (contains: bool, position: s32)`
- `path_to_string :: (path: Path, max_words := -1) -> string`
- `how_much_of_path_exists_on_the_local_filesystem :: (path: Path) -> (fully_exists: bool, how_much_exists: s32)`
- `copy_to_permanent :: (path: Path) -> Path`
- `free_permanent :: (using path: Path)`

### Android-specific declarations

- `visit_files :: (dir_name: string, recursive: bool, user_data: $T, proc: (info: *File_Visit_Info, user_data: T), visit_files := true, visit_directories := false, visit_symlinks := true, follow_directory_symlinks := true) -> bool`
- `add_modtime_and_size_if_missing :: inline (info: *File_Visit_Info) -> bool`
- `file_modtime :: (file_path: string) -> (modtime: Apollo_Time, file_exists: bool)`
- `file_modtime_and_size :: (file_path: string) -> (modtime: Apollo_Time, size: s64, file_exists: bool)`
- `file_exists :: (path: string) -> bool`
- `is_directory :: (path: string) -> bool, success: bool`

### POSIX-specific declarations

- `copy_file :: (source: string, dest: string) -> bool`
- `atomic_file_replace_or_just_rename_if_destination_file_does_not_exist :: (old_filename: string, new_filename: string) -> bool`

### Unix-specific declarations

- `visit_files :: (dir_name: string, recursive: bool, user_data: $T, proc: (info: *File_Visit_Info, user_data: T), visit_files := true, visit_directories := false, visit_symlinks := true, follow_directory_symlinks := true) -> bool` (alias of `posix_visit_files`)
- `add_modtime_and_size_if_missing :: inline (info: *File_Visit_Info) -> bool` (alias of `posix_add_modtime_and_size_if_missing`)
- `file_exists :: (path: string) -> bool` (alias of `posix_file_exists`)
- `file_modtime :: (file_path: string) -> (modtime: Apollo_Time, file_exists: bool)` (alias of `posix_file_modtime`)
- `file_modtime_and_size :: (file_path: string) -> (modtime: Apollo_Time, size: s64, file_exists: bool)` (alias of `posix_file_modtime_and_size`)
- `is_directory :: (path: string) -> bool, success: bool` (alias of `posix_is_directory`)

### Windows-specific declarations

- `copy_file :: (source: string, dest: string, silent := false) -> bool`
- `visit_files :: (dir_name: string, recursive: bool, user_data: $T, proc: (info: *File_Visit_Info, user_data: T), visit_files := true, visit_directories := false, visit_symlinks := true, follow_directory_symlinks := true) -> bool`
- `add_modtime_and_size_if_missing :: inline (info: *File_Visit_Info) -> bool`
- `file_exists :: (file_path: string) -> bool`
- `file_modtime :: (file_path: string) -> (modtime: Apollo_Time, success: bool)`
- `file_modtime_and_size :: (file_path: string) -> (modtime: Apollo_Time, size: s64, success: bool)`
- `atomic_file_replace_or_just_rename_if_destination_file_does_not_exist :: (old_filename: string, new_filename: string) -> bool`
- `is_directory :: (path: string) -> bool, success: bool`

## Public type members

#### `File_Visit_Info` (struct)

- `short_name:   string`
- `full_name:    string`
- `modification_time:  Apollo_Time`
- `size:               s64`
- `had_error:    bool`
- `is_symlink:   bool`
- `is_directory: bool`
- `descend_into_directory: bool`

#### `Path` (struct)

- `header_string: string`
- `number_of_leading_slashes: int`
- `trailing_slash: bool`
- `words: [..] string`

## Behavioral contract and constraints

The path utilities split and manipulate path components as typed arrays and provide file-content helpers. Path separator and root semantics vary by platform; use the OS-aware operations for paths that cross platform boundaries. Returned arrays/strings are allocated according to the supplied/default allocator and must be freed or kept within temporary storage as documented by each signature. File errors are returned by the underlying file operations.


Platform coverage note: exports for Nintendo and PS5 targets are unspecified.

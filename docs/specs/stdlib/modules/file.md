# File

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

### Android-specific declarations

- `File :: struct`
- `file_open :: (name: string, for_writing := false, keep_existing_content := false, log_errors := true) -> File, bool`
- `file_close :: (file: *File)`
- `is_valid :: (file: File) -> bool`
- `file_length :: (file: File) -> (length: s64, success: bool)`
- `file_current_position :: (file: File) -> s64, bool`
- `file_set_position :: (file: File, pos: s64) -> bool`
- `file_tell :: (file: File) -> (success: bool, position: s64)`
- `file_seek :: (file: File, pos: s64, type: Seek_Type) -> (success: bool)`
- `file_read :: (file: File, vdata: *void, bytes_to_read: s64) -> (success: bool, total_read: s64)`
- `read_entire_file :: (file: File, zero_terminated := false) -> string, bool`
- `file_write :: (file: *File, data: *void, length: s64) -> bool`
- `file_flush :: (file: File) -> bool`

- `file_write :: inline (file: *File, data: string) -> bool`
- `file_write :: (file: *File, builder: *String_Builder, do_reset := true) -> bool`
- `write_entire_file :: inline (name: string, data: string) -> bool`
- `write_entire_file :: (name: string, data: *void, count: int) -> bool`
- `write_entire_file :: (name: string, builder: *String_Builder, do_reset := true) -> bool`
- `read_entire_file :: (name: string, zero_terminated := false, log_errors := true) -> string, bool`
- `USE_MAP_FILE :: (OS == .WINDOWS)`
- `STORE_MAPPED_FILENAMES :: false`
- `Map_File_Info :: struct`
- `string_to_map_file_info :: (data: string) -> Map_File_Info`
- `map_entire_file_start :: (name: string, log_errors := true) -> Map_File_Info, bool`
- `map_entire_file_end :: (file_info: *Map_File_Info)`
- `make_directory_if_it_does_not_exist :: (name: string, recursive := false) -> directory_now_exists: bool`
- `get_absolute_path :: inline (relative_path: string) -> string, success: bool`

### POSIX-specific declarations

- `file_move :: (name_old: string, name_new: string) -> bool`
- `file_delete :: (filename: string) -> bool`
- `file_link :: (linkpath: string, filepath: string) -> bool`
- `delete_directory :: (dirname: string) -> bool`

### Unix-specific declarations

- `File :: struct`
- `file_open :: (name: string, for_writing := false, keep_existing_content := false, log_errors := true) -> File, bool`
- `file_move :: (name_old: string, name_new: string) -> bool`
- `file_delete :: (filename: string) -> bool`
- `file_link :: (linkpath: string, filepath: string) -> bool`
- `file_close :: (file: *File)`
- `is_valid :: (file: File) -> bool`
- `file_read :: (f: File, vdata: *void, bytes_to_read: s64) -> (success: bool, total_read: s64)`
- `file_length :: (file: File) -> (length: s64, success: bool)`
- `file_current_position :: (f: File) -> s64, bool`
- `file_set_position :: (f: File, pos: s64) -> bool`
- `read_entire_file :: (f: File, zero_terminated := false) -> string, bool`
- `file_write :: (file: *File, data: *void, length: s64) -> bool`
- `delete_directory :: (dirname: string) -> bool`
- `file_seek :: (file: File, pos: s64, type: Seek_Type) -> (success: bool)`
- `file_tell :: (file: File) -> (success: bool, position: s64)`
- `file_flush :: (file: File) -> bool`
- `DUMB_LINUX_FAILURE_NUMBER :: 0x7fff_ffff_ffff_ffff`

### Windows-specific declarations

- `File :: struct`
- `file_open :: (name: string, for_writing := false, keep_existing_content := false, log_errors := true) -> File, bool`
- `file_close :: (file: *File)`
- `file_move :: (name_old: string, name_new: string) -> bool`
- `file_delete :: (filename: string) -> bool`
- `file_link :: (link : string, filepath : string) -> bool`
- `is_valid :: (file: File) -> bool`
- `file_read :: (f: File, vdata: *void, bytes_to_read: s64) -> (success: bool, total_read: s64)`
- `file_length :: (file: File) -> s64, bool`
- `file_current_position :: (f: File) -> s64, bool`
- `file_set_position :: (f: File, pos: s64) -> bool`
- `read_entire_file :: (f: File, zero_terminated := false) -> string, bool`
- `File_Map_Info :: struct`
- `map_file :: (name: string, log_errors := true) -> File_Map_Info, bool`
- `munmap_file :: (file_map: *File_Map_Info) -> bool`
- `CSIDL_APPDATA :: 0x001a`
- `KF_FLAG_CREATE :: 0x00008000`
- `FOLDERID_SavedGames_uid_string :: "4C5C32FF-BB9D-43b0-B5B4-2D72E54EAAA4"`
- `get_path_os_program_data :: () -> string`
- `get_path_os_saved_games :: () -> string`
- `file_write :: (file: *File, data: *void, length: s64) -> bool`
- `file_get_canonical_path :: (filepath: string) -> string`
- `file_get_canonical_path :: (file: File) -> string`
- `delete_directory :: (dirname: string) -> bool`
- `file_seek :: (file: File, pos: s64, type: Seek_Type) -> (success: bool)`
- `file_tell :: (file: File) -> (success: bool, position: s64)`
- `file_flush :: (file: File) -> bool`

## Public type members

### `Seek_Type` (contextual enum used by `file_seek`)

`Seek_Type` is declared in module scope rather than exported as a standalone module name. The `type` argument accepts these enum values, including as inferred enum literals:

```jai
Seek_Type :: enum {
    START   :: 0;
    CURRENT :: 1;
    END     :: 2;
}
```

### Android-specific type declarations

#### `File` (struct; Android)

- `kind: enum { FILE_SYSTEM; ASSET; }`
- Anonymous union: `handle: *FILE` or `asset: *AAsset`; `kind` selects the active member.

#### `Map_File_Info` (struct)

- `data: string`
- [STORE_MAPPED_FILENAMES] `filename: string`
- [USE_MAP_FILE] `map_info: File_Map_Info`

### Unix-specific type declarations

#### `File` (struct)

- `handle: *FILE`

### Windows-specific type declarations

#### `File` (struct)

- `handle: HANDLE`

#### `File_Map_Info` (struct)

- `file: File = .{ INVALID_HANDLE_VALUE }`
- `mapping: HANDLE = null`
- `data: string`

## Behavioral contract and constraints

The file module wraps file open/read/write/seek/close operations and, on Android, supports both ordinary filesystem files and packaged asset files. File handles must be closed through the corresponding API; exact read/write completion and OS error reporting are exposed by the operation results. Asset handles are Android-specific. The module marks some file behavior incomplete, so do not assume full parity with desktop files.

### Operation-level behavior

| Operation group | Observable behavior |
| --- | --- |
| `file_open` | Read mode opens binary read-only. Write mode with `keep_existing_content = false` creates or truncates. Write mode with `keep_existing_content = true` creates if needed and opens for read/write while preserving existing bytes. The success result determines whether the returned handle may be used; call `file_close` once when done. |
| `file_read` | Reads at most the requested byte count. A nonpositive count and a null destination fail with zero bytes. Reaching EOF before the requested count is still success and reports the shorter count; an I/O error reports failure plus bytes already read. |
| `read_entire_file` | Allocates returned storage; `zero_terminated` adds a NUL after the view's counted bytes. For a `File` argument it starts at the current file position and does not rewind, so callers needing the whole file should seek to the start first. At EOF, a shorter read can still succeed and the returned view count is the bytes actually read. |
| `file_write` | The operation reports whether the full requested byte count was written; a false result does not report a partial count. The `String_Builder` overload walks its buffers and, by default, resets the builder even if writing fails; pass `do_reset = false` to retain it. |
| Position/flush | `file_set_position` sets an absolute byte position; `file_seek` adds a signed byte offset relative to start, current position, or end. Unknown enum values and OS seek failures return `false`; the resulting position after failure is unspecified. `file_tell` returns success separately from the position, `(success, position)`, the reverse of `file_current_position`'s `(position, ok)`; both orders are kept because programs written against either rely on them. `file_flush` reports the platform flush result. Check each boolean before relying on the resulting position or persisted data. |
| Convenience whole-file calls | `write_entire_file` uses truncating write mode. `read_entire_file(name, ...)` opens and closes its own handle and optionally logs failures. Returned data is allocated; the caller owns it. |

The wrapper does not make concurrent access to one handle safe. Short reads at EOF are distinct from read errors; write failures expose no partial byte count. Exact native error codes remain platform-specific.

The public surface is a direct native binding: declared records, constants, and function signatures preserve the represented external ABI; this wrapper does not itself define a different ownership or error model. Native status/error values and handle lifetimes follow the external API. ABI compatibility across external-library versions is not specified unless a version note appears above.

External libraries/frameworks named in source: `ole32`, `shell32`.


Platform coverage note: exports for Nintendo and PS5 targets are unspecified.

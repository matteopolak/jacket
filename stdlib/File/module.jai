// File: opening, reading, writing and managing files and directories by path.
//
// A `File` is a handle from `file_open`; `read_entire_file` and `write_entire_file` do a whole file in one call, and
// `map_entire_file_start` borrows the content. Names are UTF-8 paths, with `/` as separator on every target (Windows
// accepts `\` as well). Every procedure reports failure by its result and does not stop the program. Memory a procedure
// returns comes from `context.allocator` and the caller frees it. Walking directories, copying and metadata are in
// `File_Utilities`.
//
// ```jai
// #import "Extensions/Environment";
// base := environment_variable("TMPDIR");
// if base == "" base = environment_variable("TEMP");
// if base == "" base = "/tmp";
// path := tprint("%/jai-doc-file-module.txt", base);
// defer file_delete(path);
//
// assert(write_entire_file(path, "hello\n"));
// text, ok := read_entire_file(path);
// assert(ok && text == "hello\n");
// free(text);
// ```

File :: Impl.File;
USE_MAP_FILE :: Impl.USE_MAP_FILE;
STORE_MAPPED_FILENAMES :: Impl.STORE_MAPPED_FILENAMES;
#if OS_IS_UNIX DUMB_LINUX_FAILURE_NUMBER :: Impl.DUMB_LINUX_FAILURE_NUMBER;
Map_File_Info :: Impl.Map_File_Info;
file_open :: Impl.file_open;
file_close :: Impl.file_close;
is_valid :: Impl.is_valid;
file_read :: Impl.file_read;
file_current_position :: Impl.file_current_position;
file_tell :: Impl.file_tell;
file_set_position :: Impl.file_set_position;
file_seek :: Impl.file_seek;
file_length :: Impl.file_length;
file_flush :: Impl.file_flush;
file_write :: Impl.file_write;
read_entire_file :: Impl.read_entire_file;
write_entire_file :: Impl.write_entire_file;
file_move :: Impl.file_move;
file_delete :: Impl.file_delete;
file_link :: Impl.file_link;
delete_directory :: Impl.delete_directory;
make_directory_if_it_does_not_exist :: Impl.make_directory_if_it_does_not_exist;
get_absolute_path :: Impl.get_absolute_path;
string_to_map_file_info :: Impl.string_to_map_file_info;
map_entire_file_start :: Impl.map_entire_file_start;
map_entire_file_end :: Impl.map_entire_file_end;

#if OS == .WINDOWS {
    File_Map_Info :: Impl.File_Map_Info;
    map_file :: Impl.map_file;
    munmap_file :: Impl.munmap_file;
    CSIDL_APPDATA :: Impl.CSIDL_APPDATA;
    KF_FLAG_CREATE :: Impl.KF_FLAG_CREATE;
    FOLDERID_SavedGames_uid_string :: Impl.FOLDERID_SavedGames_uid_string;
    get_path_os_program_data :: Impl.get_path_os_program_data;
    get_path_os_saved_games :: Impl.get_path_os_saved_games;
    file_get_canonical_path :: Impl.file_get_canonical_path;
}

#scope_file
Impl :: #import "internal/files/file";

// The size of a [Package_Header] in the file, in bytes.
KNOWN_HEADER_SIZE :: 64;
// The size of a [Table_Of_Contents] in the file, before the entry records, in bytes.
KNOWN_TABLE_OF_CONTENTS_SIZE :: 64;

// One entry of a package. When read, `name` and `data` point into the package's bytes.
Entry_Info :: struct {
    name: string;
    data: [] u8; // Empty after [load_table_of_contents_only], which reads no entry data.
    offset_from_start_of_file: s64;
    size_of_entry: s64;
}

// A package being written. Set it up with [init], fill it with [add], save it with [write] and
// free it with [deinit].
Create_Package :: struct {
    data: B.String_Builder;
    entries: [..] Entry_Info;
    pool: Pools.Pool;
}

// The first [KNOWN_HEADER_SIZE] bytes of a package file.
Package_Header :: struct {
    magic: u32;
    version: u32; // 1; readers reject later versions.
    flags: u32;
    reserved0: u32;
    table_of_contents_offset: s64; // Where the [Table_Of_Contents] starts, from the start of the file.
    reserved: [5] u64;
}

// The start of a package's table of contents, before its entry records.
Table_Of_Contents :: struct {
    table_of_contents_magic: u32;
    reserved0: u32;
    number_of_entries: s64;
    reserved: [6] u64;
}

// A package being read. `lookup` maps each name to its entry; when two entries share a name,
// the first one wins.
Load_Package :: struct {
    header: Package_Header;
    data: [] u8; // The file's bytes, or only the table of contents after [load_table_of_contents_only].
    table_of_contents: Table_Of_Contents;
    entries: [] Entry_Info;
    lookup: HT.Table(string, *Entry_Info);
    owns_data: bool; // `data` was allocated by a reader and [free_package] frees it
}

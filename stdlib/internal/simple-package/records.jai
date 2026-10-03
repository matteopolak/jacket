KNOWN_HEADER_SIZE :: 64;
KNOWN_TABLE_OF_CONTENTS_SIZE :: 64;

Entry_Info :: struct {
    name: string;
    data: [] u8;
    offset_from_start_of_file: s64;
    size_of_entry: s64;
}

Create_Package :: struct {
    data: B.String_Builder;
    entries: [..] Entry_Info;
    pool: Pools.Pool;
}

Package_Header :: struct {
    magic: u32;
    version: u32;
    flags: u32;
    reserved0: u32;
    table_of_contents_offset: s64;
    reserved: [5] u64;
}

Table_Of_Contents :: struct {
    table_of_contents_magic: u32;
    reserved0: u32;
    number_of_entries: s64;
    reserved: [6] u64;
}

Load_Package :: struct {
    header: Package_Header;
    data: [] u8;
    table_of_contents: Table_Of_Contents;
    entries: [] Entry_Info;
    lookup: HT.Table(string, *Entry_Info);
}

Type_Id :: #type,distinct u32;
Item_Id :: #type,isa Type_Id;
Codeview_File_Checksum_Id :: #type,isa u32;
Codeview_String_Table_Id :: #type,isa u32;
// The type of a column number in a line table: 16 bits.
Codeview_Column_Type :: u16;

// The common header of every decoded type record; `kind` says which record struct embeds it.
Codeview_Type :: struct {
    kind: Codeview_Leaf_Type;
}

// The common header of every decoded symbol record; `symbol_type` says which record struct embeds it.
Codeview_Symbol :: struct {
    symbol_type: Codeview_Symbol_Type;
}

// A source file in the C13 file-checksum subsection: its ID, the string-table offset of its name, and its hash.
Codeview_File_Checksum :: struct {
    id: Codeview_File_Checksum_Id;
    filename_id: Codeview_String_Table_Id;
    checksum_type: Checksum_Type;
    checksum: [] u8;
    Checksum_Type :: enum u8 {
        NONE :: 0;
        MD5 :: 1;
        SHA1 :: 2;
        SHA_256 :: 3;
    }
}

// The line rows of one source file inside a [Codeview_Line_Info_Sequence].
//
// `columns` is empty without column data, has one entry per row, or two (start and end) per row.
Codeview_Lines_File_Block :: struct {
    file_id: Codeview_File_Checksum_Id;
    lines: [] Codeview_Line_Info;
    columns: [] Codeview_Column_Type;
}

// A contiguous range of code and its line rows, grouped by source file.
//
// Row offsets in the file blocks are relative to `text_offset`.
Codeview_Line_Info_Sequence :: struct {
    text_offset: u32;
    segment_index: u16;
    flags: Flags;
    length_in_bytes: u32;
    file_blocks: [..] Codeview_Lines_File_Block;
    Flags :: enum u16 {
        HAS_COLUMNS :: 0x0001;
    }
}

// Where an inlined function is defined: the file and first line of `inlinee`, and any extra files it spans.
Codeview_Inlinee_Line_Info :: struct {
    inlinee: Type_Id;
    file_id: Codeview_File_Checksum_Id;
    source_line: u32;
    extra_files: [] u32;
}

// The decoded CodeView data of one module: type and ID records, symbols, and the C13 line, checksum and string tables.
//
// `types` and `ids` are indexed by the record index minus `0x1000`. The string table holds NUL-terminated names addressed by [Codeview_String_Table_Id].
Codeview_Debug_Info :: struct {
    types: [] *Codeview_Type;
    ids: [] *Codeview_Type;
    symbols: [..] *Codeview_Symbol;
    string_table: [] u8;
    file_checksums: [..] Codeview_File_Checksum;
    line_info: [..] Codeview_Line_Info_Sequence;
    inlinee_line_info: [..] Codeview_Inlinee_Line_Info;
}

Type_Id :: #type,distinct u32;
Item_Id :: #type,isa Type_Id;
Codeview_File_Checksum_Id :: #type,isa u32;
Codeview_String_Table_Id :: #type,isa u32;
Codeview_Column_Type :: u16;

Codeview_Type :: struct {
    kind: Codeview_Leaf_Type;
}

Codeview_Symbol :: struct {
    symbol_type: Codeview_Symbol_Type;
}

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

Codeview_Lines_File_Block :: struct {
    file_id: Codeview_File_Checksum_Id;
    lines: [] Codeview_Line_Info;
    columns: [] Codeview_Column_Type;
}

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

Codeview_Inlinee_Line_Info :: struct {
    inlinee: Type_Id;
    file_id: Codeview_File_Checksum_Id;
    source_line: u32;
    extra_files: [] u32;
}

Codeview_Debug_Info :: struct {
    types: [] *Codeview_Type;
    ids: [] *Codeview_Type;
    symbols: [..] *Codeview_Symbol;
    string_table: [] u8;
    file_checksums: [..] Codeview_File_Checksum;
    line_info: [..] Codeview_Line_Info_Sequence;
    inlinee_line_info: [..] Codeview_Inlinee_Line_Info;
}

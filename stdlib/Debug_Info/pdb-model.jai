// Reviewed public PDB aggregates; contextual map storage is independently designed.
Pdb :: struct {
    header: Pdb_Header;
    signature: GUID;
    name_table: Name_Table;
    contains_id_stream: bool;
    no_type_merge: bool;
    minimal_debug_info: bool;
    tpi: [] *Codeview_Type;
    ipi: [] *Codeview_Type;
    msf: Msf;
}

Pdb_Header :: struct {
    version: Pdb_Version;
    signature: u32;
    age: u32;
}

Tpi_Stream_Header :: struct {
    version: Tpi_Stream_Version;
    header_size: u32;
    type_index_begin: u32;
    type_index_end: u32;
    type_record_bytes: u32;
    hash_stream_index: u16;
    hash_aux_stream_index: u16;
    hash_key_size: u32;
    num_hash_buckets: u32;
    hash_value_buffer_offset: s32;
    hash_value_buffer_length: u32;
    index_offset_buffer_offset: s32;
    index_offset_buffer_length: u32;
    hash_adj_buffer_offset: s32;
    hash_adj_buffer_length: u32;
}

Pdb_Version :: enum u32 {
    VC2 :: 19941610;
    VC4 :: 19950623;
    VC41 :: 19950814;
    VC50 :: 19960307;
    VC98 :: 19970604;
    VC70 :: 20000404;
    VC70Dep :: 19990604;
    VC80 :: 20030901;
    VC110 :: 20091201;
    VC140 :: 20140508;
}

Tpi_Stream_Version :: enum u32 {
    V40 :: 19950410;
    V41 :: 19951122;
    V50 :: 19961031;
    V70 :: 19990903;
    V80 :: 20040203;
}

#scope_file

GUID :: struct {
    Data1: u32;
    Data2: u16;
    Data3: u16;
    Data4: [8] u8;
}

String_Offset :: #type,distinct s32;
Name_Index :: #type,distinct u32;

Name_Table :: struct {
    buffer: [] u8;
    string_to_name_map: Pdb_Map(String_Offset, Name_Index);
}

Pdb_Map :: struct(K: Type, V: Type) {
    keys: [..] K;
    values: [..] V;
    present: Int_Set;
    deleted: Int_Set;
}

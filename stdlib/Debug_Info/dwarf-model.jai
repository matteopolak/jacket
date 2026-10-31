// A named type from the DWARF data: its tag, name and size.
Debug_Type :: struct {
    offset: s64;
    tag: Dwarf_Tag;
    name: string;
    size: u64;
    base_type: Dwarf_Base_Type;
}

// A variable or parameter: its name, where it was declared and its type.
Declaration :: struct {
    tag: Dwarf_Tag;
    name: string;
    decl_file_id: Dwarf_File_Id;
    decl_line: u32;
    type_id: u32;
    kind: enum {
        PARAMETER;
        VARIABLE;
    }
}

// A scope inside a function: its code range, the declarations in it and the scopes nested in it.
Lexical_Block :: struct {
    low_pc: u64;
    high_pc: u64;
    declarations: [..] Declaration;
    children: [..] *Lexical_Block;
}

// A function: its name, code range (`low_pc` to `high_pc`), declaration site and scope tree.
Subprogram :: struct {
    name: string;
    linkage_name: string;
    external: bool;
    decl_file_id: Dwarf_File_Id;
    decl_line: u32;
    type_id: u32;
    is_main: bool;
    calling_convention: u8;
    low_pc: u64;
    high_pc: u64;
    all_blocks: BA.Bucket_Array(Lexical_Block, 8);
    root_block: *Lexical_Block;
}

// One source file as compiled: its producer, language, functions, global variables, types and line table.
Compilation_Unit :: struct {
    producer: string;
    language: Dwarf_Language;
    name: string;
    statement_list: u64;
    low_pc: u64;
    high_pc: u64;
    types: [..] Debug_Type;
    line_info: Line_Info;
    subprograms: BA.Bucket_Array(Subprogram, 32);
    globals: [..] Declaration;
}

// Everything read from a program's DWARF sections, one [Compilation_Unit] per compiled source file.
Debug_Info :: struct {
    compilation_units: [..] Compilation_Unit;
}

#scope_file
BA :: #import "Bucket_Array";

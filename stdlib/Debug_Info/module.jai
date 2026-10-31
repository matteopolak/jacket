// Reads the debug information in a compiled program: DWARF (Linux and macOS) and CodeView in PDB files (Windows).
//
// - [parse_dwarf_debug_info] turns the DWARF sections of an executable into a [Debug_Info] of compilation units, functions, variables and line tables.
// - [get_line_mapping_for_address] finds the source line for a code address in a [Line_Info].
// - [parse_pdb] and [parse_msf] open a PDB file and its stream container; [get_stream_data] reads one stream.
//
// ```jai
// info: Line_Info;
// sequence: Line_Info_Sequence;
// array_add(*sequence.addresses, 0x1000);
// array_add(*sequence.addresses, 0x1010);
// array_add(*sequence.mappings, .{ file_id = 1, line = 42 });
// array_add(*sequence.mappings, .{ is_end = true });
// array_add(*info.sequences, sequence);
//
// mapping, found := get_line_mapping_for_address(info, 0x1004);
// assert(found && mapping.line == 42);
// ```

Int_Set :: struct {
    BPW :: 32;
    words: [] u32;
}

Dwarf_File_Id :: #type,isa u32;

// The line table of one compilation unit: its directories and files, and the address sequences that map code to lines.
Line_Info :: struct {
    dirs: [..] string;
    files: [..] File_Info;
    sequences: [..] Line_Info_Sequence;
}

// A source file in a [Line_Info]: its directory (an index into `dirs`, 0 for none) and its path.
File_Info :: struct {
    dir_id: u64;
    path: string;
}

// One run of the line program. `addresses[i]` starts the source position `mappings[i]`; the last mapping has `is_end` set and closes the run.
Line_Info_Sequence :: struct {
    addresses: [..] u64;
    mappings: [..] Line_Mapping;
}

// The source position of a code address: file, line and column, plus what kind of position it is (statement start, end of prologue, and so on).
Line_Mapping :: struct {
    file_id: Dwarf_File_Id;
    line: u32;
    column: u32;
    is_end: bool;
    flags: Flags;
    Flags :: enum_flags {
        STATEMENT;
        BASIC_BLOCK;
        PROLOGUE_END;
        EPILOGUE_BEGIN;
    }
}

Codeview_Line_Info :: struct {
    text_offset: u32;
    line_and_delta_and_flag: u32;
}

Codeview_Address_Gap :: struct {
    start_offset: u16;
    range_bytes: u16;
}

Codeview_Address_Range :: struct {
    start_offset: u32;
    start_section: u16;
    range_bytes: u16;
}

Codeview_Symbol_Defrange_Register_Rel :: struct {
    base_register: u16;
    spilled_udt_member_and_offset_parent: u16;
    base_pointer_offset: u32;
    range: Codeview_Address_Range;
    gaps: [] Codeview_Address_Gap;
}

// One DWARF call frame instruction, as found in a [Frame_Description_Entry].
Call_Frame_Address_Instruction :: struct {
    type: Call_Frame_Address_Instruction_Type;
    arg1: Argument;
    arg2: Argument;
    Argument :: union {
        uvalue: u64;
        svalue: s64;
    }
}

// The DWARF call frame information shared by a group of functions.
Frame_Common_Information_Entry :: struct {
    length: u64;
    id: u32;
    version: u8;
    augmentation: string;
    eh_data: u64;
    code_alignment_factor: u64;
    data_alignment_factor: s64;
    return_address_register: u64;
    augmentation_data: [] u8;
    initial_instructions: [] Call_Frame_Address_Instruction;
    offset: u64;
}

// The DWARF call frame instructions for the code range `pc_begin` to `pc_begin + pc_range`.
Frame_Description_Entry :: struct {
    length: u64;
    cie_offset: u64;
    pc_begin: u64;
    pc_range: u64;
    augmentation_data: [] u8;
    instructions: [] Call_Frame_Address_Instruction;
}

// The operation of a [Call_Frame_Address_Instruction]: how to find the caller's registers at one point in a function.
Call_Frame_Address_Instruction_Type :: enum u8 {
    ADVANCE_LOC :: 0x40;
    OFFSET :: 0x80;
    RESTORE :: 0xc0;
    NOP :: 0x00;
    SET_LOC :: 0x01;
    ADVANCE_LOC1 :: 0x02;
    ADVANCE_LOC2 :: 0x03;
    ADVANCE_LOC4 :: 0x04;
    OFFSET_EXTENDED :: 0x05;
    RESTORE_EXTENDED :: 0x06;
    UNDEFINED :: 0x07;
    SAME_VALUE :: 0x08;
    REGISTER :: 0x09;
    REMEMBER_STATE :: 0x0a;
    RESTORE_STATE :: 0x0b;
    DEF_CFA :: 0x0c;
    DEF_CFA_REGISTER :: 0x0d;
    DEF_CFA_OFFSET :: 0x0e;
    DEF_CFA_EXPRESSION :: 0x0f;
    OFFSET_EXTENDED_SF :: 0x10;
    DEF_CFA_SF :: 0x11;
    DEF_CFA_OFFSET_SF :: 0x12;
    VAL_OFFSET :: 0x13;
    VAL_OFFSET_SF :: 0x14;
    VAL_EXPRESSION :: 0x15;
    LO_USER :: 0x1b;
    HI_USER :: 0x3e;
}

#load "../internal/developer-tools/dwarf-frame.jai";
#load "../internal/developer-tools/codeview-lines.jai";
#load "../internal/developer-tools/codeview-symbol-fields.jai";
#load "../internal/developer-tools/debug-int-set.jai";
#load "../internal/developer-tools/dwarf-line-query.jai";
#load "../internal/developer-tools/msf-streams.jai";
#load "../internal/developer-tools/msf-page-bits.jai";
#load "dwarf-values.jai";
#load "dwarf-model.jai";
#load "dwarf-line-header.jai";
#load "../internal/developer-tools/dwarf-model-query.jai";
#load "codeview-values.jai";
#load "codeview-model.jai";
#load "../internal/developer-tools/codeview-string-query.jai";
#load "../internal/developer-tools/debug-formats/dwarf-line-wire.jai";
#load "../internal/developer-tools/dwarf-public.jai";
#load "../internal/developer-tools/msf-wire.jai";
#load "codeview-type-model.jai";
#load "../internal/developer-tools/codeview-types.jai";
#load "pdb-model.jai";
#load "../internal/developer-tools/pdb-parser.jai";
#load "../internal/developer-tools/codeview-truncate.jai";
#load "codeview-symbol-model.jai";
#load "../internal/developer-tools/codeview-inline.jai";
#load "../internal/developer-tools/codeview-symbol-parser.jai";

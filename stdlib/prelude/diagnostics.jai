// Canonical bootstrap diagnostics declarations.
Source_Code_Location :: struct {
    fully_pathed_filename: string;
    line_number: s64;
    character_number: s64;
}

Source_Code_Range :: struct {
    fully_pathed_filename: string;
    line_number_start: s64;
    line_number_end: s64;
    character_number_start: s64;
    character_number_end: s64;
}

Stack_Trace_Procedure_Info :: struct {
    name: string;
    location: Source_Code_Location;

    procedure_address: *void;

}

Stack_Trace_Node :: struct {
    next: *Stack_Trace_Node;
    info: *Stack_Trace_Procedure_Info;
    hash: u64;
    call_depth: u32;
    line_number: u32;
}

Log_Level :: enum u8 {
    NORMAL :: 0;
    VERBOSE :: 1;
    VERY_VERBOSE :: 2;
}

Log_Flags :: enum_flags u32 {
    NONE :: 0x0;
    ERROR :: 0x1;
    WARNING :: 0x2;
    CONTENT :: 0x4;
    TO_FILE_ONLY :: 0x8;
    VERBOSE_ONLY :: 0x10;
    VERY_VERBOSE_ONLY :: 0x20;
    TOPIC_ONLY :: 0x40;
}

Log_Section :: struct {
    name: string;
}

Log_Info :: struct {
    source_identifier: u64;
    location: Source_Code_Location;
    common_flags: Log_Flags;
    user_flags: u32;
    section: *Log_Section;
}

Logger :: #type (message: string, data: *void, info: Log_Info);

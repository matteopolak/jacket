// Reviewed Jai aggregates; serialized payloads are decoded separately.
Codeview_Symbol_Compile :: struct {
    using base: Codeview_Symbol = .{ symbol_type = .COMPILE };
    target_cpu: u8;
    language: u8;
    flags: u16;
    compiler_version: string;
}

Codeview_Symbol_Compile3 :: struct {
    using base: Codeview_Symbol = .{ symbol_type = .COMPILE3 };
    language: u8;
    flags1: u8;
    flags2: u8;
    reserved: u8;
    target_cpu: u16;
    fe_version: Version;
    be_version: Version;
    compiler_version: string;
    Version :: struct {
        major: u16;
        minor: u16;
        build: u16;
        qfe: u16;
    }
}

Codeview_Symbol_Procedure :: struct {
    using base: Codeview_Symbol = .{ symbol_type = .GPROC32 };
    parent: u32;
    end: u32;
    next: u32;
    length: u32;
    debug_start_offset: u32;
    debug_end_offset: u32;
    type: Type_Id;
    offset: u32;
    segment_index: u16;
    flags: Flags;
    name: string;
    Flags :: enum_flags u8 {
        CV_PFLAG_NOFPO :: 0x01;
        CV_PFLAG_INT :: 0x02;
        CV_PFLAG_FAR :: 0x04;
        CV_PFLAG_NEVER :: 0x08;
        CV_PFLAG_NOTREACHED :: 0x10;
        CV_PFLAG_CUST_CALL :: 0x20;
        CV_PFLAG_NOINLINE :: 0x40;
        CV_PFLAG_OPTDBGINFO :: 0x80;
    }
    inlinesites: [..] *Codeview_Symbol_Inline_Site;
}

Codeview_Symbol_Inline_Site :: struct {
    using base: Codeview_Symbol = .{ symbol_type = .INLINESITE };
    parent: u32;
    end: u32;
    inlinee: Item_Id;
    binary_annotations: [..] Codeview_Binary_Annotation;
    children: [..] *Codeview_Symbol_Inline_Site;
}

Codeview_Binary_Annotation :: struct {
    opcode: Codeview_Binary_Annotation_Opcode;
    operand1: Operand;
    operand2: Operand;
    Operand :: union {
        unsigned: u32;
        signed: s32;
    }
}

Codeview_Parser_State :: struct {
    data: [] u8;
    offset: s64;
    success: bool;
    verbose: bool;
}

Codeview_Type_Parser_State :: struct {
    types: [..] *Codeview_Type;
}

Codeview_Symbol_Parser_State :: struct {
    info: *Codeview_Debug_Info;
    current_procedure: *Codeview_Symbol_Procedure;
    open_inlinesites: [..] *Codeview_Symbol_Inline_Site;
}

Codeview_Binary_Annotation_Opcode :: enum u8 {
    INVALID :: 0x00;
    CODE_OFFSET :: 0x01;
    CHANGE_CODE_OFFSET_BASE :: 0x02;
    CHANGE_CODE_OFFSET :: 0x03;
    CHANGE_CODE_LENGTH :: 0x04;
    CHANGE_FILE :: 0x05;
    CHANGE_LINE_OFFSET :: 0x06;
    CHANGE_LINE_END_DELTA :: 0x07;
    CHANGE_RANGE_KIND :: 0x08;
    CHANGE_COLUMN_START :: 0x09;
    CHANGE_COLUMN_END_DELTA :: 0x0a;
    CHANGE_CODE_OFFSET_AND_LINE_OFFSET :: 0x0b;
    CHANGE_CODE_LENGTH_AND_CODE_OFFSET :: 0x0c;
    CHANGE_COLUMN_END :: 0x0d;
}

Codeview_Symbol_Section :: enum u32 {
    SYMBOLS :: 0xF1;
    LINES :: 0xF2;
    STRING_TABLE :: 0xF3;
    FILE_CHECKSUMS :: 0xF4;
    FRAME_DATA :: 0xF5;
    INLINEELINES :: 0xF6;
    CROSS_SCOPE_IMPORTS :: 0xF7;
    CROSS_SCOPE_EXPORTS :: 0xF8;
    IL_LINES :: 0xF9;
    FUNC_MDTOKEN_MAP :: 0xFA;
    TYPE_MDTOKEN_MAP :: 0xFB;
    MERGED_ASSEMBLYINPUT :: 0xFC;
    COFF_SYMBOL_RVA :: 0xFD;
    IGNORE :: 0x80000000;
}

Codeview_Inlinee_Source_Variant :: enum u32 {
    NORMAL :: 0;
    EX :: 1;
}

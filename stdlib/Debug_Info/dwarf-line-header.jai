Line_Program_Header :: struct {
    size: u32;
    version: u16;
    header_length: u32 #align 2;
    minimum_instruction_length: u8;
    maximum_operations_per_instruction: u8;
    default_is_stmt: bool;
    line_base: s8;
    line_range: u8;
    opcode_base: u8;
}

//! Numeric values of the node kinds, message kinds and instruction flags the exported records use.

/// Reviewed `Code_Type_Instantiation.inst_flags` ABI (independent of AST enum tags).
pub(super) mod inst_flags {
    pub const VARARGS: i64 = 0x1;
    pub const RESIZABLE: i64 = 0x2;
    pub const TYPE_OF_PROCEDURE: i64 = 0x4;
    pub const TYPE_DIRECTIVE: i64 = 0x8;
    pub const TYPE_DIRECTIVE_DISTINCT: i64 = 0x10;
    pub const TYPE_DIRECTIVE_ISA: i64 = 0x20;
    pub const ARRAY_VIEW: i64 = 0x40;
}

/// Values of `Message.kind`.
pub mod message_kind {
    pub const FILE: i64 = 1;
    pub const IMPORT: i64 = 2;
    pub const FAILED_IMPORT: i64 = 3;
    pub const TYPECHECKED: i64 = 5;
    pub const PERFORMANCE_REPORT: i64 = 9;
}

/// Values of `Code_Node.Kind`.
pub(super) mod node {
    pub const BLOCK: i64 = 1;
    pub const LITERAL: i64 = 2;
    pub const IDENT: i64 = 3;
    pub const UNARY_OPERATOR: i64 = 4;
    pub const BINARY_OPERATOR: i64 = 5;
    pub const PROCEDURE_BODY: i64 = 6;
    pub const PROCEDURE_CALL: i64 = 7;
    pub const CONTEXT: i64 = 8;
    pub const WHILE: i64 = 9;
    pub const IF: i64 = 10;
    pub const LOOP_CONTROL: i64 = 11;
    pub const CASE: i64 = 12;
    pub const RETURN: i64 = 14;
    pub const FOR: i64 = 15;
    pub const TYPE_INSTANTIATION: i64 = 17;
    pub const ENUM: i64 = 18;
    pub const PROCEDURE_HEADER: i64 = 19;
    pub const STRUCT: i64 = 20;
    pub const DECLARATION: i64 = 25;
    pub const COMMA_SEPARATED_ARGUMENTS: i64 = 21;
    pub const COMPOUND_DECLARATION: i64 = 48;
    pub const CAST: i64 = 26;
    pub const DIRECTIVE_IMPORT: i64 = 27;
    pub const DIRECTIVE_RUN: i64 = 31;
    pub const DIRECTIVE_EXISTS: i64 = 55;
    pub const DIRECTIVE_CODE: i64 = 32;
    pub const DIRECTIVE_LOCATION: i64 = 44;
    pub const NOTE: i64 = 40;
    pub const DEFER: i64 = 49;
    pub const USING: i64 = 50;
    pub const PUSH_CONTEXT: i64 = 39;
    pub const DIRECTIVE_MODULE_PARAMETERS: i64 = 45;
    pub const DIRECTIVE_ADD_CONTEXT: i64 = 46;
    pub const PLACEHOLDER: i64 = 51;
    pub const DIRECTIVE_BYTES: i64 = 23;
    pub const DIRECTIVE_THIS: i64 = 28;
    pub const DIRECTIVE_LOAD: i64 = 30;
    pub const ASM: i64 = 34;
    pub const DIRECTIVE_BAKE: i64 = 35;
    pub const DIRECTIVE_PLACE: i64 = 41;
    pub const DIRECTIVE_SCOPE: i64 = 42;
    pub const DIRECTIVE_MODIFY: i64 = 36;
    pub const DIRECTIVE_THROUGH: i64 = 29;
    pub const DIRECTIVE_COMPILE_TIME: i64 = 47;
    pub const DIRECTIVE_PROCEDURE_NAME: i64 = 53;
    pub const DIRECTIVE_OVERLAY: i64 = 58;
    pub const DIRECTIVE_INSERT: i64 = 52;
}

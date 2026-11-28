//! Numeric values of the node kinds, message kinds and instruction flags the exported records use.

/// Reviewed `Code_Type_Instantiation.inst_flags` ABI (independent of AST enum tags).
pub(super) mod inst_flags {
    pub(crate) const VARARGS: i64 = 0x1;
    pub(crate) const RESIZABLE: i64 = 0x2;
    pub(crate) const TYPE_OF_PROCEDURE: i64 = 0x4;
    pub(crate) const TYPE_DIRECTIVE: i64 = 0x8;
    pub(crate) const TYPE_DIRECTIVE_DISTINCT: i64 = 0x10;
    pub(crate) const TYPE_DIRECTIVE_ISA: i64 = 0x20;
    pub(crate) const ARRAY_VIEW: i64 = 0x40;
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
    pub(crate) const BLOCK: i64 = 1;
    pub(crate) const LITERAL: i64 = 2;
    pub(crate) const IDENT: i64 = 3;
    pub(crate) const UNARY_OPERATOR: i64 = 4;
    pub(crate) const BINARY_OPERATOR: i64 = 5;
    pub(crate) const PROCEDURE_BODY: i64 = 6;
    pub(crate) const PROCEDURE_CALL: i64 = 7;
    pub(crate) const CONTEXT: i64 = 8;
    pub(crate) const WHILE: i64 = 9;
    pub(crate) const IF: i64 = 10;
    pub(crate) const LOOP_CONTROL: i64 = 11;
    pub(crate) const CASE: i64 = 12;
    pub(crate) const RETURN: i64 = 14;
    pub(crate) const FOR: i64 = 15;
    pub(crate) const TYPE_INSTANTIATION: i64 = 17;
    pub(crate) const ENUM: i64 = 18;
    pub(crate) const PROCEDURE_HEADER: i64 = 19;
    pub(crate) const STRUCT: i64 = 20;
    pub(crate) const DECLARATION: i64 = 25;
    pub(crate) const COMMA_SEPARATED_ARGUMENTS: i64 = 21;
    pub(crate) const COMPOUND_DECLARATION: i64 = 48;
    pub(crate) const CAST: i64 = 26;
    pub(crate) const DIRECTIVE_IMPORT: i64 = 27;
    pub(crate) const DIRECTIVE_RUN: i64 = 31;
    pub(crate) const DIRECTIVE_EXISTS: i64 = 55;
    pub(crate) const DIRECTIVE_CODE: i64 = 32;
    pub(crate) const DIRECTIVE_LOCATION: i64 = 44;
    pub(crate) const NOTE: i64 = 40;
    pub(crate) const DEFER: i64 = 49;
    pub(crate) const USING: i64 = 50;
    pub(crate) const PUSH_CONTEXT: i64 = 39;
    pub(crate) const DIRECTIVE_MODULE_PARAMETERS: i64 = 45;
    pub(crate) const DIRECTIVE_ADD_CONTEXT: i64 = 46;
    pub(crate) const PLACEHOLDER: i64 = 51;
    pub(crate) const DIRECTIVE_BYTES: i64 = 23;
    pub(crate) const DIRECTIVE_THIS: i64 = 28;
    pub(crate) const DIRECTIVE_LOAD: i64 = 30;
    pub(crate) const ASM: i64 = 34;
    pub(crate) const DIRECTIVE_BAKE: i64 = 35;
    pub(crate) const DIRECTIVE_PLACE: i64 = 41;
    pub(crate) const DIRECTIVE_SCOPE: i64 = 42;
    pub(crate) const DIRECTIVE_MODIFY: i64 = 36;
    pub(crate) const DIRECTIVE_THROUGH: i64 = 29;
    pub(crate) const DIRECTIVE_COMPILE_TIME: i64 = 47;
    pub(crate) const DIRECTIVE_PROCEDURE_NAME: i64 = 53;
    pub(crate) const DIRECTIVE_OVERLAY: i64 = 58;
    pub(crate) const DIRECTIVE_INSERT: i64 = 52;
}

//! Text-to-tag conversions belong at the lexical boundary.
macro_rules! tags {
    ($name:ident { $($variant:ident => $text:literal),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum $name { $($variant),+ }
        impl $name {
            pub const SPELLINGS: &'static [(&'static str, Self)] = &[$(($text, Self::$variant)),+];
            pub fn from_spelling(text: &str) -> Option<Self> {
                match text { $($text => Some(Self::$variant)),+, _ => None }
            }
            pub const fn spelling(self) -> &'static str { match self { $(Self::$variant => $text),+ } }
        }
    }
}
tags!(Keyword {
    For => "for", If => "if", Ifx => "ifx", Then => "then", Else => "else", Case => "case",
    Return => "return", Struct => "struct", While => "while", Break => "break",
    Continue => "continue", Remove => "remove", Using => "using", Defer => "defer",
    SizeOf => "size_of", TypeOf => "type_of", CodeOf => "code_of", InitializerOf => "initializer_of",
    TypeInfo => "type_info", Null => "null", Enum => "enum", True => "true", False => "false",
    Inline => "inline", NoInline => "no_inline", Cast => "cast", AutoCast => "auto_cast",
    Context => "context", PushContext => "push_context", Operator => "operator",
    IsConstant => "is_constant", EnumFlags => "enum_flags", Union => "union", Interface => "interface"
});
tags!(Directive {
    Import => "#import", Load => "#load", Run => "#run", String => "#string", If => "#if",
    Foreign => "#foreign", CCall => "#c_call", Type => "#type", CppMethod => "#cpp_method",
    Char => "#char", As => "#as", Through => "#through", Elsewhere => "#elsewhere",
    ScopeFile => "#scope_file", NoContext => "#no_context", Place => "#place", Expand => "#expand",
    TypeInfoNone => "#type_info_none", Library => "#library", Asm => "#asm", Assert => "#assert",
    Must => "#must", Align => "#align", ScopeModule => "#scope_module", ScopeExport => "#scope_export",
    SystemLibrary => "#system_library", Insert => "#insert", CallerLocation => "#caller_location",
    Modify => "#modify", Code => "#code", Complete => "#complete", Compiler => "#compiler",
    BakeArguments => "#bake_arguments", AddContext => "#add_context", ModuleParameters => "#module_parameters",
    Location => "#location", Filepath => "#filepath", Deprecated => "#deprecated", This => "#this",
    NoReset => "#no_reset", ProgramExport => "#program_export", Intrinsic => "#intrinsic",
    RunAndInsert => "#run_and_insert", Bytes => "#bytes", CallerCode => "#caller_code",
    Specified => "#specified", Symmetric => "#symmetric", NoDebug => "#no_debug", NoPadding => "#no_padding"
});
// Longest spellings precede their prefixes for maximal-munch tokenization.
tags!(Punct {
    RotateLeftAssign => "<<<=", RotateRightAssign => ">>>=", TripleEqual => "===", Uninitialized => "---",
    LogicalAndAssign => "&&=", LogicalOrAssign => "||=", ShiftLeftAssign => "<<=", ShiftRightAssign => ">>=",
    RotateLeft => "<<<", RotateRight => ">>>", Constant => "::", Infer => ":=", Arrow => "->",
    QuickLambda => "=>", Range => "..", DoubleComma => ",,", StructLiteral => ".{", ArrayLiteral => ".[",
    PostfixDeref => ".*", ShiftLeft => "<<", ShiftRight => ">>", LessEqual => "<=", GreaterEqual => ">=",
    Equal => "==", NotEqual => "!=", LogicalAnd => "&&", LogicalOr => "||", AddAssign => "+=",
    SubAssign => "-=", MulAssign => "*=", DivAssign => "/=", RemAssign => "%=", AndAssign => "&=",
    OrAssign => "|=", XorAssign => "^=", DoubleMinus => "--", DoubleDollar => "$$",
    OpenBrace => "{", CloseBrace => "}", OpenBracket => "[", CloseBracket => "]", OpenParen => "(", CloseParen => ")",
    Semicolon => ";", Comma => ",", Colon => ":", Dot => ".", Dollar => "$", Backtick => "`",
    Add => "+", Sub => "-", Mul => "*", Div => "/", Rem => "%", Assign => "=", Not => "!",
    Less => "<", Greater => ">", Or => "|", And => "&", Xor => "^", Complement => "~", Question => "?", Hash => "#"
});

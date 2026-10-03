//! Parsing converts raw tokens into domain operators, names and signatures.
mod source_procedures;
pub use source_procedures::*;
mod anonymous_procedures;
mod anonymous_records;
mod array_literal_targets;
mod builtin_type_tags;
mod call_arguments;
mod caller_exports;
#[cfg(test)]
mod caller_references;
mod casts;
mod code_syntax;
mod compile_time;
mod compile_time_cases;
mod insert_replacements;
mod procedure_notes;
mod source_procedure_headers;
pub use compile_time_cases::*;
mod context_fields;
mod declaration_attributes;
mod deprecation;
mod file_conditional_bodies;
pub use deprecation::Deprecation;
mod expressions;
mod external_data;
pub use external_data::{ExternalDataBinding, ExternalDataSource};
#[cfg(test)]
mod field_placement;
mod field_prefix;
mod instruction_bytes;
mod libraries;
mod literals;
mod metadata;
mod modules;
mod numeric_literals;
mod operator_aliases;
mod operator_declarations;
mod operators;
mod parameter_baking;
mod parameter_evaluation;
mod placeholders;
pub use placeholders::PlaceholderDeclaration;
mod places;
mod pointer_prefixes;
pub use parameter_baking::ParameterBaking;
pub use parameter_evaluation::ParameterEvaluation;
pub use type_restrictions::TypeRestrictionSyntax;
mod procedures;
mod program_exports;
mod record_conditionals;
mod record_defaults;
mod record_members;
mod record_parameters;
#[cfg(test)]
mod record_placement;
#[cfg(test)]
mod record_reflection_syntax;
mod run_flag_parser;
mod run_flags;
mod safety_checks;
mod source_contracts;
pub use run_flags::RunFlags;
mod short_lambdas;
mod simd;
mod statement_conditionals;
mod statements;
mod type_annotations;
mod type_restrictions;
mod types;
mod using_declarations;
mod using_parser;
mod using_syntax;
pub use code_syntax::*;
pub use compile_time::*;
pub use declaration_attributes::*;
pub use expressions::*;
pub use field_prefix::*;
pub use insert_replacements::*;
pub use instruction_bytes::*;
use jai_lexer::{Directive, Keyword, Kind, Punct, Token, lex};
use jai_source::{Diagnostic, Span, Symbol, Symbols};
pub use libraries::*;
pub use literals::{DecimalLiteral, FloatRangeError, HereStringLiteral, HereStringModifier};
pub use metadata::*;
pub use modules::*;
pub use operator_aliases::*;
pub use operator_declarations::*;
pub use places::*;
pub use procedures::*;
pub use program_exports::*;
pub use record_members::*;
pub use record_parameters::*;
pub use safety_checks::{CheckPolicy, SafetyChecks};
pub use short_lambdas::*;
pub use simd::*;
pub use types::*;
pub use using_syntax::*;

pub use jai_types::{CastMode, IntegerType, ReturnType, ScalarType};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Positive,
    Negate,
    LogicalNot,
    Complement,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    LogicalOr,
    LogicalAnd,
    BitOr,
    BitXor,
    BitAnd,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    ShiftLeft,
    ShiftRight,
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}
impl BinaryOp {
    fn compound(p: Punct) -> Option<Self> {
        Some(match p {
            Punct::AddAssign => Self::Add,
            Punct::SubAssign => Self::Subtract,
            Punct::MulAssign => Self::Multiply,
            Punct::DivAssign => Self::Divide,
            Punct::RemAssign => Self::Remainder,
            Punct::AndAssign => Self::BitAnd,
            Punct::OrAssign => Self::BitOr,
            Punct::XorAssign => Self::BitXor,
            Punct::ShiftLeftAssign => Self::ShiftLeft,
            Punct::ShiftRightAssign => Self::ShiftRight,
            Punct::LogicalAndAssign => Self::LogicalAnd,
            Punct::LogicalOrAssign => Self::LogicalOr,
            _ => return None,
        })
    }
    fn parse(p: Punct) -> Option<(Self, u8)> {
        Some(match p {
            Punct::LogicalOr => (Self::LogicalOr, 1),
            Punct::LogicalAnd => (Self::LogicalAnd, 3),
            Punct::Equal => (Self::Equal, 5),
            Punct::NotEqual => (Self::NotEqual, 5),
            Punct::Less => (Self::Less, 7),
            Punct::LessEqual => (Self::LessEqual, 7),
            Punct::Greater => (Self::Greater, 7),
            Punct::GreaterEqual => (Self::GreaterEqual, 7),
            Punct::Or => (Self::BitOr, 9),
            Punct::Xor => (Self::BitXor, 11),
            Punct::And => (Self::BitAnd, 13),
            Punct::ShiftLeft => (Self::ShiftLeft, 15),
            Punct::ShiftRight => (Self::ShiftRight, 15),
            Punct::Add => (Self::Add, 17),
            Punct::Sub => (Self::Subtract, 17),
            Punct::Mul => (Self::Multiply, 19),
            Punct::Div => (Self::Divide, 19),
            Punct::Rem => (Self::Remainder, 19),
            _ => return None,
        })
    }
}
#[derive(Clone, Debug)]
pub struct Module {
    procedures: Vec<Procedure>,
    symbols: Symbols,
    constants: Vec<ConstantDeclaration>,
    globals: Vec<GlobalDeclaration>,
}
impl Module {
    pub fn procedures(&self) -> &[Procedure] {
        &self.procedures
    }
    pub fn constants(&self) -> &[ConstantDeclaration] {
        &self.constants
    }
    pub fn globals(&self) -> &[GlobalDeclaration] {
        &self.globals
    }
    pub fn symbols(&self) -> &Symbols {
        &self.symbols
    }
}
#[derive(Clone, Debug)]
pub struct GlobalDeclaration {
    pub declaration: Declaration,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub struct ConstantDeclaration {
    pub name: Symbol,
    pub ty: Option<TypeSyntax>,
    pub initializer: Expression,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub struct ConstantResultsDeclaration {
    pub names: Vec<(Symbol, Span)>,
    pub initializer: Expression,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub enum Declaration {
    External {
        name: Symbol,
        ty: TypeSyntax,
        binding: ExternalDataBinding,
        attributes: Vec<DeclarationAttribute>,
    },
    Inferred {
        name: Symbol,
        initializer: Expression,
        attributes: Vec<DeclarationAttribute>,
    },
    Explicit {
        name: Symbol,
        ty: ScalarType,
        initializer: Option<Expression>,
        attributes: Vec<DeclarationAttribute>,
    },
    UnresolvedExplicit {
        name: Symbol,
        ty: TypeSyntax,
        initializer: Option<Expression>,
        attributes: Vec<DeclarationAttribute>,
    },
}
impl Declaration {
    pub fn attributes(&self) -> &[DeclarationAttribute] {
        match self {
            Self::Inferred { attributes, .. }
            | Self::Explicit { attributes, .. }
            | Self::UnresolvedExplicit { attributes, .. }
            | Self::External { attributes, .. } => attributes,
        }
    }
    pub fn name(&self) -> Symbol {
        match self {
            Self::Inferred { name, .. }
            | Self::Explicit { name, .. }
            | Self::UnresolvedExplicit { name, .. }
            | Self::External { name, .. } => *name,
        }
    }
}
pub use jai_types::Direction;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JumpKind {
    Break,
    Continue,
    Remove,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopTarget {
    Innermost,
    Named(Symbol),
}
#[derive(Clone, Debug)]
pub enum WhileCondition {
    Expression(Expression),
    Binding {
        name: Symbol,
        export_span: Option<Span>,
        initializer: Expression,
    },
}
#[derive(Clone, Debug)]
pub struct RangeLoop {
    pub iterator: Symbol,
    pub iterator_export: bool,
    pub start: Expression,
    pub end: Expression,
    pub direction: Direction,
    pub reverse_control: Option<Expression>,
    pub body: Vec<Statement>,
}
#[derive(Clone, Debug)]
pub struct ArrayLoop {
    pub expansion: Option<NamePath>,
    pub iterator: Symbol,
    pub iterator_export: bool,
    pub index: Option<Symbol>,
    pub index_export: bool,
    pub sequence: Expression,
    pub direction: Direction,
    pub reverse_control: Option<Expression>,
    pub by_pointer: bool,
    pub pointer_control: Option<Expression>,
    pub body: Vec<Statement>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaseOperator {
    Equal,
    NotEqual,
}
#[derive(Clone, Debug)]
pub struct CaseStatement {
    pub value: Expression,
    pub operator: CaseOperator,
    pub arms: Vec<(Expression, Vec<Statement>, bool)>,
    pub complete: bool,
    pub default: Option<Vec<Statement>>,
}
/// A complete parsed statement and its byte range in the defining source.
///
/// The surrounding parsed file or captured code owns the source identity. Cloning
/// syntax retains this range even when name lookup is rebound during insertion.
#[derive(Clone, Debug)]
pub struct Statement {
    pub span: Span,
    pub kind: StatementKind,
}
impl Statement {
    /// Wrap retained syntax with its genuine source range; no location is inferred.
    pub fn new(span: Span, kind: StatementKind) -> Self {
        Self { span, kind }
    }
}

#[derive(Clone, Debug)]
pub enum StatementKind {
    InstructionBytes(InstructionBytes),
    Simd(SimdBlock),
    Import(ScopedImportDeclaration),
    Using(UsingDirective),
    UsingDeclaration {
        declaration: Box<Statement>,
        selection: UsingSelection,
        target_span: Span,
    },
    CallerExport(Box<Statement>),
    CompileTimeAssert {
        condition: Expression,
        message: Option<Expression>,
    },
    CompileTimeIf {
        condition: Expression,
        then_body: Vec<Statement>,
        else_body: Vec<Statement>,
    },
    CheckScope {
        checks: SafetyChecks,
        body: Vec<Statement>,
    },
    ContextField(ContextFieldDeclaration),
    Library(LibraryDeclaration),
    Procedure(Box<Procedure>),
    ProcedurePrototype(ProcedurePrototype),
    Record(RecordDeclaration),
    Enum(EnumDeclaration),
    TypeAlias(TypeAliasDeclaration),
    Insert(InsertDirective),
    Declare(Declaration),
    Constant(ConstantDeclaration),
    ConstantResults(ConstantResultsDeclaration),
    Assign(Symbol, Expression),
    Update(Symbol, BinaryOp, Expression),
    AssignPlace {
        target: PlaceSyntax,
        value: Expression,
    },
    UpdatePlace {
        target: PlaceSyntax,
        operation: BinaryOp,
        value: Expression,
    },
    DeclareResults {
        names: Vec<Symbol>,
        ty: Option<TypeSyntax>,
        values: Vec<Expression>,
    },
    MixedResults {
        bindings: Vec<ResultTargetBinding>,
        ty: Option<TypeSyntax>,
        values: Vec<Expression>,
    },
    AssignResults {
        targets: Vec<PlaceSyntax>,
        values: Vec<Expression>,
        operation: Option<BinaryOp>,
    },
    Return(Option<Expression>),
    ReturnValues(Vec<ReturnValue>),
    Expression(Expression),
    If(Expression, Vec<Statement>, Vec<Statement>),
    Cases(CaseStatement),
    CompileTimeCases(CompileTimeCases<Statement>),
    While(WhileCondition, Vec<Statement>),
    Range(RangeLoop),
    ArrayLoop(ArrayLoop),
    PushContext {
        value: Option<Expression>,
        body: Vec<Statement>,
    },
    Jump {
        kind: JumpKind,
        target: LoopTarget,
        span: Span,
    },
    Block(Vec<Statement>),
    Defer(Vec<Statement>),
}

#[derive(Clone, Debug)]
pub enum ResultTargetBinding {
    New { name: Symbol, span: Span },
    Existing(PlaceSyntax),
}

pub fn parse(source: &str) -> Result<Module, Diagnostic> {
    let mut parser = Parser {
        source,
        tokens: lex(source)?,
        at: 0,
        symbols: Symbols::default(),
        allow_qualified: false,
        record_conditional_depth: 0,
        file_conditional_depth: 0,
    };
    let mut procedures = Vec::new();
    let mut constants = Vec::new();
    let mut globals = Vec::new();
    while parser.token().kind != Kind::Eof {
        if parser.starts_procedure() {
            procedures.push(parser.procedure()?);
        } else {
            let span = parser.token().span;
            let name = parser.name()?;
            match parser.data_declaration(name, span)?.kind {
                StatementKind::Declare(declaration) => {
                    globals.push(GlobalDeclaration { declaration, span })
                }
                StatementKind::Constant(declaration) => constants.push(declaration),
                _ => unreachable!("data declaration always produces a declaration"),
            }
        }
    }
    Ok(Module {
        procedures,
        constants,
        globals,
        symbols: parser.symbols,
    })
}
struct Parser<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    at: usize,
    symbols: Symbols,
    allow_qualified: bool,
    record_conditional_depth: usize,
    file_conditional_depth: usize,
}
impl Parser<'_> {
    fn token(&self) -> Token {
        self.tokens[self.at]
    }
    fn text(&self) -> &str {
        self.token().span.text(self.source)
    }
    fn is(&self, p: Punct) -> bool {
        self.token().kind == Kind::Punctuation(p)
    }
    fn take(&mut self, p: Punct) -> bool {
        if self.is(p) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn keyword(&mut self, k: Keyword) -> bool {
        if self.token().kind == Kind::Keyword(k) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn error(&self, msg: impl Into<String>) -> Diagnostic {
        Diagnostic::new(self.token().span, msg)
    }
    fn need(&mut self, p: Punct) -> Result<(), Diagnostic> {
        if self.take(p) {
            Ok(())
        } else {
            Err(self.error(format!(
                "expected '{}', found '{}'",
                p.spelling(),
                self.text()
            )))
        }
    }
    fn name(&mut self) -> Result<Symbol, Diagnostic> {
        if !matches!(
            self.token().kind,
            Kind::Ident | Kind::Keyword(Keyword::Context)
        ) {
            return Err(self.error("expected identifier"));
        }
        let name = self.token().spelling(self.source);
        let symbol = self.symbols.intern(&name);
        self.at += 1;
        Ok(symbol)
    }
    fn name_path(&mut self) -> Result<NamePath, Diagnostic> {
        let root = self.name()?;
        let mut members = Vec::new();
        while self.take(Punct::Dot) {
            members.push(self.name()?);
        }
        Ok(NamePath { root, members })
    }
    fn scalar_type(&mut self) -> Result<ScalarType, Diagnostic> {
        let ty = match BuiltinType::from_spelling(&self.token().spelling(self.source)) {
            Some(BuiltinType::Scalar(ty)) => ty,
            _ => return Err(self.error("expected a supported scalar type")),
        };
        if self.token().kind != Kind::Ident {
            return Err(self.error("expected type"));
        }
        self.at += 1;
        Ok(ty)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recursive_procedure() {
        let m = parse("f :: (n: int) -> int { if n <= 1 return 1; return n*f(n-1); }").unwrap();
        assert_eq!(m.symbols.name(m.procedures[0].parameters[0].name), "n");
        assert_eq!(m.procedures[0].body.len(), 2);
    }
    #[test]
    fn precedence() {
        let m = parse("main :: () -> int { return 1 + 2 * 3; }").unwrap();
        let StatementKind::Return(Some(e)) = &m.procedures[0].body[0].kind else {
            panic!()
        };
        let ExpressionKind::Binary(BinaryOp::Add, _, rhs) = &e.kind else {
            panic!()
        };
        assert!(matches!(
            rhs.kind,
            ExpressionKind::Binary(BinaryOp::Multiply, _, _)
        ));
    }
    #[test]
    fn unsupported_is_not_success() {
        assert!(parse("#import \"Basic\";").is_err());
        assert!(parse("main :: () { x: float; }").is_err());
    }
    #[test]
    fn missing_delimiter() {
        assert!(parse("main :: () { return 1;").is_err());
    }
    #[test]
    fn loop_syntax_has_domain_tags() {
        let module =
            parse("main :: () { for < i: 1..4 { continue i; } while value := true break value; }")
                .unwrap();
        let StatementKind::Range(range) = &module.procedures[0].body[0].kind else {
            panic!("expected range")
        };
        assert_eq!(range.direction, Direction::Reverse);
        assert!(
            matches!(&range.body[0].kind, StatementKind::Jump { kind: JumpKind::Continue, target: LoopTarget::Named(name), .. } if *name == range.iterator)
        );
        assert!(matches!(
            &module.procedures[0].body[1].kind,
            StatementKind::While(WhileCondition::Binding { .. }, _)
        ));
        for invalid in [
            "main :: () { for 1.. {} }",
            "main :: () { for 1 {} }",
            "main :: () { break 1; }",
            "main :: () { for #v2 < 1..3 {} }",
        ] {
            assert!(parse(invalid).is_err(), "{invalid}");
        }
    }
    #[test]
    fn distinguish_parenthesized_constants_from_procedures() {
        let module = parse("N :: (1 + 2) * 14; flag : bool : true; count : int; main :: ()->int { LOCAL :: 2; return N; }").unwrap();
        assert_eq!(module.constants().len(), 2);
        assert_eq!(module.globals().len(), 1);
        assert_eq!(module.procedures().len(), 1);
        assert!(matches!(
            module.procedures()[0].body[0].kind,
            StatementKind::Constant(_)
        ));
        assert!(parse("N :: (1 + 2; main :: () {}").is_err());
    }
    #[test]
    fn reserved_words_are_not_names() {
        assert!(parse("if :: () {}").is_err());
    }
    #[test]
    fn conditional_expressions_bind_else_to_the_nearest_ifx() {
        let module =
            parse("main :: ()->int { return ifx true then ifx false 1 else 2 else 3; }").unwrap();
        let StatementKind::Return(Some(e)) = &module.procedures()[0].body[0].kind else {
            panic!()
        };
        let ExpressionKind::Conditional(outer) = &e.kind else {
            panic!()
        };
        assert!(matches!(
            outer.else_value.as_ref().unwrap().kind,
            ExpressionKind::Integer(3)
        ));
        let ExpressionKind::Conditional(inner) = &outer.then_value.kind else {
            panic!()
        };
        assert!(matches!(
            inner.else_value.as_ref().unwrap().kind,
            ExpressionKind::Integer(2)
        ));
        assert!(parse("main :: ()->int { return ifx true then 1; }").is_ok());
        for source in [
            "main :: ()->int { return ifx true then else 1; }",
            "main :: ()->int { return ifx true else 1; }",
            "main :: ()->int { return ifx true then 1 else; }",
        ] {
            assert!(parse(source).is_err(), "{source}");
        }
    }
}

#[cfg(test)]
mod case_parser_tests {
    #[test]
    fn rejects_invalid_case_structure_and_expression_cases() {
        for source in [
            "main :: () { if 1 == { case; case 1; } }",
            "main :: () { if 1 == { case 1; #through; n := 1; case; } }",
            "main :: () { if 1 == { case; #through; } }",
            "main :: () { if 1 == { n := 1; } }",
            "main :: () { if 1 == { case 1;",
            "main :: () { if #complete true { } }",
            "main :: ()->int { return ifx 1 == { case 1; 2; case; 3; }; }",
        ] {
            assert!(super::parse(source).is_err(), "{source}");
        }
    }
}

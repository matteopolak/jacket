//! Syntax tree. Types are ordinary expressions (as in Jai), so `*T`, `[]T`,
//! `struct {..}` and procedure headers all live in `ExprKind`. Semantic
//! analysis decides whether an expression denotes a type or a value.
//!
//! The tree is immutable after parsing. Declarations that sema needs to
//! identify (procedures, structs, enums, declarations) carry an `AstId`,
//! unique across the whole compilation, so per-instance results can be kept in
//! side tables (polymorphic procedures are checked once per instantiation).
use crate::intern::Sym;
use crate::source::Span;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AstId(pub u32);

thread_local! {
    static NEXT_AST_ID: Cell<u32> = const { Cell::new(0) };
}

impl AstId {
    pub fn fresh() -> AstId {
        NEXT_AST_ID.with(|n| {
            let id = n.get();
            n.set(id + 1);
            AstId(id)
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ident {
    pub name: Sym,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Note {
    pub text: Rc<str>,
    pub span: Span,
}

/// The `@note` annotations of a statement or declaration. Nearly always empty, so it is a single
/// pointer (null when empty) rather than a `Vec`'s three words; it reads as a `[Note]` slice.
#[derive(Clone, Debug, Default, PartialEq)]
// A `Box<[Note]>` would be two words; the `Box<Vec<_>>` is the thin pointer this type exists for.
#[allow(clippy::box_collection)]
pub struct Notes(Option<Box<Vec<Note>>>);

impl Notes {
    pub fn new(notes: Vec<Note>) -> Notes {
        Notes::from(notes)
    }

    pub fn extend(&mut self, notes: impl IntoIterator<Item = Note>) {
        let mut all = self.0.take().map_or_else(Vec::new, |notes| *notes);
        all.extend(notes);
        *self = Notes::from(all);
    }
}

impl From<Vec<Note>> for Notes {
    fn from(notes: Vec<Note>) -> Notes {
        Notes(if notes.is_empty() {
            None
        } else {
            Some(Box::new(notes))
        })
    }
}

impl std::ops::Deref for Notes {
    type Target = [Note];

    fn deref(&self) -> &[Note] {
        match &self.0 {
            Some(notes) => notes,
            None => &[],
        }
    }
}

impl<'a> IntoIterator for &'a Notes {
    type Item = &'a Note;
    type IntoIter = std::slice::Iter<'a, Note>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An integer literal's magnitude. Stored as two `u64` words so that `ExprKind` needs only 8-byte
/// alignment (a `u128` field would align every `Expr` to 16 bytes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Int128([u64; 2]);

impl Int128 {
    pub fn new(value: u128) -> Int128 {
        Int128([value as u64, (value >> 64) as u64])
    }

    pub fn get(self) -> u128 {
        self.0[0] as u128 | (self.0[1] as u128) << 64
    }
}

// ---------------------------------------------------------------------------
// Expressions
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Rotl,
    Rotr,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnOp {
    Neg,
    /// Unary plus (accepted and ignored).
    Plus,
    Not,
    BitNot,
    /// `*x`: address-of in value position, pointer type in type position.
    Star,
    /// `<< x` or `x.*`: dereference.
    Deref,
}

#[derive(Clone, Debug)]
pub struct Arg {
    /// `name = value` named argument / struct literal field.
    pub name: Option<Ident>,
    /// Non-identifier field target in a literal: `.{values[1] = 7, base.init = f}` (then `name` is `None`).
    pub target: Option<Box<Expr>>,
    /// Written after `,,` in a call: temporary context change (`f(x,, allocator = temp)`).
    pub context: bool,
    /// `..value` spreads an array into variadic arguments.
    pub spread: bool,
    pub value: Expr,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CastFlags {
    pub no_check: bool,
    pub truncate: bool,
    pub force: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallHint {
    None,
    Inline,
    NoInline,
}

#[derive(Clone, Debug)]
pub enum ArraySize {
    /// `[N] T` (N may be a `$N` polymorphic variable).
    Fixed(Box<Expr>),
    /// `[] T`
    View,
    /// `[..] T`
    Resizable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeModifier {
    /// `#type T`
    Plain,
    /// `#type,distinct T`
    Distinct,
    /// `#type,isa T`
    Isa,
}

#[derive(Clone, Debug)]
pub enum ExprKind {
    Ident(Sym),
    /// `$T` introduces a polymorphic variable; `$$x` is a baked argument
    /// (`baked` = true when `$$`).
    PolyVar {
        name: Sym,
        baked: bool,
    },
    /// `$T/Interface` or `$T/interface Interface` restriction.
    PolyRestricted {
        name: Sym,
        restriction: Box<Expr>,
        interface: bool,
    },
    Int(Int128),
    Float(f64),
    Str(Rc<[u8]>),
    Bool(bool),
    Null,
    /// `---`
    Uninit,
    Context,
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Unary(UnOp, Box<Expr>),
    Call {
        callee: Box<Expr>,
        args: Vec<Arg>,
        hint: CallHint,
    },
    /// `a.b`
    Member(Box<Expr>, Ident),
    /// `.FOO`: enum member whose type comes from context.
    InferredMember(Ident),
    Index(Box<Expr>, Box<Expr>),
    /// `cast(T) x`, `cast,no_check(T) x`. `ty == None` is `xx x`.
    Cast {
        ty: Option<Box<Expr>>,
        value: Box<Expr>,
        flags: CastFlags,
    },
    /// `ifx c then a else b`; `then_value == None` means the condition value is reused.
    /// `#ifx` sets `is_static` (the condition is a compile-time constant).
    /// `else_value == None` is `ifx c then a` (the type's zero value when false).
    Ifx {
        cond: Box<Expr>,
        then_value: Option<Box<Expr>>,
        else_value: Option<Box<Expr>>,
        is_static: bool,
    },
    /// `T.{...}` / `.{...}`
    StructLit {
        ty: Option<Box<Expr>>,
        fields: Vec<Arg>,
    },
    /// `T.[...]` / `.[...]`
    ArrayLit {
        ty: Option<Box<Expr>>,
        elems: Vec<Expr>,
    },
    ArrayType {
        size: ArraySize,
        elem: Box<Expr>,
    },
    /// A procedure header without a body (a procedure type).
    ProcType(Rc<ProcHeader>),
    /// A procedure literal (header + body, or a foreign/compiler declaration).
    Proc(Rc<ProcLit>),
    /// Short lambda: `x => x + 1`, `(a, b) => { ... }`. Parameter types are usually absent.
    Lambda {
        header: Rc<ProcHeader>,
        body: Box<Expr>,
    },
    /// A block in value position: `#ifx c { ...; value } else { ... }`, lambda bodies.
    Block(Box<Block>),
    Struct(Rc<StructLit>),
    Enum(Rc<EnumLit>),
    TypeDirective {
        modifier: TypeModifier,
        ty: Box<Expr>,
    },
    /// `#run expr` or `#run { ... }`.
    Run {
        body: Rc<RunBody>,
        flags: Vec<Ident>,
    },
    /// `#code expr`, `#code { ... }`.
    Code(Rc<CodeBody>),
    /// `#insert expr` in expression position. `scope` is the `,scope(x)` operand;
    /// `replacements` are the `(remove = ..., break = ...)` arguments before the code.
    Insert(Box<InsertExpr>),
    /// `#char "x"`
    Char(u32),
    /// `#location(expr)` (`None` = location of the directive itself).
    Location(Option<Box<Expr>>),
    CallerLocation,
    CallerCode,
    File,
    Line,
    Filepath,
    ProcedureName(Option<Box<Expr>>),
    This,
    CompileTime,
    /// `#bake_arguments f(a = 1)` / `#bake_constants`.
    Bake {
        callee: Box<Expr>,
        args: Vec<Arg>,
        constants: bool,
    },
    /// `#procedure_of_call f(x)`
    ProcedureOfCall(Box<Expr>),
    /// `#exists(name)`
    Exists(Box<Expr>),
    /// `#bytes "..."` or `#bytes .[...]`
    Bytes(Box<Expr>),
    /// `#placeholder`-style or unrecognized directive with an optional operand.
    UnknownDirective(Box<UnknownDirective>),
    /// `` `x ``: refer to the macro caller's scope.
    Backtick(Box<Expr>),
    /// `#asm { ... }`: x64 instructions over Jai variables (lowered by `sema::asm`).
    Asm(Rc<AsmBlock>),
}

/// The payload of `ExprKind::Insert`, boxed because it is rare and large.
#[derive(Clone, Debug)]
pub struct InsertExpr {
    pub value: Expr,
    pub flags: Vec<Ident>,
    pub scope: Option<Expr>,
    pub replacements: Vec<Arg>,
}

/// The payload of `ExprKind::UnknownDirective`, boxed because it is rare and large.
#[derive(Clone, Debug)]
pub struct UnknownDirective {
    pub name: Ident,
    /// `#library,system,link_always "x"`: the comma flags.
    pub flags: Vec<Ident>,
    pub operand: Option<Expr>,
}

// ---------------------------------------------------------------------------
// Inline assembly
// ---------------------------------------------------------------------------

/// `#asm FEATURE, FEATURE { item; item; ... }`
#[derive(Clone, Debug)]
pub struct AsmBlock {
    /// Feature-set modifiers written between `#asm` and `{` (`AVX, AVX2`).
    pub features: Vec<Ident>,
    pub items: Vec<AsmItem>,
}

#[derive(Clone, Debug)]
pub enum AsmItem {
    Inst(AsmInst),
    /// `x: gpr;`, `t: gpr === a;` or `x === a;`
    Decl(AsmDecl),
}

#[derive(Clone, Debug)]
pub struct AsmInst {
    /// Mnemonic as written, including a `lock_` / `rep_` prefix.
    pub mnemonic: Ident,
    /// `.q`, `.64` or `?T` operand size.
    pub size: Option<AsmSize>,
    pub operands: Vec<AsmOperand>,
    /// EVEX decorations: `{k}` masking (`v: &* mask`) and SAE / rounding (`v !z`).
    pub evex: AsmEvex,
    pub span: Span,
}

#[derive(Clone, Debug, Default)]
pub struct AsmEvex {
    /// `&` (merging) or `&*` (zeroing, `true`) with the mask register operand.
    pub mask: Option<(Box<AsmOperand>, bool)>,
    /// `!` / `!n` / `!d` / `!u` / `!z`: the mode letter, or `None` for plain SAE.
    pub round: Option<Option<char>>,
}

#[derive(Clone, Debug)]
pub enum AsmSize {
    /// `.b` `.w` `.d` `.q` `.8` `.16` `.32` `.64` `.x` `.y` `.z`
    Suffix(Ident),
    /// `?T` or `?BITS`: a type, or a constant number of bits.
    Dynamic(Box<Expr>),
}

#[derive(Clone, Debug)]
pub enum AsmOperand {
    /// A variable or constant expression (`count`, `17`, `-127`, `*x`).
    Value(Expr),
    /// `[base + index*scale + disp]`
    Mem(AsmMem),
    /// `name:` / `name: gpr` / `name: gpr === a` declared at its first use.
    Decl(AsmDecl),
}

#[derive(Clone, Debug)]
pub struct AsmDecl {
    pub name: Ident,
    /// False for a pin of an existing variable (`x === a`).
    pub colon: bool,
    /// Register class (`gpr`, `vec`, ...); `gpr` when omitted.
    pub class: Option<Ident>,
    /// `=== a` / `=== 15`: register pinning (ignored: registers are not modeled).
    pub pin: Option<AsmPin>,
}

#[derive(Clone, Copy, Debug)]
pub enum AsmPin {
    Name(Ident),
    Index(u128),
}

#[derive(Clone, Debug)]
pub struct AsmMem {
    pub terms: Vec<AsmMemTerm>,
    /// `[mem]!`: EVEX broadcast of a single element.
    pub broadcast: bool,
    pub span: Span,
}

/// One `+ value` / `- value` / `+ value*scale` term of a memory operand.
#[derive(Clone, Debug)]
pub struct AsmMemTerm {
    pub negate: bool,
    pub value: Expr,
    pub scale: Option<Expr>,
}

#[derive(Clone, Debug)]
pub enum RunBody {
    Expr(Expr),
    Block(Block),
}

#[derive(Clone, Debug)]
pub enum CodeBody {
    Expr(Expr),
    Block(Block),
}

// ---------------------------------------------------------------------------
// Procedures
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Param {
    pub name: Option<Ident>,
    /// `$name: T` (the parameter's value must be a compile-time constant).
    pub baked: bool,
    /// `$$name: T`: baked when the call's argument is a constant, else a runtime parameter.
    pub auto_bake: bool,
    pub using: bool,
    /// `using,except(x) q: T`
    pub using_filter: Option<UsingFilter>,
    /// `#discard` parameter flag (the argument is not evaluated when unused).
    pub discard: bool,
    /// `..` variadic: `args: ..Any` (or `..$T`).
    pub variadic: bool,
    pub ty: Option<Expr>,
    pub default: Option<Expr>,
    pub notes: Vec<Note>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Return {
    pub name: Option<Ident>,
    /// `None` for `-> x := 5` (the type comes from the default value).
    pub ty: Option<Expr>,
    pub default: Option<Expr>,
    /// `#must`
    pub must: bool,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum ForeignName {
    /// `#foreign lib` — symbol is the declaration's name.
    Default,
    Named(Rc<str>),
}

/// `#foreign lib "symbol"` / `#elsewhere lib "symbol"`; `library == None` for a bare `#foreign;`.
#[derive(Clone, Debug)]
pub struct Foreign {
    pub library: Option<Ident>,
    pub name: ForeignName,
}

/// How a procedure literal was written when it is a desugared lambda.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LambdaKind {
    /// An ordinary procedure.
    #[default]
    None,
    /// `(x) => expr`: the body is `return expr;` and the result type is inferred.
    Expr,
    /// `(x) => { ... }`: a block body; results come from the expected procedure type.
    Block,
}

#[derive(Clone, Debug, Default)]
pub struct ProcFlags {
    /// Set on the header of a lambda's desugared procedure literal.
    pub lambda: LambdaKind,
    pub c_call: bool,
    pub no_context: bool,
    pub expand: bool,
    pub compiler: bool,
    pub intrinsic: bool,
    pub elsewhere: Option<Option<Ident>>,
    pub symmetric: bool,
    pub inline: CallHintFlag,
    pub cpp_method: bool,
    pub cpp_return_type_is_non_pod: bool,
    pub runtime_support: bool,
    pub no_debug: bool,
    pub no_abc: bool,
    pub no_aoc: bool,
    pub deprecated: Option<Option<Rc<[u8]>>>,
    pub program_export: Option<Option<Rc<[u8]>>>,
    pub no_call: bool,
    /// The string after `#compiler "name"` / `#intrinsic "name"` selecting the builtin.
    pub builtin_name: Option<Rc<str>>,
    /// Any other `#directive` flag (`#dump`, `#entry_point`, `#no_alias`, ...), kept by name.
    pub other: Vec<Ident>,
    pub type_info_none: bool,
    pub library_proc: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CallHintFlag {
    #[default]
    None,
    Inline,
    NoInline,
}

#[derive(Clone, Debug)]
pub struct ProcHeader {
    pub id: AstId,
    pub params: Vec<Param>,
    pub returns: Vec<Return>,
    pub flags: ProcFlags,
    /// `#foreign library_name ["symbol"]`
    pub foreign: Option<Foreign>,
    /// `#modify { ... }` block that may rewrite polymorphic bindings.
    pub modify: Option<Block>,
    pub notes: Vec<Note>,
    pub span: Span,
    /// `operator +` style declarations: the operator text.
    pub operator: Option<Rc<str>>,
    /// `#bake_arguments` removed parameters, so `span`'s text lists more than `params`.
    pub baked: bool,
}

#[derive(Clone, Debug)]
pub struct ProcLit {
    pub header: Rc<ProcHeader>,
    /// `None` for foreign, `#compiler`, `#intrinsic` and `#elsewhere` declarations.
    pub body: Option<Block>,
}

// ---------------------------------------------------------------------------
// Aggregates
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructKind {
    Struct,
    Union,
}

#[derive(Clone, Debug, Default)]
pub struct StructFlags {
    pub type_info_none: bool,
    pub type_info_procedures_are_void_pointers: bool,
    pub type_info_no_size_complaint: bool,
    pub no_padding: bool,
    pub align: Option<Expr>,
}

#[derive(Clone, Debug)]
pub struct UnionTag {
    pub name: Ident,
    pub ty: Option<Box<Expr>>,
    pub value: Option<Box<Expr>>,
}

#[derive(Clone, Debug)]
pub struct StructLit {
    pub id: AstId,
    pub kind: StructKind,
    /// Tagged union: `union tag : TagType { .A ,, a: int; }`. The tag may have a default
    /// (`union tag: TagType = .B {`, `union tag := TagType.B {`); without a written type the
    /// type is the default's.
    pub tag: Option<UnionTag>,
    /// `struct (T: Type, N := 4)`
    pub params: Vec<Param>,
    /// Body statements: declarations, `using`, `#if`, `#place`, `#as`, nested anonymous struct/union.
    pub body: Vec<Stmt>,
    pub flags: StructFlags,
    /// `#modify { ... }`
    pub modify: Option<Block>,
    pub notes: Vec<Note>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct EnumMember {
    pub name: Ident,
    pub value: Option<Expr>,
    pub notes: Vec<Note>,
}

#[derive(Clone, Debug)]
pub enum EnumItem {
    Member(EnumMember),
    /// `#insert` inside an enum body generating members.
    Insert(Expr),
    /// `#if` inside an enum body.
    If {
        cond: Expr,
        then_items: Vec<EnumItem>,
        else_items: Vec<EnumItem>,
    },
}

#[derive(Clone, Debug)]
pub struct EnumLit {
    pub id: AstId,
    pub flags_enum: bool,
    pub base: Option<Expr>,
    pub items: Vec<EnumItem>,
    pub specified: bool,
    pub complete: bool,
    pub notes: Vec<Note>,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Statements and declarations
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
    /// `#no_abc { ... }`, or the body of a loop / `if` flagged `#no_abc`: no array bounds checks.
    pub no_abc: bool,
    /// `#no_aoc { ... }`, or the body of a loop / `if` flagged `#no_aoc`: no arithmetic overflow checks.
    pub no_aoc: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeclKind {
    /// `x: T = v` / `x := v` / `x: T`
    Var,
    /// `x :: v` / `x: T : v`
    Const,
}

#[derive(Clone, Debug)]
pub struct Decl {
    pub id: AstId,
    pub names: Vec<Ident>,
    pub kind: DeclKind,
    pub ty: Option<Expr>,
    /// `None` = default-initialize. `Some(Uninit)` = `---`.
    pub value: Option<Expr>,
    pub using: bool,
    /// `#as using` / `#as x: T` in struct bodies.
    pub as_: bool,
    /// `` `x := ... `` declares into the macro caller's scope.
    pub backtick: bool,
    pub notes: Notes,
    pub span: Span,
    /// The fields almost no declaration uses; see `DeclRare` and the accessors below.
    pub rare: Option<Box<DeclRare>>,
}

/// The rarely used parts of a `Decl`, boxed so an ordinary declaration does not carry them.
#[derive(Clone, Debug, Default)]
pub struct DeclRare {
    /// Further values of `a, b := 1, 2;` (the first is in `value`).
    pub extra_values: Vec<Expr>,
    /// Mixed declare/assign lists, parallel to `names` (empty otherwise): `true` assigns to an
    /// existing variable, as in `a=, b := f();` or `a:, b = f();`.
    pub existing: Vec<bool>,
    /// Mixed lists: for an existing entry that is not a plain name (`ok:, t.str = f();`), the
    /// place assigned to; parallel to `names` (empty otherwise). Its `names` entry is the
    /// place's root variable, spanning the whole place.
    pub targets: Vec<Option<Expr>>,
    /// `using,except(x) name: T;`
    pub using_filter: Option<UsingFilter>,
    /// Per name of a list, `` a, `b := ... ``: which ones go to the caller's scope (empty = all
    /// follow `backtick`).
    pub backtick_names: Vec<bool>,
    /// `#align N`
    pub align: Option<Expr>,
    /// `#elsewhere` / `#no_reset` / other trailing flags, kept by name.
    pub flags: Vec<Ident>,
    /// `x: T #elsewhere lib ["symbol"]`: the library (and symbol) providing the variable.
    pub foreign: Option<Foreign>,
    /// `.TAG ,, member: T;` in a tagged union body: the tag value selecting this member
    /// (an enum member, an integer such as `-12`, or a type such as `u16`).
    pub union_tag: Option<Expr>,
}

impl DeclRare {
    fn is_empty(&self) -> bool {
        self.extra_values.is_empty()
            && self.existing.is_empty()
            && self.targets.is_empty()
            && self.using_filter.is_none()
            && self.backtick_names.is_empty()
            && self.align.is_none()
            && self.flags.is_empty()
            && self.foreign.is_none()
            && self.union_tag.is_none()
    }
}

impl Decl {
    /// Sets the rare fields; `None` is stored when they are all empty.
    pub fn set_rare(&mut self, rare: DeclRare) {
        self.rare = if rare.is_empty() {
            None
        } else {
            Some(Box::new(rare))
        };
    }

    /// Drops the rare box when every field in it is empty.
    pub fn trim_rare(&mut self) {
        if self.rare.as_ref().is_some_and(|r| r.is_empty()) {
            self.rare = None;
        }
    }

    /// The rare fields for in-place edits (created empty on first use).
    pub fn rare_mut(&mut self) -> &mut DeclRare {
        self.rare.get_or_insert_with(Default::default)
    }

    pub fn extra_values(&self) -> &[Expr] {
        self.rare.as_ref().map_or(&[], |r| &r.extra_values)
    }

    pub fn existing(&self) -> &[bool] {
        self.rare.as_ref().map_or(&[], |r| &r.existing)
    }

    pub fn targets(&self) -> &[Option<Expr>] {
        self.rare.as_ref().map_or(&[], |r| &r.targets)
    }

    pub fn using_filter(&self) -> Option<&UsingFilter> {
        self.rare.as_ref().and_then(|r| r.using_filter.as_ref())
    }

    pub fn backtick_names(&self) -> &[bool] {
        self.rare.as_ref().map_or(&[], |r| &r.backtick_names)
    }

    pub fn align(&self) -> Option<&Expr> {
        self.rare.as_ref().and_then(|r| r.align.as_ref())
    }

    pub fn flags(&self) -> &[Ident] {
        self.rare.as_ref().map_or(&[], |r| &r.flags)
    }

    pub fn foreign(&self) -> Option<&Foreign> {
        self.rare.as_ref().and_then(|r| r.foreign.as_ref())
    }

    pub fn union_tag(&self) -> Option<&Expr> {
        self.rare.as_ref().and_then(|r| r.union_tag.as_ref())
    }
}

#[derive(Clone, Debug)]
pub struct Case {
    /// Empty = default case (`case;`).
    pub values: Vec<Expr>,
    pub body: Vec<Stmt>,
    /// Body ends in `#through;`
    pub through: bool,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignOp {
    Assign,
    Op(BinOp),
}

#[derive(Clone, Debug)]
pub struct For {
    /// `for x, i: range` names (iterator and index).
    pub it: Option<Ident>,
    pub index: Option<Ident>,
    /// `for *x: ...` iterates by pointer.
    pub by_pointer: bool,
    /// `for < ...` iterates in reverse.
    pub reverse: bool,
    /// ``for *=cond ...``: iterate by pointer when `cond` holds.
    pub pointer_if: Option<Expr>,
    /// ``for <=cond ...``: iterate in reverse when `cond` holds.
    pub reverse_if: Option<Expr>,
    /// `for:name ...` custom iterator modifier.
    pub iterator: Option<Ident>,
    /// ``for `it, `it_index: ...``: the names are declared in the macro caller's scope.
    pub backtick_names: bool,
    /// `for a..b` (range) or `for collection`.
    pub over: ForOver,
    pub body: Box<Stmt>,
    pub flags: Vec<Ident>,
}

#[derive(Clone, Debug)]
pub enum ForOver {
    Range(Expr, Expr),
    Collection(Expr),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeKind {
    File,
    Export,
    Module,
}

#[derive(Clone, Debug)]
pub enum ImportSource {
    /// `#import "Basic"`
    Module(Rc<str>),
    /// `#import,file "x.jai"`
    File(Rc<str>),
    /// `#import,dir "path"`
    Dir(Rc<str>),
    /// `#import,string "source"`
    String(Rc<str>),
}

#[derive(Clone, Debug)]
pub struct Import {
    pub source: ImportSource,
    /// `#import "X"(PARAM=1)`
    pub params: Vec<Arg>,
    /// `Name :: #import "X"` binds a namespace instead of importing names.
    pub name: Option<Ident>,
    /// `using,only(...)` / `#import,except` style flags, kept raw.
    pub flags: Vec<Ident>,
    /// `using,except(x) Name :: #import "X";`
    pub using: Option<UsingFilter>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum UsingFilter {
    None,
    Only(Vec<Ident>),
    Except(Vec<Ident>),
    Map(Vec<(Ident, Ident)>),
    /// A filter given as an expression: `using,only(.["+", "-"]) Basic;`, `using,except NAMES x: T;`.
    Computed {
        kind: UsingFilterKind,
        expr: Expr,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UsingFilterKind {
    Only,
    Except,
    Map,
}

#[derive(Clone, Debug)]
pub enum StmtKind {
    Decl(Rc<Decl>),
    Expr(Expr),
    Assign {
        op: AssignOp,
        lhs: Vec<Expr>,
        rhs: Vec<Expr>,
    },
    Block(Block),
    If {
        cond: Box<Expr>,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    /// `if x == { case ...; }`
    Switch(Box<SwitchStmt>),
    /// `#if x == { case ...; }`: only the matching case's statements are compiled.
    StaticSwitch(Box<SwitchStmt>),
    While(Box<WhileStmt>),
    For(Box<For>),
    Break(Option<Ident>),
    Continue(Option<Ident>),
    /// `remove;` / `remove it;` in for loops.
    Remove(Option<Ident>),
    Return {
        values: Vec<Arg>,
        backtick: bool,
    },
    Defer {
        body: Box<Stmt>,
        backtick: bool,
    },
    /// `using expr;` (filters: `using,only(a,b) x;`)
    Using(Box<UsingStmt>),
    PushContext(Box<PushContextStmt>),
    /// `#if cond { } else { }`, also at file and struct scope.
    StaticIf(Box<StaticIfStmt>),
    /// `#insert expr;` (flags e.g. `,scope(x)`).
    Insert(Box<InsertStmt>),
    /// `#assert cond "message";`, or `#assert(cond, "format %", args)`.
    Assert(Box<AssertStmt>),
    /// Top-level `#run expr;`
    Run(Box<Expr>),
    Import(Rc<Import>),
    /// `#load "file.jai";`
    Load {
        path: Rc<str>,
        span: Span,
    },
    Scope(ScopeKind),
    /// `#add_context name: T = v;`
    AddContext(Rc<Decl>),
    /// `#module_parameters (A := 1) (B := 2);` second list = parameters visible to importers' runtime.
    ModuleParameters(Box<ModuleParametersStmt>),
    /// `#placeholder name;`
    Placeholder(Vec<Ident>),
    /// `#place field;` inside struct bodies.
    Place(Box<Expr>),
    /// `#overlay(field)` prefix on the following struct member declaration.
    Overlay(Box<Expr>),
    /// `#through;` (normally folded into `Case::through`).
    Through,
    /// `#program_export`-style top-level directives we keep for completeness.
    Directive(Box<DirectiveStmt>),
    /// `push_context,defer_pop ctx;`: push now, pop at the end of the scope.
    PushContextDefer(Box<PushContextDeferStmt>),
    /// Empty statement (`;`).
    Empty,
    /// A lone `case x; ...` as the body of `#code case ...` (yield-jai builds switches
    /// from them); not valid as a statement.
    Case(Rc<Case>),
}

// The payloads below are boxed in `StmtKind` because they are rare and large: every `Stmt` pays
// for its biggest inline variant, and the tree of every compile stays alive in the language server.

/// A `switch` (`if x == { ... }`) or a `#if x == { ... }`; `complete` is always false for the latter.
#[derive(Clone, Debug)]
pub struct SwitchStmt {
    pub value: Expr,
    pub cases: Vec<Case>,
    pub complete: bool,
}

#[derive(Clone, Debug)]
pub struct WhileStmt {
    pub label: Option<Ident>,
    /// `while name := cond`: `name` also holds the condition's value inside the loop.
    pub bind_label: bool,
    /// A backtick condition binding is also visible in an expansion's caller scope.
    pub backtick_label: bool,
    pub cond: Expr,
    pub body: Box<Stmt>,
}

#[derive(Clone, Debug)]
pub struct UsingStmt {
    pub value: Expr,
    pub filter: UsingFilter,
}

#[derive(Clone, Debug)]
pub struct PushContextStmt {
    pub backtick: bool,
    pub context: Expr,
    pub body: Box<Stmt>,
    /// No context was named (`push_context { }`): `context` is the current one, and a copy
    /// of it is pushed (a default one where there is none).
    pub bare: bool,
}

#[derive(Clone, Debug)]
pub struct PushContextDeferStmt {
    pub context: Expr,
    /// `push_context,defer_pop;`: as `PushContextStmt::bare`.
    pub bare: bool,
    /// A macro backtick gives the restoration lifetime to the caller's scope.
    pub backtick: bool,
}

#[derive(Clone, Debug)]
pub struct StaticIfStmt {
    pub cond: Expr,
    pub then_branch: Vec<Stmt>,
    pub else_branch: Vec<Stmt>,
}

#[derive(Clone, Debug)]
pub struct InsertStmt {
    pub value: Expr,
    pub flags: Vec<Ident>,
    pub scope: Option<Expr>,
    pub replacements: Vec<Arg>,
}

#[derive(Clone, Debug)]
pub struct AssertStmt {
    pub cond: Expr,
    pub message: Option<Expr>,
    pub args: Vec<Expr>,
}

#[derive(Clone, Debug)]
pub struct ModuleParametersStmt {
    pub params: Vec<Param>,
    pub runtime_params: Vec<Param>,
    pub body: Option<Block>,
}

#[derive(Clone, Debug)]
pub struct DirectiveStmt {
    pub name: Ident,
    pub flags: Vec<Ident>,
    pub args: Vec<Expr>,
}

#[derive(Clone, Debug)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
    pub notes: Notes,
}

/// A parsed file.
#[derive(Clone, Debug)]
pub struct File {
    pub file: crate::source::FileId,
    pub stmts: Vec<Stmt>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    /// The language server keeps the syntax tree of every file of a compile alive, so the size of
    /// these nodes is a memory budget (a 132k-line project has about a million of them). Rare or
    /// large payloads belong behind a `Box` (see `InsertStmt`, `DeclRare`), not inline. If a change
    /// pushes a node over its bound, box the new field instead of raising the bound.
    #[test]
    fn node_sizes_stay_small() {
        assert!(
            size_of::<Expr>() <= 56,
            "Expr is {} bytes",
            size_of::<Expr>()
        );
        assert!(
            size_of::<Stmt>() <= 88,
            "Stmt is {} bytes",
            size_of::<Stmt>()
        );
        assert!(
            size_of::<Decl>() <= 176,
            "Decl is {} bytes",
            size_of::<Decl>()
        );
        assert!(size_of::<Arg>() <= 88, "Arg is {} bytes", size_of::<Arg>());
    }

    #[test]
    fn int128_round_trips() {
        for v in [0, 1, u64::MAX as u128, 1 << 64, u128::MAX] {
            assert_eq!(Int128::new(v).get(), v);
        }
    }
}

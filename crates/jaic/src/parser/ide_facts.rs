//! Syntax facts for editors, read from a parsed file and its tokens: what a file declares at
//! its top level, its named procedures with their notes and `$` parameters, and the call an
//! editor's cursor is in. None of these need a compile, so they hold while the program does not
//! type-check; with `parse_file_ide`'s recovery they hold for most text that does not parse.
use crate::ast::{
    AssertStmt, BinOp, Decl, DeclKind, Expr, ExprKind, File, Ident, Import, Note, ProcHeader,
    ScopeKind, Stmt, StmtKind, UnOp,
};
use crate::intern::Sym;
use crate::lexer::{P, Tok, Token};
use crate::source::Span;

/// What a declaration's value makes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeclClass {
    /// `x := v`, `x: T`.
    Variable,
    /// A procedure, procedure type, lambda, or a baked or chosen procedure
    /// (`#bake_arguments`, `#bake_constants`, `#procedure_of_call`).
    Procedure,
    /// `struct` or `union`.
    Struct,
    /// `enum` or `enum_flags`.
    Enum,
    /// `#type T`, an array or a pointer type.
    TypeAlias,
    /// `#library`, `#system_library` and their `#foreign_` forms.
    Library,
    /// Any other constant.
    Constant,
}

/// The library directives a constant can name.
const LIBRARY_DIRECTIVES: &[&str] = &[
    "library",
    "system_library",
    "foreign_library",
    "foreign_system_library",
];

pub fn decl_class(decl: &Decl) -> DeclClass {
    if decl.kind == DeclKind::Var {
        return DeclClass::Variable;
    }
    let Some(value) = &decl.value else {
        return DeclClass::Constant;
    };
    match &value.kind {
        ExprKind::Proc(_)
        | ExprKind::ProcType(_)
        | ExprKind::Lambda {
            ..
        }
        | ExprKind::Bake {
            ..
        }
        | ExprKind::ProcedureOfCall(_) => DeclClass::Procedure,
        ExprKind::Struct(_) => DeclClass::Struct,
        ExprKind::Enum(_) => DeclClass::Enum,
        ExprKind::TypeDirective {
            ..
        }
        | ExprKind::ArrayType {
            ..
        }
        | ExprKind::Unary(UnOp::Star, _) => DeclClass::TypeAlias,
        ExprKind::UnknownDirective(directive)
            if LIBRARY_DIRECTIVES.contains(&directive.name.name.as_str()) =>
        {
            DeclClass::Library
        }
        _ => DeclClass::Constant,
    }
}

/// The header of a declaration's procedure value, when it has one.
pub fn decl_header(decl: &Decl) -> Option<&ProcHeader> {
    match &decl.value.as_ref()?.kind {
        ExprKind::Proc(lit) => Some(&lit.header),
        ExprKind::ProcType(header)
        | ExprKind::Lambda {
            header, ..
        } => Some(header),
        _ => None,
    }
}

/// `a.b.c` as its names, for a constant that only renames another (`Alias :: Module.Name;`).
pub fn alias_path(decl: &Decl) -> Option<Vec<Sym>> {
    fn walk(expr: &Expr, out: &mut Vec<Sym>) -> Option<()> {
        match &expr.kind {
            ExprKind::Ident(name) => out.push(*name),
            ExprKind::Member(base, name) => {
                walk(base, out)?;
                out.push(name.name);
            }
            _ => return None,
        }
        Some(())
    }
    if decl.kind != DeclKind::Const {
        return None;
    }
    let mut path = Vec::new();
    walk(decl.value.as_ref()?, &mut path)?;
    Some(path)
}

/// Who can see a top-level statement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visibility {
    /// Importers of the module (and the rest of the program).
    Export,
    /// The files of its module or program only (`#scope_module`).
    Module,
    /// Only the declaring file (`#scope_file`).
    File,
}

impl From<ScopeKind> for Visibility {
    fn from(kind: ScopeKind) -> Self {
        match kind {
            ScopeKind::Export => Visibility::Export,
            ScopeKind::Module => Visibility::Module,
            ScopeKind::File => Visibility::File,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TopLevelKind<'a> {
    Decl(&'a Decl),
    /// `#import "X";` or `X :: #import "X";`.
    Import(&'a Import),
    /// `#load "x.jai";`, and the directive's span.
    Load(&'a str, Span),
    /// `#assert cond;`
    Assert(&'a AssertStmt),
}

/// A statement of a file's top level, `#if` branches included.
#[derive(Clone, Copy, Debug)]
pub struct TopLevel<'a> {
    pub kind: TopLevelKind<'a>,
    pub span: Span,
    pub visibility: Visibility,
    /// Inside a top-level `#if` branch rather than directly at the top level.
    pub nested: bool,
}

/// The declarations, imports, loads and asserts of `file`'s top level, in source order. A scope
/// directive inside an `#if` branch holds to the end of that branch.
pub fn top_level(file: &File) -> Vec<TopLevel<'_>> {
    fn walk<'a>(
        stmts: &'a [Stmt],
        mut visibility: Visibility,
        nested: bool,
        out: &mut Vec<TopLevel<'a>>,
    ) {
        for stmt in stmts {
            let kind = match &stmt.kind {
                StmtKind::Scope(kind) => {
                    visibility = (*kind).into();
                    continue;
                }
                StmtKind::StaticIf(branches) => {
                    walk(&branches.then_branch, visibility, true, out);
                    walk(&branches.else_branch, visibility, true, out);
                    continue;
                }
                StmtKind::Decl(decl) => TopLevelKind::Decl(decl),
                StmtKind::Import(import) => TopLevelKind::Import(import),
                StmtKind::Load {
                    path,
                    span,
                } => TopLevelKind::Load(path, *span),
                StmtKind::Assert(assert) => TopLevelKind::Assert(assert),
                _ => continue,
            };
            out.push(TopLevel {
                kind,
                span: stmt.span,
                visibility,
                nested,
            });
        }
    }
    let mut out = Vec::new();
    walk(&file.stmts, Visibility::Export, false, &mut out);
    out
}

/// The targets an `#assert OS == .A || OS == .B` allows (`[A, B]`); `None` when it names no
/// target, or tests `OS` any other way.
pub fn asserted_targets(assert: &AssertStmt) -> Option<Vec<Sym>> {
    fn walk(expr: &Expr, out: &mut Vec<Sym>) -> Option<()> {
        match &expr.kind {
            ExprKind::Binary(BinOp::And | BinOp::Or, left, right) => {
                walk(left, out)?;
                walk(right, out)?;
            }
            ExprKind::Binary(BinOp::Eq, left, right) if is_os(left) => match &right.kind {
                ExprKind::InferredMember(target) => out.push(target.name),
                _ => return None,
            },
            _ if mentions_os(expr) => return None,
            _ => {}
        }
        Some(())
    }
    fn is_os(expr: &Expr) -> bool {
        matches!(&expr.kind, ExprKind::Ident(name) if name.as_str() == "OS")
    }
    fn mentions_os(expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Binary(_, left, right) => mentions_os(left) || mentions_os(right),
            ExprKind::Unary(_, operand) => mentions_os(operand),
            _ => is_os(expr),
        }
    }
    let mut found = Vec::new();
    walk(&assert.cond, &mut found)?;
    (!found.is_empty()).then_some(found)
}

/// A named procedure (or procedure type) declaration.
#[derive(Clone, Copy, Debug)]
pub struct Procedure<'a> {
    pub name: Ident,
    pub header: &'a ProcHeader,
    /// The notes after the declaration (`f :: () {} @Note`); the header has those before its body.
    pub notes: &'a [Note],
    /// The whole declaration statement.
    pub span: Span,
}

impl Procedure<'_> {
    /// Whether the procedure carries `@name`, before its body or after it.
    pub fn has_note(&self, name: &str) -> bool {
        self.header
            .notes
            .iter()
            .chain(self.notes)
            .any(|note| note.text.trim() == name)
    }
}

/// Every named procedure of `file`, nested ones (in bodies, `#if` branches and structs) included.
pub fn procedures(file: &File) -> Vec<Procedure<'_>> {
    let mut out = Vec::new();
    let mut pending: Vec<&Stmt> = file.stmts.iter().rev().collect();
    while let Some(stmt) = pending.pop() {
        let before = pending.len();
        push_children(stmt, &mut pending);
        if let StmtKind::Decl(decl) = &stmt.kind
            && let Some(header) = decl_header(decl)
        {
            out.extend(decl.names.iter().map(|&name| Procedure {
                name,
                header,
                notes: &decl.notes,
                span: stmt.span,
            }));
        }
        pending[before..].reverse();
    }
    out
}

/// The statements directly inside `stmt` (a procedure's body, a struct's, a branch's).
fn push_children<'a>(stmt: &'a Stmt, out: &mut Vec<&'a Stmt>) {
    match &stmt.kind {
        StmtKind::Decl(decl) => match decl.value.as_ref().map(|v| &v.kind) {
            Some(ExprKind::Proc(lit)) => out.extend(lit.body.iter().flat_map(|b| &b.stmts)),
            Some(ExprKind::Struct(lit)) => out.extend(&lit.body),
            _ => {}
        },
        StmtKind::Block(block) => out.extend(&block.stmts),
        StmtKind::If {
            then_branch,
            else_branch,
            ..
        } => {
            out.push(then_branch);
            out.extend(else_branch.as_deref());
        }
        StmtKind::Switch(switch) | StmtKind::StaticSwitch(switch) => {
            out.extend(switch.cases.iter().flat_map(|c| &c.body));
        }
        StmtKind::While(w) => out.push(&w.body),
        StmtKind::For(f) => out.push(&f.body),
        StmtKind::Defer {
            body, ..
        } => out.push(body),
        StmtKind::PushContext(push) => out.push(&push.body),
        StmtKind::StaticIf(branches) => {
            out.extend(&branches.then_branch);
            out.extend(&branches.else_branch);
        }
        _ => {}
    }
}

/// The names a header introduces with `$`: baked parameters (`$n: int`, `$$n: int`) and the
/// polymorphic variables of its parameter and result types (`x: $T`, `[] $E`, `Table($K, $V)`).
/// Each span is the name's, without the sigil.
pub fn type_parameters(header: &ProcHeader) -> Vec<Ident> {
    fn walk(expr: &Expr, out: &mut Vec<Ident>) {
        match &expr.kind {
            ExprKind::PolyVar {
                name,
                baked,
            } => {
                let sigil = if *baked {
                    2
                } else {
                    1
                };
                out.push(Ident {
                    name: *name,
                    span: Span {
                        start: expr.span.start + sigil,
                        ..expr.span
                    },
                });
            }
            ExprKind::PolyRestricted {
                name, ..
            } => {
                let start = expr.span.start + 1;
                out.push(Ident {
                    name: *name,
                    span: Span {
                        start,
                        end: start + name.as_str().len() as u32,
                        ..expr.span
                    },
                });
            }
            ExprKind::Unary(_, inner) => walk(inner, out),
            ExprKind::ArrayType {
                elem, ..
            } => walk(elem, out),
            ExprKind::Call {
                args, ..
            } => args.iter().for_each(|a| walk(&a.value, out)),
            ExprKind::ProcType(inner) => out.extend(type_parameters(inner)),
            _ => {}
        }
    }
    let mut out: Vec<Ident> = header
        .params
        .iter()
        .filter(|p| p.baked || p.auto_bake)
        .filter_map(|p| p.name)
        .collect();
    let types = header.params.iter().filter_map(|p| p.ty.as_ref());
    for ty in types.chain(header.returns.iter().filter_map(|r| r.ty.as_ref())) {
        walk(ty, &mut out);
    }
    out
}

/// An argument of the call an editor's cursor is in.
#[derive(Clone, Debug, PartialEq)]
pub struct CallArgument {
    /// `name = value`.
    pub name: Option<Sym>,
    /// `..value`.
    pub spread: bool,
    /// The value's text, without the name or `..`; `None` when nothing is written yet.
    pub value: Option<Span>,
    /// The value is a lone number (`3`, `-1.5`).
    pub literal: bool,
}

/// The call around a cursor, from tokens alone, so it holds while the call is unfinished.
#[derive(Clone, Debug, PartialEq)]
pub struct CallSite {
    /// The token before the `(`: the callee's name, or the end of a callee expression.
    pub callee: Span,
    /// The token before the `(` is a name (`f(`, `a.f(`), not a closing bracket (`g()(`).
    pub callee_is_name: bool,
    /// The `(`.
    pub open: u32,
    /// Past the `)`, or where the unfinished call stops (a `;`, a closing bracket, or a
    /// declaration on a following line).
    pub end: u32,
    pub args: Vec<CallArgument>,
    /// The argument the cursor is in.
    pub active: usize,
    /// The name being written at the cursor, through a `:` or `:=` typed after it (empty at
    /// the cursor when no name is started).
    pub word: Span,
    /// The cursor's argument already has its `=`.
    pub has_equals: bool,
}

/// The call whose argument list holds `byte`, when the cursor is where an argument name can be
/// written: at the start of an argument, in the name before its `=`, or in a name not followed
/// by anything else. `None` inside comments and literals, at lexical errors, and inside
/// brackets nested in the arguments.
pub fn call_at(text: &str, tokens: &[Token], lex_errors: &[Span], byte: u32) -> Option<CallSite> {
    // Comments and literals are opaque, commas and brackets included. The prefix also finds a
    // literal or comment not closed yet.
    let file = tokens
        .first()
        .map_or(crate::source::FileId(0), |t| t.span.file);
    let regions = crate::lexer::lexical_regions(file, text)
        .or_else(|_| crate::lexer::lexical_regions(file, &text[..byte as usize]))
        .ok()?;
    if regions.iter().any(|s| s.start <= byte && byte < s.end)
        || lex_errors.iter().any(|e| e.start <= byte && byte <= e.end)
    {
        return None;
    }
    let open = enclosing_paren(tokens, byte)?;
    let before = &tokens[open.checked_sub(1)?];
    if !matches!(
        before.tok,
        Tok::Ident(_) | Tok::Punct(P::RParen | P::RBracket | P::RBrace | P::DotStar)
    ) {
        return None;
    }
    let (groups, active, end) = argument_groups(tokens, open, byte, text.len() as u32)?;
    let current = &groups[active];
    let typed: Vec<&Token> = current
        .iter()
        .copied()
        .filter(|t| t.span.start < byte)
        .collect();
    let word = match typed.as_slice() {
        [] => current
            .first()
            .copied()
            .filter(|t| t.span.start == byte && matches!(t.tok, Tok::Ident(_))),
        [
            t @ Token {
                tok: Tok::Ident(_),
                ..
            },
        ] => Some(*t),
        _ => return None,
    };
    let start = word.map_or(byte, |t| t.span.start);
    let mut word_end = word.map_or(byte, |t| t.span.end);
    let separator = current.get(usize::from(word.is_some()));
    let has_equals = separator.is_some_and(|t| t.tok == Tok::Punct(P::Eq));
    match separator.map(|t| &t.tok) {
        None | Some(Tok::Punct(P::Eq)) => {}
        Some(Tok::Punct(P::Colon | P::ColonEq)) => word_end = separator?.span.end,
        Some(_) => return None,
    }
    Some(CallSite {
        callee: before.span,
        callee_is_name: matches!(before.tok, Tok::Ident(_)),
        open: tokens[open].span.start,
        end,
        args: groups.iter().map(|g| call_argument(g)).collect(),
        active,
        word: Span {
            file,
            start,
            end: word_end,
        },
        has_equals,
    })
}

fn closer(p: P) -> Option<P> {
    match p {
        P::LParen => Some(P::RParen),
        P::LBrace | P::DotBrace => Some(P::RBrace),
        P::LBracket | P::DotBracket => Some(P::RBracket),
        _ => None,
    }
}

/// The index of the innermost `(` still open at `byte`.
fn enclosing_paren(tokens: &[Token], byte: u32) -> Option<usize> {
    let mut stack: Vec<(usize, P)> = Vec::new();
    for (i, t) in tokens
        .iter()
        .enumerate()
        .take_while(|(_, t)| t.span.start < byte)
    {
        let Tok::Punct(p) = t.tok else {
            continue;
        };
        if let Some(close) = closer(p) {
            stack.push((i, close));
        } else if matches!(p, P::RParen | P::RBrace | P::RBracket) {
            if stack.last().is_some_and(|&(_, close)| close == p) {
                stack.pop();
            } else {
                stack.clear();
            }
        } else if p == P::Semi {
            // A semicolon ends a statement, even after an unfinished call.
            while stack.last().is_some_and(|&(_, close)| close != P::RBrace) {
                stack.pop();
            }
        }
    }
    match stack.last()? {
        &(open, P::RParen) => Some(open),
        _ => None,
    }
}

/// The tokens of each argument after the `(` at `open`, the argument holding `byte`, and where
/// the call ends.
fn argument_groups(
    tokens: &[Token],
    open: usize,
    byte: u32,
    text_end: u32,
) -> Option<(Vec<Vec<&Token>>, usize, u32)> {
    let mut groups: Vec<Vec<&Token>> = vec![Vec::new()];
    let mut nested = Vec::new();
    let mut active = 0;
    for (i, t) in tokens.iter().enumerate().skip(open + 1) {
        if nested.is_empty() && t.span.start >= byte && declaration_starts(tokens, i) {
            // A declaration on a following line is where recovery stops, not more of the
            // unfinished argument.
            return Some((groups, active, t.span.start));
        }
        if let Tok::Punct(p) = t.tok {
            if nested.is_empty() {
                match p {
                    P::RParen => return Some((groups, active, t.span.end)),
                    P::Semi | P::RBrace | P::RBracket => {
                        return Some((groups, active, t.span.start));
                    }
                    // `f(a,, b)` overrides the context: no argument names there.
                    P::Comma if groups.len() > 1 && groups.last()?.is_empty() => return None,
                    P::Comma => {
                        groups.push(Vec::new());
                        if t.span.start < byte {
                            active = groups.len() - 1;
                        }
                        continue;
                    }
                    _ => {}
                }
            }
            if let Some(close) = closer(p) {
                nested.push(close);
            } else if nested.last() == Some(&p) {
                nested.pop();
            }
        }
        if t.tok != Tok::Eof {
            groups.last_mut()?.push(t);
        }
    }
    Some((groups, active, text_end))
}

/// `name :`, `name :=` or `name ::` first on its line.
fn declaration_starts(tokens: &[Token], i: usize) -> bool {
    tokens[i].newline_before
        && matches!(tokens[i].tok, Tok::Ident(_))
        && tokens.get(i + 1).is_some_and(|next| {
            matches!(next.tok, Tok::Punct(P::Colon | P::ColonEq | P::ColonColon))
        })
}

fn call_argument(tokens: &[&Token]) -> CallArgument {
    let name = match tokens {
        [
            Token {
                tok: Tok::Ident(name),
                ..
            },
            Token {
                tok: Tok::Punct(P::Eq),
                ..
            },
            ..,
        ] => Some(*name),
        _ => None,
    };
    let rest = &tokens[(usize::from(name.is_some()) * 2).min(tokens.len())..];
    let spread = rest.first().is_some_and(|t| t.tok == Tok::Punct(P::DotDot));
    let value = &rest[usize::from(spread)..];
    let number = |t: &Token| matches!(t.tok, Tok::Int(_) | Tok::Float(_));
    let sign = |t: &Token| matches!(t.tok, Tok::Punct(P::Minus | P::Plus));
    let literal = match value {
        [n] => number(n),
        [s, n] => sign(s) && number(n),
        _ => false,
    };
    CallArgument {
        name,
        spread,
        value: value
            .first()
            .zip(value.last())
            .map(|(first, last)| first.span.to(last.span)),
        literal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::FileId;

    fn parse(text: &str) -> super::super::IdeParse {
        super::super::parse_file_ide(FileId(0), text)
    }

    fn site(text: &str, marker: &str) -> Option<CallSite> {
        let at = text.find(marker).unwrap() as u32;
        let (tokens, errors) = crate::lexer::lex_ide(FileId(0), text);
        let errors: Vec<Span> = errors.iter().map(|e| e.span).collect();
        call_at(text, &tokens, &errors, at)
    }

    #[test]
    fn top_level_tracks_scopes_and_static_if_branches() {
        let file = parse(
            "a :: 1;\n#if X { b :: 2; #scope_file c :: 3; }\nd :: 4;\n#scope_module\ne :: 5;\n#load \"x.jai\";\n",
        );
        let items: Vec<(Visibility, bool)> = top_level(&file.file)
            .iter()
            .map(|t| (t.visibility, t.nested))
            .collect();
        use Visibility::*;
        assert_eq!(
            items,
            [
                (Export, false),
                (Export, true),
                (File, true),
                (Export, false),
                (Module, false),
                (Module, false)
            ]
        );
    }

    #[test]
    fn classes_and_aliases() {
        let file = parse(
            "p :: () {}\nS :: union { x: int; }\nE :: enum_flags { A; }\nT :: #type int;\nA :: []int;\nL :: #library \"l\";\nb :: #bake_arguments p();\nv := 1;\nAlias :: M.Name;\nC :: 1 + 2;\n",
        );
        let classes: Vec<DeclClass> = top_level(&file.file)
            .iter()
            .filter_map(|t| match t.kind {
                TopLevelKind::Decl(d) => Some(decl_class(d)),
                _ => None,
            })
            .collect();
        use DeclClass::*;
        assert_eq!(
            classes,
            [
                Procedure, Struct, Enum, TypeAlias, TypeAlias, Library, Procedure, Variable,
                Constant, Constant
            ]
        );
        let TopLevelKind::Decl(alias) = top_level(&file.file)[8].kind else {
            panic!()
        };
        let path: Vec<&str> = alias_path(alias)
            .unwrap()
            .iter()
            .map(|s| s.as_str())
            .collect();
        assert_eq!(path, ["M", "Name"]);
    }

    #[test]
    fn os_asserts() {
        let targets = |text: &str| {
            let file = parse(text);
            let assert = top_level(&file.file).iter().find_map(|t| match t.kind {
                TopLevelKind::Assert(a) => Some(asserted_targets(a)),
                _ => None,
            })?;
            assert.map(|t| t.iter().map(|s| s.as_str().to_string()).collect::<Vec<_>>())
        };
        assert_eq!(
            targets("#assert OS == .MACOS || OS == .IOS \"Apple only\";"),
            Some(vec!["MACOS".into(), "IOS".into()])
        );
        assert_eq!(targets("#assert OS != .WINDOWS;"), None);
        assert_eq!(targets("#assert SIZE == 8;"), None);
    }

    #[test]
    fn nested_procedures_notes_and_type_parameters() {
        let text = "outer :: () {\n    inner :: (x: $T, $n: int) -> []$R { } @PrintLike\n}\nF :: struct { m :: (a: Table($K, $$V)) {} }\n";
        let file = parse(text);
        let procs = procedures(&file.file);
        let names: Vec<&str> = procs.iter().map(|p| p.name.name.as_str()).collect();
        assert_eq!(names, ["outer", "inner", "m"]);
        assert!(procs[1].has_note("PrintLike") && !procs[0].has_note("PrintLike"));
        let spelled = |p: &Procedure| {
            type_parameters(p.header)
                .iter()
                .map(|i| &text[i.span.start as usize..i.span.end as usize])
                .collect::<Vec<_>>()
        };
        assert_eq!(spelled(&procs[1]), ["n", "T", "R"]);
        assert_eq!(spelled(&procs[2]), ["K", "V"]);
    }

    #[test]
    fn call_sites() {
        let text = "main :: () { f(1, na = 2, ..xs, \n";
        let call = site(text, "na").unwrap();
        assert_eq!(call.active, 1);
        assert!(call.callee_is_name && call.has_equals);
        assert_eq!(
            &text[call.word.start as usize..call.word.end as usize],
            "na"
        );
        assert!(call.args[0].literal && call.args[2].spread);
        assert_eq!(call.args[1].name.map(|s| s.as_str()), Some("na"));
        assert!(site("g(\"a, b\")", "b\"").is_none());
        assert!(site("g(h(1), x)", "1)").is_none());
        assert!(site("g(a,, b)", " b").is_none());
        let expression = site("g()(x)", "x)").unwrap();
        assert!(!expression.callee_is_name);
    }
}

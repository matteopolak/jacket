//! Syntax facts for editors, read from a parsed file and its tokens: what a file declares at
//! its top level, its named procedures with their notes and `$` parameters, and the call an
//! editor's cursor is in. None of these need a compile, so they hold while the program does not
//! type-check; with `parse_file_ide`'s recovery they hold for most text that does not parse.
mod call_site;
mod cursor;

pub use self::call_site::{CallArgument, CallSite, call_at};
pub use self::cursor::{CursorContext, MemberAt, PathDirective, StringAt, cursor_context};
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
        ExprKind::UnknownDirective(directive) if directive.kind.is_library() => DeclClass::Library,
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

/// The spans of the names `decl` introduces, in order: not the places a mixed list
/// (`ok:, t.str = f()`) assigns to.
pub fn decl_name_spans(decl: &Decl) -> Vec<Span> {
    let existing = |i: usize| {
        decl.rare
            .as_ref()
            .is_some_and(|r| r.existing.get(i).copied().unwrap_or(false))
    };
    decl.names
        .iter()
        .enumerate()
        .filter(|&(i, _)| !existing(i))
        .map(|(_, n)| n.span)
        .collect()
}

/// The span of the first name `decl` introduces.
pub fn decl_name_span(decl: &Decl) -> Option<Span> {
    decl_name_spans(decl).first().copied()
}

/// The names declared with no written type, from the tokens alone so they are found wherever
/// the surrounding text stopped parsing: `x` in `x := 1`, both names of `a, b := f()`.
pub fn inferred_name_spans(tokens: &[Token]) -> Vec<Span> {
    let mut out = Vec::new();
    for (colon, t) in tokens.iter().enumerate() {
        if t.tok != Tok::Punct(P::ColonEq) {
            continue;
        }
        let mut at = colon;
        while let Some(name) = at
            .checked_sub(1)
            .filter(|&i| matches!(tokens[i].tok, Tok::Ident(_)))
        {
            out.push(tokens[name].span);
            at = match name.checked_sub(1) {
                Some(comma) if tokens[comma].tok == Tok::Punct(P::Comma) => comma,
                _ => break,
            };
        }
    }
    out
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
        assert_eq!(
            call.args[1].name.map(crate::intern::Sym::as_str),
            Some("na")
        );
        assert!(site("g(\"a, b\")", "b\"").is_none());
        assert!(site("g(h(1), x)", "1)").is_none());
        assert!(site("g(a,, b)", " b").is_none());
        let expression = site("g()(x)", "x)").unwrap();
        assert!(!expression.callee_is_name);
    }

    /// The context at the `|` in `marked`, which is removed from the text.
    fn ctx(marked: &str) -> (String, CursorContext) {
        let offset = marked.find('|').unwrap();
        let text = marked.replacen('|', "", 1);
        let context = cursor_context(&text, offset as u32);
        (text, context)
    }

    fn slice(text: &str, span: Span) -> &str {
        &text[span.start as usize..span.end as usize]
    }

    #[test]
    fn strings_are_found_by_the_lexer() {
        let (text, context) = ctx("#import \"Bas|");
        let CursorContext::String(s) = context else {
            panic!("{context:?}");
        };
        assert!(!s.closed && s.directive == Some(PathDirective::Import));
        assert_eq!(&text[s.start as usize..], "Bas");
        let (_, context) = ctx("#load \"a/b|\";");
        assert!(matches!(
            context,
            CursorContext::String(StringAt {
                closed: true,
                directive: Some(PathDirective::Load),
                ..
            })
        ));
        // Options between the directive and the path.
        let (_, context) = ctx("#import,file \"x|");
        assert!(matches!(
            context,
            CursorContext::String(StringAt {
                directive: Some(PathDirective::Import),
                ..
            })
        ));
        // Not a path.
        let (_, context) = ctx("print(\"he|");
        assert!(matches!(
            context,
            CursorContext::String(StringAt {
                directive: None,
                ..
            })
        ));
    }

    #[test]
    fn escaped_quotes_stay_inside_the_string() {
        let (text, context) = ctx("x := \"a\\\"b|c\";");
        let CursorContext::String(s) = context else {
            panic!("{context:?}");
        };
        assert_eq!(slice(&text, s.span), "\"a\\\"bc\"");
        // After the closing quote it is code again.
        let (_, context) = ctx("x := \"a\\\"b\"|;");
        assert_eq!(context, CursorContext::Code);
        // A quote in a comment opens nothing.
        let (_, context) = ctx("// say \"hi\nf :: |");
        assert_eq!(context, CursorContext::Code);
    }

    #[test]
    fn a_comma_in_a_string_is_not_a_declaration_separator() {
        let text = "a, b := f(\",\");";
        let file = parse(text).file;
        let decl = match &file.stmts[0].kind {
            StmtKind::Decl(d) => d,
            other => panic!("{other:?}"),
        };
        let names: Vec<&str> = decl_name_spans(decl)
            .into_iter()
            .map(|s| slice(text, s))
            .collect();
        assert_eq!(names, ["a", "b"]);
        let tokens = crate::lexer::lex_ide(FileId(0), text).0;
        let inferred: Vec<&str> = inferred_name_spans(&tokens)
            .into_iter()
            .map(|s| slice(text, s))
            .collect();
        assert_eq!(inferred.len(), 2);
        assert!(inferred.contains(&"a") && inferred.contains(&"b"));
        assert_eq!(slice(text, decl_name_span(decl).unwrap()), "a");
        let typed = crate::lexer::lex_ide(FileId(0), "x : int = 1; y = 2; z: int;").0;
        assert!(inferred_name_spans(&typed).is_empty());
        // The cursor in the second name.
        let (_, context) = ctx("a, b| := f(\",\");");
        let CursorContext::DeclName {
            names,
            at,
        } = context
        else {
            panic!("{context:?}");
        };
        assert_eq!((names.len(), at), (2, 1));
        // The comma inside the string is inside a string.
        let (_, context) = ctx("a, b := f(\",|\");");
        assert!(matches!(context, CursorContext::String(_)));
    }

    #[test]
    fn comments_hide_everything_inside() {
        assert!(matches!(
            ctx("x := 1; // note|").1,
            CursorContext::Comment(_)
        ));
        assert!(matches!(
            ctx("/* a \" b| */ x").1,
            CursorContext::Comment(_)
        ));
        assert!(matches!(ctx("/* open|").1, CursorContext::Comment(_)));
        // A `//` inside a string is no comment.
        assert!(matches!(
            ctx("u := \"http://x|\";").1,
            CursorContext::String(_)
        ));
        // The end of a closed block comment is outside it.
        assert_eq!(ctx("/* c */|").1, CursorContext::Code);
    }

    #[test]
    fn member_access_names_the_receiver() {
        let (text, context) = ctx("main :: () { a.b.c|; }");
        let CursorContext::Member(m) = context else {
            panic!("{context:?}");
        };
        assert_eq!(slice(&text, m.word), "c");
        assert_eq!(slice(&text, m.receiver.unwrap()), "a.b");
        let names: Vec<&str> = m.chain.iter().map(|s| slice(&text, *s)).collect();
        assert_eq!(names, ["a", "b"]);
        // Nothing typed yet.
        let (text, context) = ctx("p.|\n");
        let CursorContext::Member(m) = context else {
            panic!("{context:?}");
        };
        assert_eq!((m.word.start, m.word.end), (2, 2));
        assert_eq!(slice(&text, m.receiver.unwrap()), "p");
        // A call or an index is a receiver with no names to look up.
        let (text, context) = ctx("f(x).y|");
        let CursorContext::Member(m) = context else {
            panic!("{context:?}");
        };
        assert_eq!(slice(&text, m.receiver.unwrap()), "f(x)");
        assert!(m.chain.is_empty());
        let (_, context) = ctx("xs[i].|");
        assert!(
            matches!(&context, CursorContext::Member(m) if m.chain.is_empty()
            && m.receiver.is_some())
        );
        // Inferred members have no receiver.
        for text in ["paint(.|", "c == .|", "case .|", "d: Color = .G|"] {
            let (_, context) = ctx(text);
            assert!(
                matches!(&context, CursorContext::Member(m) if m.receiver.is_none()),
                "{text}: {context:?}"
            );
        }
        // A range is not member access.
        assert_eq!(ctx("for 0..ta|").1, CursorContext::Code);
        let (_, context) = ctx("a..b.c|");
        assert!(matches!(&context, CursorContext::Member(m) if m.chain.len() == 1));
        // A comma in a string before the dot changes nothing.
        let (text, context) = ctx("f(\"a.b,\"); q.r|");
        let CursorContext::Member(m) = context else {
            panic!("{context:?}");
        };
        assert_eq!(slice(&text, m.receiver.unwrap()), "q");
    }
}

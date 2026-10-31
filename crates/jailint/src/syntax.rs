//! The file a rule looks at: its syntax tree, tokens and text, the procedures in it, and
//! helpers to walk statements and expressions and to read source text.
use jaic::ast::walk::{Node, walk_node};
use jaic::ast::{Block, Decl, Expr, ExprKind as E, File, ProcHeader, Stmt, StmtKind as S};
use jaic::ide::{Found, IdeCast, IdeExprFact};
use jaic::intern::Sym;
use jaic::lexer::{Comment, P, Tok, Token};
use jaic::sema::{Compiler, EntityId, ProcId};
use jaic::source::{FileId, LineIndex, Span};
use jaic::types::TypeId;

pub(crate) struct Cx<'a> {
    pub compiler: &'a Compiler,
    pub file: FileId,
    pub text: &'a str,
    /// The line index of `text`.
    pub lines: &'a LineIndex,
    pub tokens: &'a [Token],
    /// The file's comments, in order (from `jaic::lexer::lex_with_trivia`).
    pub comments: &'a [Comment],
    /// The file's top-level statements.
    pub stmts: &'a [Stmt],
    pub procs: Vec<ProcSite<'a>>,
    pub imports: Vec<crate::rules::unused_import::Unused>,
    /// Links in doc comments that name nothing (`broken_doc_link`).
    pub doc_links: Vec<crate::rules::broken_doc_link::Broken>,
    /// The compile finished without errors (otherwise some code went unchecked).
    pub complete: bool,
    /// The configured nesting limit (`excessive_nesting`).
    pub max_nesting: usize,
    /// The configured complexity limit (`cognitive_complexity`).
    pub max_complexity: usize,
    /// The configured parameter limit (`too_many_arguments`).
    pub max_arguments: usize,
    /// The configured `bool` parameter limit (`excessive_bools`).
    pub max_bools: usize,
    /// The configured size limit for local arrays in bytes (`large_stack_array`).
    pub max_stack_array_bytes: usize,
    /// Which numbers `magic_number` lets through.
    pub magic_numbers: crate::config::MagicNumbers,
    /// What rules remember across the files of this run.
    pub run: &'a crate::rules::RunState,
}

/// A procedure literal with a body.
pub(crate) struct ProcSite<'a> {
    pub header: &'a ProcHeader,
    pub body: &'a Block,
    /// The declaration naming it (`name :: (...) { ... }`), if any.
    pub decl: Option<&'a Decl>,
    /// Polymorphic, or nested in a polymorphic procedure: checked once per instance, so types
    /// may differ between checks.
    pub poly: bool,
    /// An `#expand` macro (or nested in one): checked where it is expanded, with names that
    /// may belong to the caller.
    pub is_macro: bool,
    /// The compiler checked the whole body (in every instance).
    pub clean: bool,
}

impl ProcSite<'_> {
    /// Type facts recorded for this body describe it fully and mean the same in every check.
    pub fn typed(&self) -> bool {
        self.clean && !self.poly && !self.is_macro
    }
}

impl<'a> Cx<'a> {
    pub fn new(
        compiler: &'a Compiler,
        text: &'a str,
        ast: &'a File,
        tokens: &'a [Token],
        comments: &'a [Comment],
        imports: Vec<crate::rules::unused_import::Unused>,
        run: &'a crate::rules::RunState,
    ) -> Self {
        let mut procs = Vec::new();
        collect_procs(&ast.stmts, None, false, false, compiler, &mut procs);
        Self {
            compiler,
            file: ast.file,
            text,
            lines: compiler.src.sources.get(ast.file).lines(),
            tokens,
            comments,
            stmts: &ast.stmts,
            procs,
            imports,
            doc_links: Vec::new(),
            complete: true,
            max_nesting: crate::config::DEFAULT_MAX_NESTING,
            max_complexity: crate::config::DEFAULT_MAX_COMPLEXITY,
            max_arguments: crate::config::DEFAULT_MAX_ARGUMENTS,
            max_bools: crate::config::DEFAULT_MAX_BOOLS,
            max_stack_array_bytes: crate::config::DEFAULT_MAX_STACK_ARRAY_BYTES,
            magic_numbers: crate::config::MagicNumbers::default(),
            run,
        }
    }

    /// Source text of `span` (empty when out of range).
    pub fn src(&self, span: Span) -> &'a str {
        self.text
            .get(span.start as usize..span.end as usize)
            .unwrap_or("")
    }

    /// Byte range of the expression with the parentheses written around it and around its
    /// edge operands (`Expr::extent`).
    pub fn whole<'n>(&self, node: impl Into<Node<'n>>) -> (usize, usize) {
        let extent = jaic::ast::walk::extent(node.into());
        (extent.start as usize, extent.end as usize)
    }

    /// The source text of [`Cx::whole`].
    pub fn whole_src<'n>(&self, node: impl Into<Node<'n>>) -> &'a str {
        let (start, end) = self.whole(node);
        self.text.get(start..end).unwrap_or("")
    }

    /// The tokens of the expression of `e`, parentheses around it and comments left out.
    fn expression_tokens(&self, e: &Expr) -> impl Iterator<Item = &'a Tok> {
        let (start, end) = self.whole(e);
        let first = self
            .tokens
            .partition_point(|t| (t.span.start as usize) < start);
        let last = self
            .tokens
            .partition_point(|t| (t.span.end as usize) <= end);
        self.tokens[first..last.max(first)].iter().map(|t| &t.tok)
    }

    /// Two expressions are spelled with the same tokens (whitespace and comments aside).
    pub fn same_tokens(&self, a: &Expr, b: &Expr) -> bool {
        self.expression_tokens(a).eq(self.expression_tokens(b))
    }

    /// The two byte ranges hold the same tokens (whitespace and comments aside). Unlike
    /// [`Cx::same_tokens`] the ranges are taken as given: no parentheses are added around them.
    pub fn same_token_ranges(&self, a: Span, b: Span) -> bool {
        let tokens = |s: Span| {
            let first = self.tokens.partition_point(|t| t.span.start < s.start);
            let last = self.tokens.partition_point(|t| t.span.end <= s.end);
            &self.tokens[first..last.max(first)]
        };
        let (a, b) = (tokens(a), tokens(b));
        a.len() == b.len()
            && a.iter()
                .zip(b)
                .all(|(x, y)| self.src(x.span) == self.src(y.span))
    }

    /// The byte offset lies in the body of a macro (an `#expand` procedure, or one nested in it).
    pub fn in_macro(&self, at: usize) -> bool {
        self.procs.iter().any(|p| {
            p.is_macro && (p.body.span.start as usize) <= at && at < p.body.span.end as usize
        })
    }

    /// The file is library code of a module: it, or a directory above it, holds a `module.jai`,
    /// and no `tests` or `test` folder lies between that directory and the file (those hold
    /// programs that use the module). A library is used by programs that decide what is printed
    /// and when the process ends.
    pub fn in_module(&self) -> bool {
        let path = self.compiler.src.sources.get(self.file).path.to_string();
        let path = std::fs::canonicalize(&path).unwrap_or_else(|_| path.into());
        let Some(root) = path
            .ancestors()
            .skip(1)
            .find(|d| jaic::paths::module_entry(d).is_file())
        else {
            return false;
        };
        let below = path.strip_prefix(root).unwrap_or(&path);
        !below.components().any(|c| {
            let c = c.as_os_str();
            c == "tests" || c == "test"
        })
    }

    /// A comment overlaps the bytes `start..end`.
    pub fn has_comment_between(&self, start: usize, end: usize) -> bool {
        let first = self
            .comments
            .partition_point(|c| (c.span.end as usize) <= start);
        self.comments
            .get(first)
            .is_some_and(|c| (c.span.start as usize) < end)
    }

    /// The entities the identifier at `span` resolved to.
    pub fn entities(&self, span: Span) -> Found<EntityId> {
        self.compiler.ide_entities_at(span)
    }

    /// The variables declared at `span` (see `Compiler::ide_locals_declared_at`).
    pub fn locals(&self, span: Span) -> Found<EntityId> {
        self.compiler.ide_locals_declared_at(span)
    }

    /// The procedures made from the header at `span`.
    pub fn procs_with_header(&self, header: Span) -> Found<ProcId> {
        self.compiler.ide_procs_with_header(header)
    }

    /// The type recorded for an expression, if all its checks agreed.
    pub fn ty(&self, e: &Expr) -> Option<jaic::types::TypeId> {
        self.compiler.type_at(e.span)
    }

    /// The expression was a compile-time constant in every check.
    pub fn constant(&self, e: &Expr) -> bool {
        self.compiler
            .expr_fact(e.span)
            .is_some_and(|f| f.constant && !f.conflicting)
    }

    /// An `#insert` directive is written within `start..end`. The tokens are in source order, so
    /// only those in the range are looked at (a scan of the whole file per procedure or variable
    /// made large files quadratic).
    pub fn has_insert(&self, start: u32, end: u32) -> bool {
        let first = self.tokens.partition_point(|t| t.span.start < start);
        self.tokens[first..]
            .iter()
            .take_while(|t| t.span.end <= end)
            .any(|t| t.tok.directive() == Some(jaic::lexicon::directive::INSERT))
    }

    /// A `$` or `$$` token is written within `span` (a polymorphic declaration).
    pub fn has_dollar(&self, span: Span) -> bool {
        let first = self.tokens.partition_point(|t| t.span.start < span.start);
        self.tokens[first..]
            .iter()
            .take_while(|t| t.span.end <= span.end)
            .any(|t| matches!(t.tok, Tok::Punct(P::Dollar | P::DollarDollar)))
    }

    /// Token indices of identifiers spelled `name` within `start..end`, not counting member
    /// names (`x.name`).
    pub fn name_tokens(&self, name: Sym, start: u32, end: u32) -> impl Iterator<Item = usize> {
        let first = self.tokens.partition_point(|t| t.span.start < start);
        (first..self.tokens.len())
            .take_while(move |&i| self.tokens[i].span.end <= end)
            .filter(move |&i| {
                matches!(self.tokens[i].tok, Tok::Ident(n) if n == name)
                    && !(i > 0 && matches!(self.tokens[i - 1].tok, Tok::Punct(P::Dot)))
            })
    }

    /// The file's first lines say a tool generated it (`Automatically generated`,
    /// `auto-generated`, `Generated by`, `from` or `with`, `Regenerate with`, `DO NOT EDIT`, in any case). Generators emit imports,
    /// parameters and locals whether or not the output uses them.
    pub fn generated(&self) -> bool {
        let first = self
            .tokens
            .first()
            .map_or(usize::MAX, |t| t.span.start as usize);
        let head = self
            .text
            .match_indices('\n')
            .nth(9)
            .map_or(self.text.len(), |(i, _)| i);
        self.comments
            .iter()
            .take_while(|c| (c.span.start as usize) < first.min(head))
            .any(|c| {
                let body = c.body(self.text).to_ascii_lowercase();
                [
                    "automatically generated",
                    "auto-generated",
                    "generated by",
                    "generated from",
                    "generated with",
                    "generated only from",
                    "regenerate with",
                    "do not edit",
                    "do not hand-edit",
                ]
                .iter()
                .any(|m| body.contains(m))
            })
    }

    /// The name occurs as an identifier within `start..end` anywhere but `except`.
    pub fn mentions(&self, name: Sym, start: u32, end: u32, except: Span) -> bool {
        self.name_tokens(name, start, end).any(|i| {
            let s = self.tokens[i].span;
            !(except.start <= s.start && s.end <= except.end)
        })
    }

    /// Start of the line holding `offset`, and the indentation written there.
    pub fn line_start(&self, offset: usize) -> (usize, &'a str) {
        let start = jaic::source::line_start(self.text, offset);
        let rest = &self.text[start..];
        let indent = &rest[..rest.len() - rest.trim_start_matches([' ', '\t']).len()];
        (start, indent)
    }

    /// `offset`, moved past the `@note` tokens that follow it with only whitespace between.
    fn past_notes(&self, offset: usize) -> usize {
        let mut end = offset;
        let first = self
            .tokens
            .partition_point(|t| (t.span.start as usize) < offset);
        for t in &self.tokens[first..] {
            if !matches!(t.tok, Tok::Note(_))
                || !self.text[end..t.span.start as usize].trim().is_empty()
            {
                break;
            }
            end = t.span.end as usize;
        }
        end
    }

    /// The byte range of a whole statement line (with its newline) when the statement is alone
    /// on its lines, else just the statement (and the space before a trailing `//` comment,
    /// which stays).
    pub fn statement_removal(&self, span: Span) -> (usize, usize) {
        let (line, _) = self.line_start(span.start as usize);
        let before = &self.text[line..span.start as usize];
        let mut end = span.end as usize;
        let rest = &self.text[end..];
        let line_end = rest.find('\n').map_or(self.text.len(), |n| end + n);
        let after = &self.text[end..line_end];
        // A `;` the statement's span stops before.
        let after_trim = after.trim_start();
        if after_trim.starts_with(';') {
            end += after.len() - after_trim.len() + 1;
        }
        // The `@notes` that follow belong to the statement, on this line or the next ones.
        end = self.past_notes(end);
        let line_end = self.text[end..]
            .find('\n')
            .map_or(self.text.len(), |n| end + n);
        let after = &self.text[end..line_end];
        if before.trim().is_empty() && after.trim().is_empty() {
            let stop = (line_end + 1).min(self.text.len());
            (line, stop)
        } else if before.trim().is_empty() && after.trim().starts_with("//") {
            // A trailing comment stays, on the line the statement leaves.
            let comment = end + (after.len() - after.trim_start().len());
            (span.start as usize, comment)
        } else {
            (span.start as usize, end)
        }
    }
}

impl Cx<'_> {
    /// [`Cx::statement_removal`], also taking the blank lines that follow when the statement
    /// stood alone on its lines right after a blank line or an opening `{`, so removing it
    /// leaves no hole.
    pub fn tidy_removal(&self, span: Span) -> (usize, usize) {
        let (start, mut end) = self.statement_removal(span);
        if self.line_start(start).0 != start {
            return (start, end);
        }
        let before = self.text[..start].trim_end_matches([' ', '\t']);
        if before.is_empty() || before.ends_with("{\n") || before.ends_with("\n\n") {
            loop {
                let rest = &self.text[end..];
                let trimmed = rest.trim_start_matches([' ', '\t']);
                if !trimmed.starts_with('\n') {
                    break;
                }
                end += rest.len() - trimmed.len() + 1;
            }
        }
        (start, end)
    }
}

/// Is the header polymorphic (`$T` binders, baked parameters)?
pub(crate) fn header_is_poly(h: &ProcHeader) -> bool {
    h.params
        .iter()
        .any(|p| p.baked || p.auto_bake || p.ty.as_ref().is_some_and(jaic::sema::procs::has_poly))
        || h.returns
            .iter()
            .any(|r| r.ty.as_ref().is_some_and(jaic::sema::procs::has_poly))
}

/// Every procedure literal with a body under `list`, at any depth (inside other procedures,
/// structs, `#if` branches, `#run` blocks).
fn collect_procs<'a>(
    list: &'a [Stmt],
    decl: Option<&'a Decl>,
    poly: bool,
    is_macro: bool,
    compiler: &Compiler,
    out: &mut Vec<ProcSite<'a>>,
) {
    for s in list {
        let decl = match &s.kind {
            S::Decl(d) => Some(&**d),
            _ => decl.filter(|_| false),
        };
        let mut stack = Vec::new();
        find_procs(
            Node::Stmt(s),
            decl,
            poly,
            is_macro,
            compiler,
            out,
            &mut stack,
        );
    }
}

fn find_procs<'a>(
    node: Node<'a>,
    decl: Option<&'a Decl>,
    poly: bool,
    is_macro: bool,
    compiler: &Compiler,
    out: &mut Vec<ProcSite<'a>>,
    stack: &mut Vec<Node<'a>>,
) {
    walk_node(node, stack, &mut |n, _| {
        let Some(x) = n.expr() else {
            return true;
        };
        match &x.kind {
            E::Proc(lit) => {
                let header = &*lit.header;
                let p = poly || header_is_poly(header);
                let m = is_macro || header.flags.expand();
                // The declaration names this procedure only when it is its value.
                let named = decl.filter(|d| d.value.as_ref().is_some_and(|v| v.span == x.span));
                if let Some(body) = &lit.body {
                    out.push(ProcSite {
                        header,
                        body,
                        decl: named,
                        poly: p,
                        is_macro: m,
                        clean: compiler.ide_body_is_clean(body.span),
                    });
                    collect_procs(&body.stmts, None, p, m, compiler, out);
                }
                false
            }
            E::Struct(st) => {
                collect_procs(&st.body, None, poly, is_macro, compiler, out);
                false
            }
            _ => true,
        }
    });
}

/// Strip parentheses the parser kept in a span: the text of an expression as written.
pub(crate) fn is_atom(e: &Expr) -> bool {
    matches!(
        e.kind,
        E::Ident(_)
            | E::Member(..)
            | E::Call { .. }
            | E::Index(..)
            | E::Int(..)
            | E::Float(..)
            | E::Str(_)
            | E::Bool(_)
            | E::Null
    )
}

/// The identifier an access path starts from: `a` of `a.b[i].c`.
pub(crate) fn root_ident(e: &Expr) -> Option<(Sym, Span)> {
    match &e.kind {
        E::Ident(n) => Some((*n, e.span)),
        E::Member(base, _) | E::Index(base, _) => root_ident(base),
        E::Unary(jaic::ast::UnOp::Deref, base) => root_ident(base),
        _ => None,
    }
}

/// `a`, `a.b`, `a.b.c`: names and member accesses only.
pub(crate) fn is_path(e: &Expr) -> bool {
    match &e.kind {
        E::Ident(_) => true,
        E::Member(base, _) => is_path(base),
        _ => false,
    }
}

/// Lookups on the compiler's records.
pub trait Recorded {
    fn expr_fact(&self, span: Span) -> Option<IdeExprFact>;
    fn cast_fact(&self, span: Span) -> Option<IdeCast>;
    fn is_used(&self, e: EntityId) -> bool;

    /// The type of the expression at `span`, when every check of it agreed.
    fn type_at(&self, span: Span) -> Option<TypeId> {
        self.expr_fact(span)
            .filter(|f| !f.conflicting)
            .map(|f| f.ty)
    }
}

impl Recorded for Compiler {
    fn expr_fact(&self, span: Span) -> Option<IdeExprFact> {
        self.ed.facts()?.exprs.get(&span).copied()
    }

    fn cast_fact(&self, span: Span) -> Option<IdeCast> {
        self.ed.facts()?.casts.get(&span).copied()
    }

    fn is_used(&self, e: EntityId) -> bool {
        self.ed.facts().is_some_and(|i| i.used.contains(&e))
    }
}

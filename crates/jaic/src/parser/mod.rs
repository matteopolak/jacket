//! Recursive-descent parser producing `ast`.
//!
//! The parser works on the token vector from `lexer::lex`. Keywords are plain
//! identifiers, so every keyword check is made in a syntactic position where a
//! keyword is expected (`if`, `for`, `cast`, ...). Types are parsed as ordinary
//! expressions. Strict parsing stops at the first error. Separate IDE entry points
//! recover independent statements without changing CLI acceptance.
//!
//! Layout:
//! - `expr`      precedence climbing, unary/postfix operators, primaries
//! - `directive` `#directive` expressions (`#run`, `#code`, `#type`, ...)
//! - `asm`       `#asm` blocks (instructions, operands, register declarations)
//! - `procedure` procedure headers, parameters, return lists, lambdas
//! - `aggregate` struct/union/enum literals
//! - `stmt`      statements and control flow
//! - `decl`      declarations (names, types, values, `using` / `#as` modifiers, flags)
//! - `directive_stmt` statement-level directives (`#import`, `#load`, `#if`, `#run`, ...)
mod aggregate;
mod asm;
pub mod asm_ide;
mod decl;
mod directive;
mod directive_stmt;
mod expr;
mod ide;
pub mod ide_facts;
mod procedure;
mod stmt;
#[cfg(test)]
mod tests;

use crate::ast::{File, Ident, Note, Stmt};
pub use crate::lexer::PlatformTarget;
use crate::lexer::{P, Tok, Token, lex};
use crate::source::{Applicability, Diagnostic, DiagnosticKind, FileId, Span, Suggestion};
pub(crate) use directive_stmt::GENERIC_DIRECTIVES;
pub(crate) use ide::parse_callee_ide;
pub use ide::{IdeParse, ParseReuse, SharedParse, parse_file_ide, parse_tokens_ide};
pub(crate) use stmt::check_bits;

pub(crate) type PResult<T> = Result<T, Diagnostic>;

/// Deepest nesting of expressions and statements the parser accepts. The parser and everything
/// after it recurse on the tree, so without a bound a file of a few kilobytes of `(` or `{`
/// overflows the stack. Real code stays far below this.
pub const MAX_NESTING: usize = 1000;

/// Parses one source file.
pub fn parse_file(file: FileId, text: &str) -> Result<File, Diagnostic> {
    let tokens = lex(file, text)?;
    let mut parser = Parser::new(text, tokens);
    let stmts = parser.parse_file_stmts()?;
    Ok(File {
        file,
        stmts,
        lenient: parser.lenient,
    })
}

/// `parse_file`, for a library file whose dead file-level `#if` branches need be neither
/// lexed nor parsed (see [`PlatformTarget`]).
/// Everything a branch could declare is absent from the tree, so a syntax error inside a branch
/// the target rules out goes unreported: use it only for files the project ships and tests on
/// every platform.
pub fn parse_file_for_target(
    file: FileId,
    text: &str,
    target: PlatformTarget,
) -> Result<File, Diagnostic> {
    let tokens = crate::lexer::lex_for_target(file, text, target)?;
    let mut parser = Parser::new(text, tokens);
    parser.platform = Some(target);
    let stmts = parser.parse_file_stmts()?;
    Ok(File {
        file,
        stmts,
        lenient: parser.lenient,
    })
}

/// Parses `text` from its tokens (`lexer::lex` of the same text and `file`), handing the
/// tokens back so a caller that also needs them does not lex twice.
pub fn parse_tokens(
    file: FileId,
    text: &str,
    tokens: Vec<Token>,
) -> (Result<File, Diagnostic>, Vec<Token>) {
    let mut parser = Parser::new(text, tokens);
    let parsed = parser.parse_file_stmts().map(|stmts| File {
        file,
        stmts,
        lenient: std::mem::take(&mut parser.lenient),
    });
    (parsed, parser.toks)
}

/// Parses `tokens` (the lexing of `text`, whose spans may have been moved) as one expression
/// that fills them all.
pub fn parse_expression(text: &str, tokens: Vec<Token>) -> Result<crate::ast::Expr, Diagnostic> {
    let mut parser = Parser::new(text, tokens);
    let expr = parser.parse_expr()?;
    if !parser.at_eof() {
        return Err(parser.expected("the end of the expression", ""));
    }
    Ok(expr)
}

pub(crate) struct Parser<'a> {
    src: &'a str,
    toks: Vec<Token>,
    pos: usize,
    /// True inside argument and parameter lists, where a comma separates items
    /// instead of continuing a multi-value return list.
    in_list: bool,
    /// True while parsing a return type: a procedure type there (`-> (int, int) -> bool`) is not
    /// a literal, whatever follows it, because the `{` that follows belongs to the procedure
    /// being declared.
    in_return_type: bool,
    /// True while parsing an `#asm` `?T` size: it ends at whitespace, so `mov?T [p], v` keeps `[p]`
    /// as the operand instead of indexing `T`.
    in_asm_size: bool,
    /// Operator text (`+`, `[]`, ...) for the procedure header parsed next.
    pending_operator: Option<std::rc::Rc<str>>,
    /// Token index just past a brace-terminated construct that ended with trailing flags
    /// (`struct {..} #no_padding`); such a statement needs no `;`.
    block_end: usize,
    /// Notes parsed in the middle of a construct, attached when it completes.
    pending_notes: Vec<Note>,
    /// Declarations an expression needs before its statement (`(#import "M").f()`), moved in
    /// front of that statement by the enclosing statement list.
    hoisted: Vec<Stmt>,
    /// True while parsing the statement of a `#code` literal: a closing `)`, `]` or `,` of the
    /// enclosing expression ends it like a `;` would.
    in_code_stmt: bool,
    /// Whether the condition of the `#assert` just parsed contained a braced body.
    assert_cond_has_body: bool,
    /// True while parsing the braces of a `#code` block written inside an argument list, up to a
    /// procedure body: the official compiler reads a comma there as the end of an argument.
    code_in_args: bool,
    /// True directly in a struct or union body: a member may be named `null`, `true` or `false`
    /// (it is reached as `x.null`), unlike a variable, constant or parameter.
    in_aggregate_body: bool,
    /// Expressions and statements currently open (see `MAX_NESTING`).
    depth: usize,
    /// Set only by the editor entry points. Strict parsing never resynchronizes.
    recovery: Option<Vec<Diagnostic>>,
    ide_closed_braces: crate::fxhash::HashSet<u32>,
    /// Only an IDE query sets this: capture one real callee before argument parsing fails.
    ide_call_open: Option<u32>,
    ide_call_callee: Option<crate::ast::Expr>,
    /// Set by `parse_file_for_target`.
    platform: Option<PlatformTarget>,
    /// The `depth` at which a statement is still at file level: `#if` branches at file level
    /// are file level too, anything else (a block, a struct body) is not, since a name such as
    /// `OS` may be declared there.
    file_level_depth: usize,
    /// Constructs accepted although the official compiler rejects them.
    lenient: Vec<crate::ast::ParsedLeniency>,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(src: &'a str, toks: Vec<Token>) -> Self {
        debug_assert!(matches!(toks.last().map(|t| &t.tok), Some(Tok::Eof)));
        Self {
            src,
            toks,
            pos: 0,
            in_list: false,
            in_return_type: false,
            in_asm_size: false,
            pending_operator: None,
            block_end: usize::MAX,
            pending_notes: Vec::new(),
            hoisted: Vec::new(),
            in_code_stmt: false,
            assert_cond_has_body: false,
            code_in_args: false,
            in_aggregate_body: false,
            depth: 0,
            recovery: None,
            ide_closed_braces: crate::fxhash::HashSet::default(),
            ide_call_open: None,
            ide_call_callee: None,
            platform: None,
            file_level_depth: 1,
            lenient: Vec::new(),
        }
    }

    /// Note that the code at `span` is a `kind` the official compiler rejects.
    fn note_lenient(&mut self, kind: crate::leniency::Leniency, span: Span) {
        let found = crate::ast::ParsedLeniency {
            kind,
            span,
        };
        // A speculative parse may read the same text twice.
        if !self.lenient.contains(&found) {
            self.lenient.push(found);
        }
    }

    /// Run `f` one nesting level deeper, failing past `MAX_NESTING`.
    fn nested<T>(&mut self, f: impl FnOnce(&mut Self) -> PResult<T>) -> PResult<T> {
        if self.depth >= MAX_NESTING {
            return Err(self
                .error(format!(
                    "code is nested too deeply (more than {MAX_NESTING} levels)"
                ))
                .with_kind(DiagnosticKind::NestingLimit));
        }
        self.depth += 1;
        let result = f(self);
        self.depth -= 1;
        result
    }

    fn parse_file_stmts(&mut self) -> PResult<Vec<Stmt>> {
        let mut stmts = Vec::new();
        while !self.at_eof() {
            let hoisted = self.hoisted.len();
            if let Some(stmt) = self.parse_stmt_recovering(None)? {
                stmts.extend(self.hoisted.split_off(hoisted));
                stmts.push(stmt);
            }
        }
        Ok(stmts)
    }

    // -- token access -----------------------------------------------------

    fn tok(&self) -> &Tok {
        self.tok_at(0)
    }

    fn tok_at(&self, n: usize) -> &Tok {
        &self.toks[(self.pos + n).min(self.toks.len() - 1)].tok
    }

    fn token_at(&self, n: usize) -> &Token {
        &self.toks[(self.pos + n).min(self.toks.len() - 1)]
    }

    fn span(&self) -> Span {
        self.token_at(0).span
    }

    fn prev_span(&self) -> Span {
        self.toks[self.pos.saturating_sub(1)].span
    }

    fn newline_before(&self) -> bool {
        self.token_at(0).newline_before
    }

    /// True if the previous token is a `#string` here-string, which ends a statement by itself.
    fn prev_is_here_string(&self) -> bool {
        let Some(prev) = self.pos.checked_sub(1).map(|i| &self.toks[i]) else {
            return false;
        };
        matches!(prev.tok, Tok::Str(_))
            && self.src.as_bytes().get(prev.span.start as usize) == Some(&b'#')
    }

    fn at_eof(&self) -> bool {
        matches!(self.tok(), Tok::Eof)
    }

    /// Consumes the current token and returns its span.
    fn bump(&mut self) -> Span {
        let span = self.span();
        if self.pos + 1 < self.toks.len() {
            self.pos += 1;
        }
        span
    }

    fn at(&self, p: P) -> bool {
        self.at_n(0, p)
    }

    fn at_n(&self, n: usize, p: P) -> bool {
        matches!(self.tok_at(n), Tok::Punct(q) if *q == p)
    }

    fn eat(&mut self, p: P) -> bool {
        let found = self.at(p);
        if found {
            self.bump();
        }
        found
    }

    fn expect(&mut self, p: P, context: &str) -> PResult<Span> {
        if self.at(p) {
            Ok(self.bump())
        } else {
            Err(self.expected(&format!("`{}`", p.text()), context))
        }
    }

    /// Text of the identifier at offset `n`, if the token is an identifier.
    fn kw_at(&self, n: usize) -> Option<&'static str> {
        match self.tok_at(n) {
            Tok::Ident(s) => Some(s.as_str()),
            _ => None,
        }
    }

    fn kw(&self) -> Option<&'static str> {
        self.kw_at(0)
    }

    fn at_kw(&self, kw: &str) -> bool {
        self.kw() == Some(kw)
    }

    fn eat_kw(&mut self, kw: &str) -> bool {
        let found = self.at_kw(kw);
        if found {
            self.bump();
        }
        found
    }

    /// Name of the directive at offset `n`, if the token is a directive.
    fn directive_at(&self, n: usize) -> Option<&'static str> {
        match self.tok_at(n) {
            Tok::Directive(s) => Some(s.as_str()),
            _ => None,
        }
    }

    fn directive(&self) -> Option<&'static str> {
        self.directive_at(0)
    }

    fn at_directive(&self, name: &str) -> bool {
        self.directive() == Some(name)
    }

    fn ident(&mut self, context: &str) -> PResult<Ident> {
        match self.tok() {
            Tok::Ident(name) => {
                let name = *name;
                let span = self.bump();
                Ok(Ident {
                    name,
                    span,
                })
            }
            _ => Err(self.expected("identifier", context)),
        }
    }

    /// A name being declared. `null`, `true` and `false` always read as their literals, so a
    /// variable, constant or parameter of that name could never be used.
    fn reject_literal_name(&self, name: Ident) -> PResult<Ident> {
        let word = name.name.as_str();
        if is_builtin_type_name(word) {
            return Err(Diagnostic::error(
                name.span,
                format!("`{word}` is the name of a builtin type and cannot be declared"),
            )
            .with_kind(crate::source::DiagnosticKind::ShadowsBuiltinType)
            .with_help("choose another name"));
        }
        if self.in_aggregate_body || !matches!(word, "null" | "true" | "false") {
            return Ok(name);
        }
        Err(Diagnostic::error(
            name.span,
            format!("`{word}` is a keyword and cannot be declared"),
        )
        .with_help(format!(
            "`{word}` always means the literal; use another name"
        )))
    }

    /// Consumes a string literal token.
    fn string_lit(&mut self, context: &str) -> PResult<(std::rc::Rc<[u8]>, Span)> {
        match self.tok() {
            Tok::Str(s) => {
                let s = s.clone();
                Ok((s, self.bump()))
            }
            _ => Err(self.expected("string literal", context)),
        }
    }

    /// Consumes any trailing `@note` tokens.
    fn parse_notes(&mut self) -> Vec<Note> {
        let after_block = self.at_prev(P::RBrace);
        let mut notes = Vec::new();
        while let Tok::Note(text) = self.tok() {
            let text = text.clone();
            let span = self.bump();
            notes.push(Note {
                text,
                span,
            });
        }
        // `} @Note` still ends the statement with its block.
        if after_block && !notes.is_empty() {
            self.block_end = self.pos;
        }
        notes
    }

    // -- diagnostics ------------------------------------------------------

    fn error(&self, message: impl Into<String>) -> Diagnostic {
        Diagnostic::error(self.span(), message)
    }

    /// A suggestion that inserts `text` where the previous token ends.
    fn insert_after_previous(
        &self,
        title: impl Into<String>,
        text: &str,
        applicability: Applicability,
    ) -> Suggestion {
        let end = self.prev_span().end;
        let at = Span {
            start: end,
            ..self.prev_span()
        };
        Suggestion::insert(title, at, text, applicability)
    }

    /// `error` for a missing `;`, with the suggestion that writes it where the previous token
    /// ends. It is certain when the next token starts a new line or closes a block, and a guess
    /// when more of the same line follows.
    fn missing_semicolon(&self, error: Diagnostic) -> Diagnostic {
        let certain =
            self.newline_before() || self.at_eof() || matches!(self.tok(), Tok::Punct(P::RBrace));
        let applicability = if certain {
            Applicability::MachineApplicable
        } else {
            Applicability::MaybeIncorrect
        };
        error.with_suggestion(
            self.insert_after_previous("end the statement here with `;`", ";", applicability)
                .with_action("Insert `;`"),
        )
    }

    /// "expected `}` <context>" for the block opened at `open`: the note points back at the
    /// `{`, and a block left open at the end of the file gets the `}` that closes it.
    fn unclosed_block(&self, open: Span, context: &str) -> Diagnostic {
        let error = self
            .expected("`}`", context)
            .with_note(open, "the block opens here");
        if !self.at_eof() {
            return error;
        }
        let end = self.src.len() as u32;
        let at = Span {
            start: end,
            end,
            ..self.span()
        };
        let text = if self.src.ends_with('\n') {
            "}\n"
        } else {
            "\n}\n"
        };
        error.with_suggestion(
            Suggestion::insert(
                "close the block at the end of the file with `}`",
                at,
                text,
                Applicability::MaybeIncorrect,
            )
            .with_action("Insert `}`"),
        )
    }

    /// "expected X <context>, found Y" at the current token.
    fn expected(&self, what: &str, context: &str) -> Diagnostic {
        let context = if context.is_empty() {
            String::new()
        } else {
            format!(" {context}")
        };
        self.error(format!(
            "expected {what}{context}, found {}",
            describe(self.tok())
        ))
    }
}

/// Human-readable token description for diagnostics.
fn describe(tok: &Tok) -> String {
    match tok {
        Tok::Ident(s) => format!("`{s}`"),
        Tok::Directive(s) => format!("`#{s}`"),
        Tok::Note(s) => format!("`@{s}`"),
        Tok::Int(_) | Tok::Float(_) => "number".into(),
        Tok::Str(_) => "string literal".into(),
        Tok::Punct(p) => format!("`{}`", p.text()),
        Tok::Eof => "end of file".into(),
    }
}

/// Whether `word` names a builtin type, which no declaration may take as its name.
pub(super) fn is_builtin_type_name(word: &str) -> bool {
    matches!(
        word,
        "int"
            | "float"
            | "bool"
            | "string"
            | "void"
            | "Any"
            | "Code"
            | "s8"
            | "s16"
            | "s32"
            | "s64"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "float32"
            | "float64"
    )
}

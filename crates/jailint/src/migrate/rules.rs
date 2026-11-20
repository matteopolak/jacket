//! The migrations, oldest change first. Each is a `Migration` constant and an entry in
//! [`MIGRATIONS`]; `docs/tools/jacket-migrate.md` lists them for users.
use super::{Answer, CompileError, Detect, Migration, Source};
use jaic::lexer::{P, Tok, Token};

/// Every migration, in the order the language changed.
pub static MIGRATIONS: &[Migration] = &[
    CONTEXT_TYPE_DIRECTIVE,
    FOREIGN_LIBRARY_DIRECTIVE,
    VECTOR_COMPONENT_CONSTRUCTOR,
    REMOVED_BASIC_PROCEDURES,
    ALLOCATOR_CONTEXT_ARGUMENT,
    VERSIONED_MODULE_IMPORT,
];

/// `Context` as a type became `#Context`.
const CONTEXT_TYPE_DIRECTIVE: Migration = Migration {
    name: "context-type-directive",
    summary: "the context's type is written `#Context`, not `Context`",
    ambiguous: false,
    detect: Detect::Error {
        codes: &["unknown-identifier"],
        answer: context_type,
    },
};

/// An unknown `Context` is the old spelling of the context's type: code that declares or
/// imports its own `Context` has no error here and is left alone.
fn context_type(e: &CompileError, source: &Source) -> Option<Answer> {
    (source.slice(e.start, e.end) == "Context").then(|| Answer::Edit {
        start: e.start,
        end: e.end,
        text: "#Context".into(),
    })
}

/// `#foreign_library` and `#foreign_system_library` became `#library` and `#system_library`.
const FOREIGN_LIBRARY_DIRECTIVE: Migration = Migration {
    name: "foreign-library-directive",
    summary: "`#foreign_library` and `#foreign_system_library` are `#library` and `#system_library`",
    ambiguous: false,
    detect: Detect::Tokens(foreign_library),
};

/// The old directive names mean exactly what the new ones do, so every occurrence is renamed.
fn foreign_library(source: &Source) -> Vec<Answer> {
    source
        .tokens
        .iter()
        .filter_map(|t| {
            let new = match t.tok.directive()? {
                "foreign_library" => "#library",
                "foreign_system_library" => "#system_library",
                _ => return None,
            };
            Some(Answer::Edit {
                start: t.span.start as usize,
                end: t.span.end as usize,
                text: new.into(),
            })
        })
        .collect()
}

/// `make_vector2(x, y)` and its siblings lost their one-argument-per-component overloads.
const VECTOR_COMPONENT_CONSTRUCTOR: Migration = Migration {
    name: "vector-component-constructor",
    summary: "`make_vector2(x, y)` (one argument per component) is the literal `Vector2.{x, y}`",
    ambiguous: false,
    detect: Detect::Error {
        codes: &["argument-count", "call-no-overload-matches-arguments"],
        answer: vector_constructor,
    },
};

/// A call `make_vectorN` with exactly N arguments that no longer matches becomes the struct
/// literal of `VectorN`, whose members are the components in order. A qualifier stays:
/// `Math.make_vector3(a, b, c)` becomes `Math.Vector3.{a, b, c}`.
fn vector_constructor(e: &CompileError, source: &Source) -> Option<Answer> {
    let call = enclosing_call(source.tokens, e.start)?;
    let Tok::Ident(name) = &source.tokens[call.callee].tok else {
        return None;
    };
    let (ty, components) = match name.as_str() {
        "make_vector2" => ("Vector2", 2),
        "make_vector3" => ("Vector3", 3),
        "make_vector4" => ("Vector4", 4),
        _ => return None,
    };
    if call.arguments(source.tokens) != components {
        return None;
    }
    let open = &source.tokens[call.open];
    let close = &source.tokens[call.close];
    let inner = source.slice(open.span.end as usize, close.span.start as usize);
    Some(Answer::Edit {
        start: source.tokens[call.callee].span.start as usize,
        end: close.span.end as usize,
        text: format!("{ty}.{{{inner}}}"),
    })
}

/// Basic procedures removed with no drop-in replacement: what to do instead.
const REMOVED_BASIC_PROCEDURES: Migration = Migration {
    name: "removed-basic-procedures",
    summary: "`get_time` and `bubble_sort` were removed; says what to use instead",
    ambiguous: false,
    detect: Detect::Error {
        codes: &["unknown-identifier"],
        answer: removed_procedure,
    },
};

fn removed_procedure(e: &CompileError, source: &Source) -> Option<Answer> {
    let message = match source.slice(e.start, e.end) {
        "get_time" => {
            "`get_time` was removed; `seconds_since_init()` gives seconds as a `float64` since the program started, check that is what this code measures"
        }
        "bubble_sort" => {
            "`bubble_sort` was removed with no replacement; sort with a procedure of your own"
        }
        _ => return None,
    };
    Some(Answer::Manual {
        start: e.start,
        end: e.end,
        message: message.into(),
    })
}

/// Allocator parameters (`New(T, allocator = a)`) were removed; the allocator goes in the
/// context arguments: `New(T,, allocator = a)`.
const ALLOCATOR_CONTEXT_ARGUMENT: Migration = Migration {
    name: "allocator-context-argument",
    summary: "`f(x, allocator = a)` passes the allocator in the context: `f(x,, allocator = a)`",
    ambiguous: false,
    detect: Detect::Error {
        codes: &["call-mismatch"],
        answer: allocator_argument,
    },
};

/// The error points at the value of a named argument the procedure no longer has. When that
/// argument is `allocator = ...` among the ordinary arguments, the comma before it becomes
/// `,,`, which makes it, and everything after it, a context argument.
fn allocator_argument(e: &CompileError, source: &Source) -> Option<Answer> {
    let tokens = source.tokens;
    let value = tokens
        .iter()
        .position(|t| t.span.start as usize == e.start)?;
    let [.., comma, name, eq] = tokens.get(..value)? else {
        return None;
    };
    let is = |t: &Token, p: P| t.tok == Tok::Punct(p);
    if !is(eq, P::Eq)
        || !matches!(&name.tok, Tok::Ident(n) if n.as_str() == "allocator")
        || !is(comma, P::Comma)
    {
        return None;
    }
    // Already after `,,`: the error is about something else.
    let call = enclosing_call(tokens, e.start)?;
    if tokens[call.open..value]
        .windows(2)
        .any(|w| is(&w[0], P::Comma) && is(&w[1], P::Comma))
    {
        return None;
    }
    Some(Answer::Edit {
        start: comma.span.start as usize,
        end: comma.span.end as usize,
        text: ",,".into(),
    })
}

/// Modules shipped under a versioned folder name (`freetype-2.12.1`) dropped the version.
const VERSIONED_MODULE_IMPORT: Migration = Migration {
    name: "versioned-module-import",
    summary: "`#import \"name-1.2.3\"` of a module now shipped as `name`",
    ambiguous: false,
    detect: Detect::Error {
        codes: &["module-not"],
        answer: versioned_import,
    },
};

/// A missing `name-<digits>[.<digits>...]` import where `name` exists becomes `name`.
fn versioned_import(e: &CompileError, source: &Source) -> Option<Answer> {
    let span = source.slice(e.start, e.end);
    let open = span.find('"')?;
    let close = span[open + 1..].find('"')? + open + 1;
    let name = &span[open + 1..close];
    let (base, version) = name.rsplit_once('-')?;
    let is_version = !version.is_empty()
        && version
            .split('.')
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()));
    if !is_version || base.is_empty() || !source.module_exists(base) {
        return None;
    }
    Some(Answer::Edit {
        start: e.start + open + 1,
        end: e.start + close,
        text: base.to_string(),
    })
}

/// A call's tokens: the procedure's name, `(` and the matching `)`.
struct Call {
    callee: usize,
    open: usize,
    close: usize,
}

impl Call {
    /// How many arguments are between the parentheses (commas at depth one, plus one).
    fn arguments(&self, tokens: &[Token]) -> usize {
        if self.close == self.open + 1 {
            return 0;
        }
        let mut depth = 0usize;
        let mut count = 1;
        for t in &tokens[self.open + 1..self.close] {
            match t.tok {
                Tok::Punct(P::LParen | P::LBracket | P::LBrace | P::DotBrace | P::DotBracket) => {
                    depth += 1;
                }
                Tok::Punct(P::RParen | P::RBracket | P::RBrace) => depth = depth.saturating_sub(1),
                Tok::Punct(P::Comma) if depth == 0 => count += 1,
                _ => {}
            }
        }
        count
    }
}

/// The innermost call whose name or parentheses hold `offset`.
fn enclosing_call(tokens: &[Token], offset: usize) -> Option<Call> {
    let at = tokens.iter().position(|t| {
        (t.span.start as usize) <= offset
            && offset < (t.span.end as usize).max(t.span.start as usize + 1)
    })?;
    // On the name itself, or the start of a qualified one (`Math.make_vector3(...)`), where
    // the error points at the callee.
    let mut name = at;
    while matches!(tokens[name].tok, Tok::Ident(_))
        && tokens
            .get(name + 1)
            .is_some_and(|t| t.tok == Tok::Punct(P::Dot))
        && tokens
            .get(name + 2)
            .is_some_and(|t| matches!(t.tok, Tok::Ident(_)))
    {
        name += 2;
    }
    let open = if matches!(tokens[name].tok, Tok::Ident(_))
        && tokens
            .get(name + 1)
            .is_some_and(|t| t.tok == Tok::Punct(P::LParen))
    {
        name + 1
    } else {
        let mut depth = 0usize;
        let mut i = at;
        loop {
            match tokens[i].tok {
                Tok::Punct(P::RParen) if i != at => depth += 1,
                Tok::Punct(P::LParen) if depth == 0 => break i,
                Tok::Punct(P::LParen) => depth -= 1,
                _ => {}
            }
            i = i.checked_sub(1)?;
        }
    };
    let callee = open.checked_sub(1)?;
    if !matches!(tokens[callee].tok, Tok::Ident(_)) {
        return None;
    }
    let mut depth = 0usize;
    let close = tokens[open..].iter().position(|t| match t.tok {
        Tok::Punct(P::LParen) => {
            depth += 1;
            false
        }
        Tok::Punct(P::RParen) => {
            depth -= 1;
            depth == 0
        }
        _ => false,
    })? + open;
    Some(Call {
        callee,
        open,
        close,
    })
}

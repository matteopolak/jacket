//! The migrations, oldest change first. Each is a `Migration` constant and an entry in
//! [`MIGRATIONS`]; `docs/tools/jacket-migrate.md` lists them for users.
use jaic::lexer::{P, Tok, Token};

use super::{Answer, CompileError, Detect, Migration, Source};

/// Every migration, in the order the language changed.
pub static MIGRATIONS: &[Migration] = &[
    CONTEXT_TYPE_DIRECTIVE,
    FOREIGN_LIBRARY_DIRECTIVE,
    VECTOR_COMPONENT_CONSTRUCTOR,
    RENAMED_IDENTIFIERS,
    RENAMED_MEMBERS,
    ALLOCATOR_CONTEXT_ARGUMENT,
    RENAMED_ARGUMENTS,
    ASSERT_MESSAGE_POSITIONAL,
    VERSIONED_MODULE_IMPORT,
    REMOVED_MODULES,
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
    let (call, ty, components) = calls_containing(source.tokens, e.start)
        .into_iter()
        .find_map(|call| {
            let Tok::Ident(name) = &source.tokens[call.callee].tok else {
                return None;
            };

            match name.as_str() {
                "make_vector2" => Some((call, "Vector2", 2)),
                "make_vector3" => Some((call, "Vector3", 3)),
                "make_vector4" => Some((call, "Vector4", 4)),
                _ => None,
            }
        })?;

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

/// Names that were renamed with the same meaning: the old name, then the new one. A bare `Swap`
/// or `get_time` that the program declares or imports itself resolves, so it never reaches the
/// checker's `unknown-identifier` and is left alone.
const RENAMED_IDENTIFIER_TABLE: &[(&str, &str)] = &[
    ("Swap", "swap"),
    ("get_time", "seconds_since_init"),
    // `bubble_sort(array, compare)` and `quick_sort(array, compare)` take the same arguments.
    // Equal elements may change places.
    ("bubble_sort", "quick_sort"),
    ("__temporary_allocator", "temp"),
    (
        "compiler_get_command_line_arguments",
        "get_command_line_arguments",
    ),
    ("Build_Output_Type", "Build_Options.Output_Type"),
];

/// Procedures and types that were renamed (`Swap` is `swap`).
const RENAMED_IDENTIFIERS: Migration = Migration {
    name: "renamed-identifiers",
    summary: "`Swap`, `get_time`, `bubble_sort`, `__temporary_allocator` and a few more have new names",
    ambiguous: false,
    detect: Detect::Error {
        codes: &["unknown-identifier"],
        answer: renamed_identifier,
    },
};

/// An unknown identifier from the table gets its new name. The checker's error is the proof that
/// nothing else of that name is in scope.
fn renamed_identifier(e: &CompileError, source: &Source) -> Option<Answer> {
    let old = source.slice(e.start, e.end);
    let (_, new) = RENAMED_IDENTIFIER_TABLE.iter().find(|(o, _)| *o == old)?;

    Some(Answer::Edit {
        start: e.start,
        end: e.end,
        text: (*new).into(),
    })
}

/// Enum members that were renamed: the old name, then the new one.
const RENAMED_MEMBER_TABLE: &[(&str, &str)] = &[("COMPILATION_PHASE", "PHASE")];

/// Enum members that were renamed (`.COMPILATION_PHASE` is `.PHASE`).
const RENAMED_MEMBERS: Migration = Migration {
    name: "renamed-members",
    summary: "the message kind `.COMPILATION_PHASE` is `.PHASE`",
    ambiguous: false,
    detect: Detect::Error {
        codes: &["no-member"],
        answer: renamed_member,
    },
};

/// The error points at the member's name; a leading `.` stays.
fn renamed_member(e: &CompileError, source: &Source) -> Option<Answer> {
    let at = source.slice(e.start, e.end);
    let name = at.strip_prefix('.').unwrap_or(at);
    let (_, new) = RENAMED_MEMBER_TABLE.iter().find(|(o, _)| *o == name)?;
    let start = e.end - name.len();

    Some(Answer::Edit {
        start,
        end: e.end,
        text: (*new).into(),
    })
}

/// Named arguments that were renamed: the procedure's argument name, then the new one.
const RENAMED_ARGUMENT_TABLE: &[(&str, &str)] = &[("follow_symlinks", "follow_directory_symlinks")];

/// Named arguments that were renamed (`file_list(p, follow_symlinks = f)`).
const RENAMED_ARGUMENTS: Migration = Migration {
    name: "renamed-arguments",
    summary: "`file_list(..., follow_symlinks = f)` is `follow_directory_symlinks = f`",
    ambiguous: false,
    detect: Detect::Error {
        codes: &["call-mismatch"],
        answer: renamed_argument,
    },
};

/// The error points at the value of a named argument the procedure no longer has; the name is
/// the two tokens before it (`name =`).
fn renamed_argument(e: &CompileError, source: &Source) -> Option<Answer> {
    let value = source
        .tokens
        .iter()
        .position(|t| t.span.start as usize == e.start)?;
    let [.., name, eq] = source.tokens.get(..value)? else {
        return None;
    };
    let Tok::Ident(old) = &name.tok else {
        return None;
    };

    if eq.tok != Tok::Punct(P::Eq) {
        return None;
    }

    let (_, new) = RENAMED_ARGUMENT_TABLE
        .iter()
        .find(|(o, _)| *o == old.as_str())?;

    Some(Answer::Edit {
        start: name.span.start as usize,
        end: name.span.end as usize,
        text: (*new).into(),
    })
}

/// `#assert(cond, message = "...")` takes the message by position.
const ASSERT_MESSAGE_POSITIONAL: Migration = Migration {
    name: "assert-message-positional",
    summary: "`#assert(c, message = \"m\")` is `#assert(c, \"m\")`",
    ambiguous: false,
    detect: Detect::Error {
        codes: &["parse-expected-token"],
        answer: assert_message,
    },
};

/// The parse error is reported on the `#assert` directive. Inside its parentheses, a top-level
/// argument written `message = value` loses the name and the `=`, which leaves the value in place.
fn assert_message(e: &CompileError, source: &Source) -> Option<Answer> {
    let tokens = source.tokens;
    let at = tokens
        .iter()
        .position(|t| t.span.start as usize == e.start)?;

    if tokens[at].tok.directive() != Some("assert")
        || tokens.get(at + 1)?.tok != Tok::Punct(P::LParen)
    {
        return None;
    }

    let mut depth = 0usize;

    for i in at + 1..tokens.len() {
        match tokens[i].tok {
            Tok::Punct(P::LParen | P::LBracket | P::LBrace | P::DotBrace | P::DotBracket) => {
                depth += 1;
            }
            Tok::Punct(P::RParen | P::RBracket | P::RBrace) => {
                depth -= 1;

                if depth == 0 {
                    return None;
                }
            }
            Tok::Ident(ref n)
                if depth == 1
                    && n.as_str() == "message"
                    && tokens[i - 1].tok == Tok::Punct(P::Comma)
                    && tokens.get(i + 1)?.tok == Tok::Punct(P::Eq) =>
            {
                return Some(Answer::Edit {
                    start: tokens[i].span.start as usize,
                    end: tokens.get(i + 2)?.span.start as usize,
                    text: String::new(),
                });
            }
            _ => {}
        }
    }

    None
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
/// argument is `allocator = ...` among the ordinary arguments, it moves to the end of the list
/// after `,,`, which makes it a context argument: `f(a, allocator = x, b = 1)` becomes
/// `f(a, b = 1,, allocator = x)`, since everything after `,,` is a context argument.
fn allocator_argument(e: &CompileError, source: &Source) -> Option<Answer> {
    let tokens = source.tokens;
    let value = tokens
        .iter()
        .position(|t| t.span.start as usize == e.start)?;
    let [.., before, name, eq] = tokens.get(..value)? else {
        return None;
    };
    let is = |t: &Token, p: P| t.tok == Tok::Punct(p);
    // The argument follows a comma, or the call's `(` when it is the first one.
    let first = is(before, P::LParen);

    if !is(eq, P::Eq)
        || !matches!(&name.tok, Tok::Ident(n) if n.as_str() == "allocator")
        || !(first || is(before, P::Comma))
    {
        return None;
    }

    let comma = before;
    let call = calls_containing(tokens, e.start).into_iter().next()?;
    // Already after `,,`: the error is about something else.
    let double_comma = |range: &[Token]| {
        range
            .windows(2)
            .any(|w| is(&w[0], P::Comma) && is(&w[1], P::Comma))
    };

    if double_comma(&tokens[call.open..value]) {
        return None;
    }

    // The argument ends at the next comma at the call's own depth, or at its `)`.
    let mut depth = 0usize;
    let mut end = call.close;

    for (i, t) in tokens.iter().enumerate().take(call.close).skip(value) {
        match t.tok {
            Tok::Punct(P::LParen | P::LBracket | P::LBrace | P::DotBrace | P::DotBracket) => {
                depth += 1;
            }
            Tok::Punct(P::RParen | P::RBracket | P::RBrace) => depth = depth.saturating_sub(1),
            Tok::Punct(P::Comma) if depth == 0 => {
                end = i;
                break;
            }
            _ => {}
        }
    }

    if double_comma(&tokens[end..call.close]) {
        return None;
    }

    let argument_end = tokens[end].span.start as usize;
    let list_end = tokens[call.close].span.start as usize;
    let rest = source.slice(argument_end, list_end);

    if first {
        // Nothing before it: the rest of the list, then `,,` and the argument.
        let rest = rest.strip_prefix(',').unwrap_or(rest).trim_start();
        let argument = source.slice(name.span.start as usize, argument_end);

        return Some(Answer::Edit {
            start: comma.span.end as usize,
            end: list_end,
            text: format!("{rest},, {argument}"),
        });
    }

    let argument = source.slice(comma.span.end as usize, argument_end);

    Some(Answer::Edit {
        start: comma.span.start as usize,
        end: list_end,
        text: format!("{rest},,{argument}"),
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

/// Modules removed from the standard distribution: what to use instead.
const REMOVED_MODULES: Migration = Migration {
    name: "removed-modules",
    summary: "`#import \"Render\"` names a module that no longer exists; says where its parts went",
    ambiguous: false,
    detect: Detect::Error {
        codes: &["module-not"],
        answer: removed_module,
    },
};

/// An import of a missing module that was removed: no rewrite can say which of its parts the
/// program uses, so the note says where they live now.
fn removed_module(e: &CompileError, source: &Source) -> Option<Answer> {
    let span = source.slice(e.start, e.end);
    let message = if span.contains("\"Render\"") {
        "the `Render` module was removed; its font drawing (`Dynamic_Font`) is in `Simp`"
    } else {
        return None;
    };

    Some(Answer::Manual {
        start: e.start,
        end: e.end,
        message: message.into(),
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

/// The calls (a name directly followed by parentheses) that hold `offset` in their name or their
/// parentheses, innermost first. `cast(float) x` and `xx (y)` are calls here too; callers pick
/// the callee they want by name.
fn calls_containing(tokens: &[Token], offset: usize) -> Vec<Call> {
    let mut open_parens: Vec<usize> = Vec::new();
    let mut calls = Vec::new();

    for (i, t) in tokens.iter().enumerate() {
        match t.tok {
            Tok::Punct(P::LParen) => open_parens.push(i),
            Tok::Punct(P::RParen) => {
                let Some(open) = open_parens.pop() else {
                    continue;
                };
                let Some(callee) = open.checked_sub(1) else {
                    continue;
                };

                if !matches!(tokens[callee].tok, Tok::Ident(_)) {
                    continue;
                }

                // A qualified callee (`Math.make_vector3(`) starts at its first name.
                let mut head = callee;

                while head >= 2
                    && tokens[head - 1].tok == Tok::Punct(P::Dot)
                    && matches!(tokens[head - 2].tok, Tok::Ident(_))
                {
                    head -= 2;
                }

                if (tokens[head].span.start as usize) <= offset && offset < t.span.end as usize {
                    calls.push(Call {
                        callee,
                        open,
                        close: i,
                    });
                }
            }
            _ => {}
        }
    }

    calls.sort_by_key(|c| c.close - c.open);
    calls
}

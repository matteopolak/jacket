//! Completion that inserts what a procedure or keyword needs around its name: the call
//! parentheses of a procedure and the skeleton of `if`, `for`, `while` and `case`. Both are
//! snippets, so they need a client that advertises `snippetSupport`.
use crate::analysis::KEYWORDS;
use crate::model::{CompletionItem, CompletionKind};
use jaic::lexer::{P, Tok, Token, lex_with_trivia};
use jaic::sema::ide::IdeCall;
use jaic::source::FileId;

/// What accepting a procedure completion inserts (`completion.callSnippets`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CallSnippets {
    /// The name alone.
    None,
    /// `name(` and `)` with the cursor between them (`name()` when nothing can be passed).
    #[default]
    AddParentheses,
    /// As `AddParentheses`, with a placeholder for each parameter a call must pass.
    FillArguments,
}

impl CallSnippets {
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "none" => Some(Self::None),
            "addParentheses" => Some(Self::AddParentheses),
            "fillArguments" => Some(Self::FillArguments),
            _ => None,
        }
    }
}

/// Builtins that take their arguments like a call but are listed as keywords.
pub(crate) const CALL_KEYWORDS: &[&str] = &[
    "size_of",
    "type_of",
    "code_of",
    "initializer_of",
    "type_info",
];

/// Calls whose argument is read as a type or a name rather than called.
const NAMING_CALLS: &[&str] = &["type_of", "type_info", "size_of", "cast", "code_of"];

/// Where a completed word stands, read from the tokens around it.
pub(crate) struct Site {
    /// A call's parentheses fit after the word.
    pub parens: bool,
    /// The word starts a statement and nothing follows it on the line.
    pub statement: bool,
    /// The word is directly inside the braces of an `if x == { ... }`.
    pub in_case_block: bool,
}

pub(crate) fn site(text: &str, start: usize, end: usize) -> Site {
    let mut tokens = lex_with_trivia(FileId(0), text).0;
    if tokens.last().is_some_and(|t| t.tok == Tok::Eof) {
        tokens.pop();
    }
    let before = tokens.partition_point(|t| (t.span.end as usize) <= start);
    let after = tokens.partition_point(|t| (t.span.start as usize) < end);
    let prev = before.checked_sub(1).map(|i| &tokens[i]);
    let before_prev = before.checked_sub(2).map(|i| &tokens[i]);
    let next = tokens.get(after).filter(|t| !t.newline_before);
    let punct = |t: Option<&Token>, p: P| t.is_some_and(|t| t.tok == Tok::Punct(p));
    let ident = |t: Option<&Token>, names: &[&str]| {
        t.is_some_and(|t| matches!(&t.tok, Tok::Ident(n) if names.contains(&n.as_str())))
    };

    // Text that makes a call wrong or redundant.
    let called = punct(next, P::LParen);
    let declared = [P::Colon, P::ColonColon, P::ColonEq]
        .iter()
        .any(|&p| punct(next, p));
    let procedure_of_call = |t: Option<&Token>| {
        t.is_some_and(|t| matches!(&t.tok, Tok::Directive(n) if n.as_str() == "procedure_of_call"))
    };
    let named = procedure_of_call(prev)
        || (punct(prev, P::LParen)
            && (procedure_of_call(before_prev) || ident(before_prev, NAMING_CALLS)));
    // `*name` takes an address; `a * name` multiplies.
    let address_of = punct(prev, P::Star)
        && !before_prev.is_some_and(|t| match &t.tok {
            Tok::Int(_) | Tok::Float(_) | Tok::Str(_) | Tok::Punct(P::RParen | P::RBracket) => true,
            Tok::Ident(n) => {
                !KEYWORDS.contains(&n.as_str()) || matches!(n.as_str(), "true" | "false" | "null")
            }
            _ => false,
        });
    // A procedure used as a value: `f :: name;`, and a global's initialiser (which cannot
    // call at run time).
    let depth: i32 = tokens[..before]
        .iter()
        .map(|t| match t.tok {
            Tok::Punct(P::LBrace | P::DotBrace) => 1,
            Tok::Punct(P::RBrace) => -1,
            _ => 0,
        })
        .sum();
    let initialiser = punct(prev, P::ColonColon) || (punct(prev, P::ColonEq) && depth <= 0);
    // `x: name`, `-> name`, `$name`: a type is written there.
    let type_position = [P::Colon, P::Arrow, P::Dollar, P::DollarDollar]
        .iter()
        .any(|&p| punct(prev, p));

    let line_start = text[..start].rfind('\n').map_or(0, |i| i + 1);
    let starts_line = text[line_start..start].trim().is_empty();
    let statement = next.is_none()
        && (starts_line
            || prev.is_none()
            || [P::Semi, P::LBrace, P::RBrace]
                .iter()
                .any(|&p| punct(prev, p))
            || ident(prev, &["else"]));
    Site {
        parens: !(called || declared || named || address_of || initialiser || type_position),
        statement,
        in_case_block: enclosing_case_block(&tokens[..before]),
    }
}

/// The innermost open brace follows `==`: the body of `if x == {`.
fn enclosing_case_block(tokens: &[Token]) -> bool {
    let mut depth = 0i32;
    for (i, t) in tokens.iter().enumerate().rev() {
        match t.tok {
            Tok::Punct(P::RBrace) => depth += 1,
            Tok::Punct(P::LBrace | P::DotBrace) if depth > 0 => depth -= 1,
            Tok::Punct(P::LBrace) => {
                return i > 0 && tokens[i - 1].tok == Tok::Punct(P::EqEq);
            }
            Tok::Punct(P::DotBrace) => return false,
            _ => {}
        }
    }
    false
}

/// The snippet that calls `name`, and whether the cursor ends up inside the parentheses.
fn call_text(name: &str, call: &IdeCall, mode: CallSnippets) -> (String, bool) {
    if call.parameterless {
        return (format!("{name}()$0"), false);
    }
    if mode == CallSnippets::FillArguments
        && let Some(required) = call.required.as_ref().filter(|r| !r.is_empty())
    {
        let placeholders: Vec<String> = required
            .iter()
            .enumerate()
            .map(|(i, p)| format!("${{{}:{}}}", i + 1, p))
            .collect();
        return (format!("{name}({})$0", placeholders.join(", ")), true);
    }
    (format!("{name}($0)"), true)
}

/// The skeleton a statement keyword expands to.
fn keyword_text(name: &str) -> Option<&'static str> {
    Some(match name {
        "if" => "if ${1:condition} {\n    $0\n}",
        "while" => "while ${1:condition} {\n    $0\n}",
        "for" => "for ${1:iterable} {\n    $0\n}",
        _ => return None,
    })
}

/// Turn procedures into calls and statement keywords into skeletons where the client can
/// take snippets and the text around the word allows it.
pub(crate) fn apply(
    items: &mut [CompletionItem],
    text: &str,
    start: usize,
    end: usize,
    mode: CallSnippets,
) {
    let wants = |item: &CompletionItem| {
        (item.call.is_some() && mode != CallSnippets::None)
            || (item.kind == CompletionKind::Keyword
                && item.call.is_none()
                && (keyword_text(&item.label).is_some() || item.label == "case"))
    };
    if !items.iter().any(wants) {
        return;
    }
    let site = site(text, start, end);
    for item in items.iter_mut().filter(|i| wants(i)) {
        if let Some(call) = &item.call {
            if site.parens {
                let (snippet, inside) = call_text(&item.label, call, mode);
                item.insert_text = Some(snippet);
                item.snippet = true;
                item.trigger_signature_help = inside;
            }
            continue;
        }
        let snippet = match item.label.as_str() {
            "case" if site.statement && site.in_case_block => Some("case ${1:value};\n$0"),
            "case" => None,
            other if site.statement => keyword_text(other),
            _ => None,
        };
        if let Some(snippet) = snippet {
            item.insert_text = Some(snippet.into());
            item.snippet = true;
        }
    }
}

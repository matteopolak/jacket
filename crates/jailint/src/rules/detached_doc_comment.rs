//! `detached_doc_comment`: a comment that describes a declaration but is separated from it by a
//! blank line.
//!
//! ```jai
//! // Splits `text` at every `sep` and returns the pieces.
//!
//! split :: (text: string, sep: u8) -> [] string { ... }
//! ```
//!
//! A doc comment is the run of `//` lines directly above a declaration (see
//! [doc comments](../compiler/doc-comments.md)); with a blank line between them it is an
//! ordinary comment, and hover, completion and the generated docs show nothing. Usually an
//! edit left the gap behind.
//!
//! Looks at top-level procedures, structs and enums. Fires when the own-line `//` or `/* */`
//! comments just above a blank line directly above the declaration read as a sentence (at
//! least three words, starting with a capital letter or a backtick and ending in `.`, `!` or
//! `?`, which leaves out section headings and commented-out code, and not a `jailint:`
//! directive or a `TODO`-style marker). A
//! comment at the very top of the file, before any code, is a file header and is left alone.
//! The editor offers to close the gap; `--fix` does not, since a comment above a gap may be
//! meant for the section below it. Off by default (`allow`): section comments written that way
//! on purpose are the common case.
use crate::syntax::Cx;
use crate::{Edit, Finding, Fix};
use jaic::ast::{DeclKind, ExprKind as E, StmtKind as S};
use jaic::lexer::{Comment, CommentKind};

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "detached_doc_comment",
    category: crate::config::Category::Pedantic,
    summary: "a comment that reads as a declaration's docs but has a blank line between it and the declaration",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    if cx.generated() {
        return;
    }
    for stmt in cx.stmts {
        let S::Decl(decl) = &stmt.kind else {
            continue;
        };
        let ([name], Some(value)) = (decl.names.as_slice(), &decl.value) else {
            continue;
        };
        if decl.kind != DeclKind::Const
            || !matches!(value.kind, E::Proc(_) | E::Struct(_) | E::Enum(_))
        {
            continue;
        }
        let at = stmt.span.start as usize;
        let first = cx.tokens.partition_point(|t| (t.span.start as usize) < at);
        // Before any code the comments are the file's header.
        let Some(previous) = first.checked_sub(1).map(|i| &cx.tokens[i]) else {
            continue;
        };
        let between: Vec<&Comment> = cx
            .comments
            .iter()
            .filter(|c| {
                c.span.start >= previous.span.end
                    && (c.span.end as usize) <= at
                    && own_line(cx.text, c)
            })
            .collect();
        let Some(last) = between.last() else {
            continue;
        };
        let end = last.span.end as usize;
        if cx.text[end..at].matches('\n').count() < 2 || !cx.text[end..at].trim().is_empty() {
            continue;
        }
        let run = run_ending_at(cx.text, &between);
        if !reads_as_docs(cx.text, run) {
            continue;
        }
        // The blank lines: from the end of the comment's line to the start of the declaration's.
        let gap_from = end + cx.text[end..at].find('\n').map_or(0, |n| n + 1);
        let gap_to = jaic::source::line_start(cx.text, at);
        out.push(Finding {
            start: last.span.start as usize,
            end,
            message: format!(
                "this comment is separated from `{}` by a blank line, so it is not its doc comment",
                name.name.as_str()
            ),
            label: None,
            help: Some("remove the blank line to document it".into()),
            fix: Some(Fix {
                title: "remove the blank line".into(),
                edits: vec![Edit {
                    start: gap_from,
                    end: gap_to,
                    text: String::new(),
                }],
                machine_applicable: false,
            }),
        });
    }
}

/// The comment starts its line.
fn own_line(text: &str, c: &Comment) -> bool {
    let start = c.span.start as usize;
    text[jaic::source::line_start(text, start)..start]
        .trim()
        .is_empty()
}

/// The comments at the end of `comments` that follow one another on adjacent lines.
fn run_ending_at<'a>(text: &str, comments: &'a [&'a Comment]) -> &'a [&'a Comment] {
    let mut from = comments.len() - 1;
    while from > 0 {
        let gap = &text[comments[from - 1].span.end as usize..comments[from].span.start as usize];
        if gap.matches('\n').count() != 1 {
            break;
        }
        from -= 1;
    }
    &comments[from..]
}

/// Prose, not commented-out code, a marker or a directive.
fn reads_as_docs(text: &str, run: &[&Comment]) -> bool {
    if run
        .iter()
        .any(|c| c.kind == CommentKind::Block && !c.terminated)
    {
        return false;
    }
    let lines: Vec<&str> = run.iter().map(|c| c.body(text).trim()).collect();
    let (Some(first), Some(last)) = (lines.first(), lines.last()) else {
        return false;
    };
    let capital = first
        .chars()
        .next()
        .is_some_and(|c| c.is_uppercase() || c == '`');
    let words = lines
        .iter()
        .map(|l| l.split_whitespace().count())
        .sum::<usize>();
    let sentence = last.ends_with(['.', '!', '?']);
    let marker = super::todo_marker::MARKERS
        .iter()
        .any(|m| lines.iter().any(|l| l.starts_with(m)));
    let directive = lines.iter().any(|l| l.starts_with("jailint:"));
    capital && words >= 3 && sentence && !marker && !directive
}

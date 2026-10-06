//! Lints as text, through `jaic`'s shared diagnostic renderer (`jaic::render`). In the plain
//! layout (output that is not a terminal) a lint keeps its long-standing rustc-like form:
//!
//! ```text
//! warning[index_only_loop]: `i` is only used to index `xs`
//!  --> sum.jai:4:5
//!   |
//! 4 |     for i: 0..xs.count-1 {
//!   |     ^^^^^^^^^^^^^^^^^^^^ loop over `xs` itself
//!   |
//! help: iterate over the elements of `xs` and use `it`
//!   |
//! 4 ~     for xs {
//! 5 ~         total += it;
//!   |
//!   = note: `index_only_loop` is `warn` by default
//! ```
//!
//! On a terminal it gains context lines, multi-line spans, colour and (with UTF-8) box drawing.
use crate::{Level, Lint};
use jaic::render::{Fix, Help, Label, Report, Severity};
pub use jaic::render::{Style, line_col};

/// `lint` in `text` (the file's contents), its path shown as `path`.
pub fn render(lint: &Lint, text: &str, path: &str, default: Level, style: &Style) -> String {
    let severity = match lint.level {
        Level::Deny => Severity::Error,
        _ => Severity::Warning,
    };
    let mut report = Report::new(severity, lint.message.clone());
    report.compact = false;
    report.code = Some(lint.rule);
    report.primary = Some(Label {
        path,
        text,
        start: lint.start,
        end: lint.end,
        message: lint.label.clone(),
    });
    if let Some(help) = &lint.help {
        report.help.push(Help {
            message: help.clone(),
            fix: lint.fix.as_ref().map(|fix| Fix {
                text,
                edits: fix
                    .edits
                    .iter()
                    .map(|e| (e.start, e.end, e.text.clone()))
                    .collect(),
            }),
        });
    }
    if lint.level == default {
        report.notes.push(format!(
            "`{}` is `{}` by default",
            lint.rule,
            default.as_str()
        ));
    }
    report.render_with(*style)
}

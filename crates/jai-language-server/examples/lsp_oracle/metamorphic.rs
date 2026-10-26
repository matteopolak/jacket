//! Check (c): answers that must not change when only the layout does.
//!
//! * `c-blank`, `c-format`: a fresh session on the file with blank lines inserted, or as `jaifmt`
//!   writes it, answers like one on the original once positions are mapped through the tokens
//!   (both texts have the same tokens; only the whitespace and comments between them differ).
//! * `c-edit`: the original session, edited into that text a few gaps at a time with requests in
//!   between, answers exactly like a fresh session opened on it.
use super::{Finding, byte_of, pos_of};
use jai_language_server::{
    Diagnostic, DocumentUri, Hover, Location, Position, Range, Session, TextChange,
};
use jaic::lexer::{Tok, Token};
use jaic::source::FileId;

/// A position in a text, as the token it falls in (and the offset in it) or the gap after one.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Place {
    In(usize, usize),
    After(Option<usize>, usize),
    /// Another document, or a position that does not map.
    Elsewhere(String),
}

struct Layout<'a> {
    text: &'a str,
    uri: &'a DocumentUri,
    tokens: Vec<Token>,
}

impl<'a> Layout<'a> {
    fn new(text: &'a str, uri: &'a DocumentUri) -> Option<Self> {
        let tokens = jaic::lexer::lex(FileId(0), text)
            .ok()?
            .into_iter()
            .filter(|t| !matches!(t.tok, Tok::Eof))
            .collect();
        Some(Self {
            text,
            uri,
            tokens,
        })
    }

    fn place(&self, position: Position) -> Place {
        let Some(byte) = byte_of(self.text, position) else {
            return Place::Elsewhere(format!("{position:?}"));
        };
        let i = self
            .tokens
            .partition_point(|t| t.span.start as usize <= byte);
        match i.checked_sub(1) {
            Some(j) if byte < self.tokens[j].span.end as usize => {
                Place::In(j, byte - self.tokens[j].span.start as usize)
            }
            // Gaps differ in length between the texts: where in one is not comparable.
            j => Place::After(j, 0),
        }
    }

    fn location(&self, l: &Location) -> Place {
        if l.uri == self.uri.as_str() {
            self.place(l.range.start)
        } else {
            Place::Elsewhere(format!(
                "{}:{}:{}",
                l.uri, l.range.start.line, l.range.start.character
            ))
        }
    }

    fn same_tokens(&self, other: &Layout<'_>) -> bool {
        self.tokens.len() == other.tokens.len()
            && self.tokens.iter().zip(&other.tokens).all(|(a, b)| {
                self.text[a.span.start as usize..a.span.end as usize]
                    == other.text[b.span.start as usize..b.span.end as usize]
            })
    }

    /// Indices of the identifier tokens, `cap` of them spread over the file.
    fn identifiers(&self, cap: usize) -> Vec<usize> {
        let all: Vec<usize> = self
            .tokens
            .iter()
            .enumerate()
            .filter(
                |(_, t)| matches!(&t.tok, Tok::Ident(n) if !jaic::lexicon::is_reserved(n.as_str())),
            )
            .map(|(i, _)| i)
            .collect();
        let step = all.len().div_ceil(cap.max(1)).max(1);
        all.into_iter().step_by(step).collect()
    }
}

/// What a session answers about a text, positions as [`Place`]s.
#[derive(Debug, PartialEq, Eq)]
struct View {
    diagnostics: Vec<(Place, String)>,
    probes: Vec<Probe>,
}

/// A token, the hover there and where definition goes.
type Probe = (usize, Option<(Place, String)>, Vec<Place>);

/// Hover text without whitespace, when the layout of what it quotes may differ. A `line:column`
/// pair in an evaluated constant (a program that prints where a literal is) moves with the layout
/// too, so its numbers are dropped.
fn squeeze(s: &str, loose: bool) -> String {
    if !loose {
        return s.to_owned();
    }
    let text: Vec<char> = s.split_whitespace().flat_map(str::chars).collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < text.len() {
        let digits = |from: usize| {
            text[from..]
                .iter()
                .take_while(|c| c.is_ascii_digit())
                .count()
        };
        let first = digits(i);
        if first > 0 && text.get(i + first) == Some(&':') && digits(i + first + 1) > 0 {
            out.push_str("#:#");
            i += first + 1 + digits(i + first + 1);
        } else {
            out.push(text[i]);
            i += 1;
        }
    }
    out
}

/// `on line 353` moves with the text: compare messages without the number.
fn without_line_numbers(message: &str) -> String {
    let mut out = String::with_capacity(message.len());
    let mut rest = message;
    while let Some(i) = rest.find("line ") {
        out.push_str(&rest[..i + 5]);
        rest = rest[i + 5..].trim_start_matches(|c: char| c.is_ascii_digit());
    }
    out.push_str(rest);
    out
}

fn view(session: &Session, layout: &Layout<'_>, probes: &[usize], loose: bool) -> View {
    let mut diagnostics: Vec<(Place, String)> = session
        .diagnostics(layout.uri)
        .unwrap_or_default()
        .iter()
        .map(|d: &Diagnostic| {
            (
                layout.place(d.range.start),
                format!(
                    "{:?} {:?} {}",
                    d.severity,
                    d.code,
                    squeeze(&without_line_numbers(&d.message), loose)
                ),
            )
        })
        .collect();
    diagnostics.sort();
    let probes = probes
        .iter()
        .map(|&i| {
            let at = pos_of(layout.text, layout.tokens[i].span.start as usize);
            let hover = session
                .hover(layout.uri, at)
                .ok()
                .flatten()
                .map(|h: Hover| {
                    (
                        layout.place(h.range.start),
                        // Hover quotes declarations, whose layout blank lines change too.
                        squeeze(&h.contents.value, true),
                    )
                });
            let mut definitions: Vec<Place> = session
                .definition(layout.uri, at)
                .unwrap_or_default()
                .iter()
                .map(|l| layout.location(l))
                .collect();
            definitions.sort();
            (i, hover, definitions)
        })
        .collect();
    View {
        diagnostics,
        probes,
    }
}

/// The first difference between two views, described with `layout`'s positions.
fn difference(a: &View, b: &View, layout: &Layout<'_>) -> Option<(Position, String, String)> {
    for ((pa, ma), (pb, mb)) in a.diagnostics.iter().zip(&b.diagnostics) {
        if pa != pb || ma != mb {
            return Some((
                Position::default(),
                "diagnostic".into(),
                format!("{pa:?} {ma} / {pb:?} {mb}"),
            ));
        }
    }
    if a.diagnostics.len() != b.diagnostics.len() {
        let (long, short) = if a.diagnostics.len() > b.diagnostics.len() {
            (a, b)
        } else {
            (b, a)
        };
        let extra = &long.diagnostics[short.diagnostics.len()];
        return Some((
            Position::default(),
            "diagnostic".into(),
            format!(
                "{} vs {} diagnostics, first extra {extra:?}",
                a.diagnostics.len(),
                b.diagnostics.len()
            ),
        ));
    }
    for (x, y) in a.probes.iter().zip(&b.probes) {
        if x != y {
            let token = &layout.tokens[x.0];
            let name = layout.text[token.span.start as usize..token.span.end as usize].to_owned();
            let at = pos_of(layout.text, token.span.start as usize);
            let what = if x.1 == y.1 {
                format!("definition {:?} / {:?}", x.2, y.2)
            } else {
                format!("hover {:?} / {:?}", x.1, y.1)
            };
            return Some((at, name, what));
        }
    }
    None
}

/// `text` with a blank line before every third line that starts with a token and does not follow
/// a comment (a blank line there would detach a doc comment).
pub fn with_blank_lines(text: &str) -> String {
    let Ok(tokens) = jaic::lexer::lex(FileId(0), text) else {
        return text.to_owned();
    };
    let mut at: Vec<usize> = Vec::new();
    for t in tokens
        .iter()
        .filter(|t| t.newline_before && !matches!(t.tok, Tok::Eof))
    {
        let start = t.span.start as usize;
        let line = text[..start].rfind('\n').map_or(0, |i| i + 1);
        if !text[line..start].trim().is_empty() || line == 0 {
            continue;
        }
        let previous = text[..line - 1].rsplit('\n').next().unwrap_or("").trim();
        if previous.starts_with("//") || previous.ends_with("*/") || previous.contains("//") {
            continue;
        }
        at.push(line);
    }
    let mut out = String::with_capacity(text.len() + at.len());
    let mut last = 0;
    for (n, &line) in at.iter().enumerate() {
        if n % 3 == 0 {
            out.push_str(&text[last..line]);
            out.push('\n');
            last = line;
        }
    }
    out.push_str(&text[last..]);
    out
}

/// What `jaifmt --stdin` makes of `text`, when it formats it.
pub fn formatted(jaifmt: &std::path::Path, text: &str) -> Option<String> {
    use std::io::Write;
    let mut child = std::process::Command::new(jaifmt)
        .arg("--stdin")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;
    let mut stdin = child.stdin.take()?;
    let input = text.to_owned();
    let writer = std::thread::spawn(move || stdin.write_all(input.as_bytes()));
    let output = child.wait_with_output().ok()?;
    writer.join().ok()?.ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8(output.stdout).ok())
        .flatten()
}

/// The gaps between tokens that differ, as (start, end) in `from` and the text to put there.
fn gap_edits(from: &Layout<'_>, to: &Layout<'_>) -> Vec<(usize, usize, String)> {
    let mut edits = Vec::new();
    let (mut a, mut b) = (0usize, 0usize);
    for (x, y) in from
        .tokens
        .iter()
        .zip(&to.tokens)
        .map(|(x, y)| (x.span, y.span))
    {
        let (ga, gb) = (
            &from.text[a..x.start as usize],
            &to.text[b..y.start as usize],
        );
        if ga != gb {
            edits.push((a, x.start as usize, gb.to_owned()));
        }
        a = x.end as usize;
        b = y.end as usize;
    }
    if from.text[a..] != to.text[b..] {
        edits.push((a, from.text.len(), to.text[b..].to_owned()));
    }
    edits
}

pub struct Context<'a> {
    pub uri: &'a DocumentUri,
    pub text: &'a str,
    pub fresh: &'a dyn Fn(&str) -> Option<Session>,
    pub probes: usize,
}

/// Compares a fresh session on `variant` with `original` through the tokens (`check`), then
/// edits `session` into `variant` and compares it with that fresh session exactly. Returns the
/// session, now holding `variant`, and the version it is at.
#[allow(clippy::too_many_arguments)]
pub fn compare(
    cx: &Context<'_>,
    check: &'static str,
    variant: &str,
    session: &mut Session,
    version: &mut i32,
    findings: &mut Vec<Finding>,
    loose: bool,
) -> bool {
    let (Some(original), Some(changed)) =
        (Layout::new(cx.text, cx.uri), Layout::new(variant, cx.uri))
    else {
        return false;
    };
    if !original.same_tokens(&changed) || cx.text == variant {
        return false;
    }
    let probes = original.identifiers(cx.probes);
    let Some(fresh) = (cx.fresh)(variant) else {
        findings.push(Finding::new(
            check,
            Position::default(),
            "",
            "could not open the variant".into(),
        ));
        return false;
    };
    let before = view(session, &original, &probes, loose);
    let after = view(&fresh, &changed, &probes, loose);
    if let Some((at, name, what)) = difference(&before, &after, &original) {
        findings.push(Finding::new(check, at, &name, what));
    }
    // The same text reached by editing: a few versions, a request after each.
    let edits = gap_edits(&original, &changed);
    let mut current = cx.text.to_owned();
    let batches = 4usize;
    let per = edits.len().div_ceil(batches).max(1);
    let mut chunks: Vec<&[(usize, usize, String)]> = edits.chunks(per).collect();
    // Later gaps first, so earlier byte offsets stay valid.
    chunks.reverse();
    for chunk in chunks {
        let mut changes = Vec::new();
        for (start, end, new) in chunk.iter().rev() {
            let range = Range {
                start: pos_of(&current, *start),
                end: pos_of(&current, *end),
            };
            current.replace_range(*start..*end, new);
            changes.push(TextChange {
                range: Some(range),
                range_length: None,
                text: new.clone(),
            });
        }
        *version += 1;
        if let Err(e) = session.change(cx.uri, *version, &changes) {
            findings.push(Finding::new(
                "c-edit",
                Position::default(),
                check,
                format!("change refused: {e:?}"),
            ));
            return false;
        }
        let _ = session.diagnostics(cx.uri);
        let _ = session.hover(
            cx.uri,
            pos_of(&current, current.floor_char_boundary(current.len() / 2)),
        );
    }
    if current != variant {
        findings.push(Finding::new(
            "c-edit",
            Position::default(),
            check,
            "the edits did not reproduce the text".into(),
        ));
        return false;
    }
    let edited = view(session, &changed, &probes, false);
    let reference = view(&fresh, &changed, &probes, false);
    if let Some((at, name, what)) = difference(&reference, &edited, &changed) {
        findings.push(Finding::new(
            "c-edit",
            at,
            &name,
            format!("after {check}: {what}"),
        ));
    }
    true
}

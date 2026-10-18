//! Queries and snapshots: what a session answers, and how two sets of answers differ.

use super::*;

/// A request made to a session, kept so that a failing run can be replayed with the same
/// requests between the same edits (what the session cached depends on them).
#[derive(Clone, Debug)]
pub enum Query {
    Diagnostics(DocumentUri),
    Tokens(DocumentUri),
    Hover(DocumentUri, Position),
    Definition(DocumentUri, Position),
    Completion(DocumentUri, Position),
    SignatureHelp(DocumentUri, Position),
}

impl Query {
    /// Make the request and drop the answer.
    pub fn execute(&self, session: &Session) {
        match self {
            Self::Diagnostics(u) => drop(session.diagnostics(u)),
            Self::Tokens(u) => drop(session.semantic_tokens(u)),
            Self::Hover(u, p) => drop(session.hover(u, *p)),
            Self::Definition(u, p) => drop(session.definition(u, *p)),
            Self::Completion(u, p) => drop(session.completion(u, *p)),
            Self::SignatureHelp(u, p) => drop(session.signature_help(u, *p)),
        }
    }
}

// Snapshots.

pub(super) type Answer<T> = Result<T, String>;

pub(super) struct Probe {
    pub(super) byte: usize,
    pub(super) position: Position,
    pub(super) definition: Answer<Vec<Location>>,
    pub(super) hover: Answer<Option<Hover>>,
}

pub(super) struct Snapshot {
    pub(super) diagnostics: Answer<Vec<Diagnostic>>,
    pub(super) tokens: Answer<Vec<SemanticToken>>,
    pub(super) probes: Vec<Probe>,
}

pub(super) fn answer<T>(result: Result<T, Error>) -> Answer<T> {
    result.map_err(|e| format!("{e:?}"))
}

pub(super) fn sorted_answer<T: std::fmt::Debug>(a: &Answer<Vec<T>>) -> Answer<Vec<String>> {
    a.as_ref()
        .map(|items| sorted_debug(items))
        .map_err(Clone::clone)
}

pub(super) fn sorted_debug<T: std::fmt::Debug>(items: &[T]) -> Vec<String> {
    let mut keys: Vec<_> = items.iter().map(|i| format!("{i:?}")).collect();
    keys.sort();
    keys
}

/// Identifier offsets, thinned to at most `cap` by a fixed stride.
pub(super) fn sample_identifiers(text: &str, cap: usize) -> Vec<usize> {
    let skipped: Vec<(usize, usize)> = here_strings(text)
        .into_iter()
        .chain(conditional_blocks(text))
        .collect();
    let all: Vec<usize> = identifier_starts(text)
        .into_iter()
        .filter(|&i| !skipped.iter().any(|&(a, b)| a <= i && i < b))
        .collect();
    let stride = all.len().div_ceil(cap.max(1)).max(1);
    all.into_iter().step_by(stride).collect()
}

/// Byte ranges of `#if` blocks with their `else` branches. Which branch is live depends on the
/// target, and code another target would check is answered from a compile for that target, so
/// the answer there is not the one from the compile the checker compares against.
pub(super) fn conditional_blocks(text: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut offset = 0;
    let mut open: Option<(usize, usize)> = None;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        match open {
            None if trimmed.starts_with("#if ") => {
                if trimmed.trim_end().ends_with('{') {
                    open = Some((offset, indent));
                } else {
                    out.push((offset, offset + line.len()));
                }
            }
            Some((start, at))
                if indent == at
                    && trimmed.starts_with('}')
                    && !trimmed.starts_with("} else")
                    && !trimmed.starts_with("}else") =>
            {
                out.push((start, offset + line.len()));
                open = None;
            }
            _ => {}
        }
        offset += line.len();
    }
    if let Some((start, _)) = open {
        out.push((start, text.len()));
    }
    out
}

pub(super) fn probe_at(
    clock: &mut Clock,
    session: &Session,
    uri: &DocumentUri,
    text: &str,
    byte: usize,
) -> Probe {
    let position = pos_of(text, byte);
    let definition = clock.run(Query::Definition(uri.clone(), position), || {
        answer(session.definition(uri, position))
    });
    let hover = clock.run(Query::Hover(uri.clone(), position), || {
        answer(session.hover(uri, position))
    });
    Probe {
        byte,
        position,
        definition,
        hover,
    }
}

impl Snapshot {
    pub(super) fn take(
        clock: &mut Clock,
        session: &Session,
        uri: &DocumentUri,
        text: &str,
        cap: usize,
    ) -> Self {
        let diagnostics = clock.run(Query::Diagnostics(uri.clone()), || {
            answer(session.diagnostics(uri))
        });
        let tokens = clock.run(Query::Tokens(uri.clone()), || {
            answer(session.semantic_tokens(uri))
        });
        let probes = sample_identifiers(text, cap)
            .into_iter()
            .map(|byte| probe_at(clock, session, uri, text, byte))
            .collect();
        Self {
            diagnostics,
            tokens,
            probes,
        }
    }
}

/// The first difference between two snapshots of the same text, if any.
pub(super) fn snapshot_difference(a: &Snapshot, b: &Snapshot) -> Option<String> {
    if sorted_answer(&a.diagnostics) != sorted_answer(&b.diagnostics) {
        return Some(format!(
            "diagnostics differ:\n  was {:?}\n  now {:?}",
            a.diagnostics, b.diagnostics
        ));
    }
    if sorted_answer(&a.tokens) != sorted_answer(&b.tokens) {
        let (x, y) = (sorted_answer(&a.tokens), sorted_answer(&b.tokens));
        let detail = match (x, y) {
            (Ok(x), Ok(y)) => {
                let before = x.iter().filter(|t| !y.contains(t)).take(3);
                let after = y.iter().filter(|t| !x.contains(t)).take(3);
                format!(
                    "\n  only in the first: {:?}\n  only in the second: {:?}",
                    before.collect::<Vec<_>>(),
                    after.collect::<Vec<_>>()
                )
            }
            _ => String::new(),
        };
        return Some(format!("semantic tokens differ ({detail})"));
    }
    if a.probes.len() != b.probes.len() {
        return Some("different number of probes".into());
    }
    for (x, y) in a.probes.iter().zip(&b.probes) {
        if x.position != y.position {
            return Some(format!(
                "probe positions differ {:?} {:?}",
                x.position, y.position
            ));
        }
        if x.definition != y.definition {
            return Some(format!(
                "definition at {:?} differs:\n  was {:?}\n  now {:?}",
                x.position, x.definition, y.definition
            ));
        }
        if x.hover != y.hover {
            return Some(format!(
                "hover at {:?} differs:\n  was {:?}\n  now {:?}",
                x.position, x.hover, y.hover
            ));
        }
    }
    None
}

/// A diagnostic that follows from what a body does, not from where it is written. A lint about
/// what is used is raised by removing the only use of an import elsewhere, and a compile-time
/// call fails where it is made when the procedure it runs stops returning a value.
pub(super) fn usage_lint(d: &Diagnostic) -> bool {
    matches!(d.code, DiagnosticCode::Lint(name) if name.starts_with("unused"))
        || d.message.starts_with("error during compile-time execution")
}

/// Whether the session reports an incomplete lexical edit: a literal or comment that stopped
/// closing is left out of its analysis, to keep the rest of the file's features while it is being
/// typed. A fresh session lexes the whole text as it is, so the two legitimately differ then.
pub(super) fn omits_literal(incremental: &Snapshot) -> bool {
    let Ok(now) = &incremental.diagnostics else {
        return false;
    };
    now.iter()
        .any(|d| d.message.starts_with("Incomplete lexical edit"))
}

/// Oracle g: text that lexes without error has nothing left out.
pub(super) fn stale_damage(incremental: &Snapshot, text: &str) -> bool {
    omits_literal(incremental) && jaic::lexer::lexical_regions(FileId(0), text).is_ok()
}

/// Oracle e on one document's diagnostics.
pub(super) fn sanity(text: &str, diagnostics: &[Diagnostic]) -> Option<String> {
    let lines: Vec<&str> = text.split('\n').collect();
    let valid = |p: Position| {
        let Some(line) = lines.get(p.line as usize) else {
            return false;
        };
        // A character offset must fall between UTF-16 code points of the line.
        let mut units = 0;
        for ch in line.chars() {
            if units == p.character {
                return true;
            }
            units += ch.len_utf16() as u32;
        }
        units >= p.character
    };
    let mut seen = Vec::new();
    for d in diagnostics {
        if !valid(d.range.start) || !valid(d.range.end) || d.range.start > d.range.end {
            return Some(format!("diagnostic range outside the document: {d:?}"));
        }
        let key = format!("{d:?}");
        if seen.contains(&key) {
            return Some(format!("duplicate diagnostic: {d:?}"));
        }
        seen.push(key);
    }
    None
}

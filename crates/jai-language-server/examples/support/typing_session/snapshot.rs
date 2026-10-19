//! Queries and snapshots: what a session answers, and how two sets of answers differ.

use super::{
    Clock, Diagnostic, DiagnosticCode, DocumentUri, Error, FileId, Hover, Location, Position,
    SemanticToken, Session, here_strings, identifier_starts, pos_of,
};

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
    let skipped: Vec<(usize, usize)> = here_strings(text);
    // A `#run` shows the value its call computed, which follows from what another body does.
    let is_run = |i: usize| {
        text[..i].ends_with('#')
            && text[i..]
                .strip_prefix("run")
                .is_some_and(|rest| !rest.starts_with(|c: char| c == '_' || c.is_alphanumeric()))
    };
    let all: Vec<usize> = identifier_starts(text)
        .into_iter()
        .filter(|&i| !skipped.iter().any(|&(a, b)| a <= i && i < b) && !is_run(i))
        .collect();
    let stride = all.len().div_ceil(cap.max(1)).max(1);
    all.into_iter().step_by(stride).collect()
}

/// `byte` is inside a polymorphic struct (`struct(T: Type) { ... }`): the types its hovers show
/// are those of whichever instance the editor has, so only their presence is compared.
pub(super) fn in_poly_struct(text: &str, byte: usize) -> bool {
    let bytes = text.as_bytes();
    let mut from = 0;
    while let Some(found) = text[from..].find("struct(") {
        let start = from + found;
        from = start + 1;
        if start > byte {
            return false;
        }
        let mut depth = 0i32;
        let mut seen_body = false;
        let mut end = bytes.len();
        for (i, &b) in bytes.iter().enumerate().skip(start + "struct".len()) {
            match b {
                b'(' | b'[' => depth += 1,
                b')' | b']' => depth -= 1,
                b'{' => {
                    depth += 1;
                    seen_body = true;
                }
                b'}' => {
                    depth -= 1;
                    if seen_body && depth <= 0 {
                        end = i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        if byte < end {
            return true;
        }
    }
    false
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

//! Checks (a), (b), (d) and (e): what the server reports on a file the compiler accepts, against what
//! the compiler resolved.
use jai_language_server::{
    DiagnosticCode, DiagnosticSeverity, DocumentUri, Hover, Position, Session,
};
use serde::Serialize;

use super::oracle::{Oracle, Resolved};
use super::{Finding, pos_of};

/// A region the server fades as unreachable, with where a statement could be put to see whether
/// it runs after all (`None`: the region does not start at a statement).
#[derive(Serialize)]
pub struct Dead {
    pub start: usize,
    pub end: usize,
    pub line: u32,
    pub probe: Option<usize>,
    pub text: String,
}

/// Hover text that is only the syntax layer's fallback, which the compile has nothing behind.
pub fn real_hover(hover: Option<&Hover>) -> bool {
    hover.is_some_and(|h| !h.contents.value.contains("Source syntax declaration"))
}

fn skip_space_and_comments_back(text: &str, mut at: usize) -> usize {
    let bytes = text.as_bytes();

    loop {
        while at > 0 && bytes[at - 1].is_ascii_whitespace() {
            at -= 1;
        }

        // A line comment ending just before: back to its `//`.
        let line_start = text[..at].rfind('\n').map_or(0, |i| i + 1);

        match text[line_start..at].find("//") {
            Some(i) if !text[line_start..line_start + i].contains('"') => at = line_start + i,
            _ => return at,
        }
    }
}

/// Where a marker statement can go in `text[start..end]` so that it runs exactly when the region
/// does: at its start when that starts a statement, inside a leading `{`, or after a `case`
/// label.
fn probe_point(text: &str, start: usize, end: usize) -> Option<usize> {
    let region = &text[start..end];
    let trimmed = region.trim_start();
    let at = start + (region.len() - trimmed.len());

    if trimmed.starts_with('{') {
        return Some(at + 1);
    }

    if trimmed.starts_with("case")
        && !trimmed[4..].starts_with(|c: char| c.is_alphanumeric() || c == '_')
    {
        return trimmed.find(';').map(|i| at + i + 1).filter(|&p| p <= end);
    }

    if trimmed.starts_with('#') || trimmed.starts_with("else") {
        return None;
    }

    let before = skip_space_and_comments_back(text, at);

    matches!(
        text.as_bytes().get(before.wrapping_sub(1)),
        Some(b';' | b'{' | b'}')
    )
    .then_some(at)
}

/// (a): no error diagnostics, and every unreachable region listed for the run check.
pub fn diagnostics(
    session: &Session,
    uri: &DocumentUri,
    text: &str,
    findings: &mut Vec<Finding>,
    dead: &mut Vec<Dead>,
) {
    let diagnostics = match session.diagnostics(uri) {
        Ok(d) => d,
        Err(e) => {
            findings.push(Finding::new(
                "a-request",
                Position::default(),
                "",
                format!("{e:?}"),
            ));
            return;
        }
    };

    for d in diagnostics {
        if d.code == DiagnosticCode::Unreachable {
            let (Some(start), Some(end)) = (
                super::byte_of(text, d.range.start),
                super::byte_of(text, d.range.end),
            ) else {
                continue;
            };
            let shown: String = text[start..end].chars().take(120).collect();

            dead.push(Dead {
                start,
                end,
                line: d.range.start.line + 1,
                probe: probe_point(text, start, end),
                text: shown,
            });
        } else if d.severity == DiagnosticSeverity::Error || d.code == DiagnosticCode::Limit {
            findings.push(Finding::new(
                "a-error",
                d.range.start,
                "",
                format!("{:?}: {}", d.code, d.message),
            ));
        }
    }
}

fn evenly<T>(items: &[T], cap: usize) -> impl Iterator<Item = &T> {
    let step = items.len().div_ceil(cap.max(1)).max(1);
    items.iter().step_by(step)
}

/// (b): each identifier the compiler resolved has a hover and, where it is declared in a file,
/// a definition there.
pub fn resolutions(
    session: &Session,
    uri: &DocumentUri,
    text: &str,
    oracle: &Oracle,
    cap: usize,
    findings: &mut Vec<Finding>,
) -> usize {
    let mut probed = 0;

    for r in evenly(&oracle.refs, cap) {
        probed += 1;
        let name = &text[r.start..r.end];
        let at = pos_of(text, r.start);
        let hover = session.hover(uri, at);

        match &hover {
            Ok(h) if real_hover(h.as_ref()) => {}
            Ok(_) => findings.push(Finding::new(
                "b-hover",
                at,
                name,
                format!("{:?}", r.resolved),
            )),
            Err(e) => findings.push(Finding::new("b-hover", at, name, format!("{e:?}"))),
        }

        let wanted = match &r.resolved {
            Resolved::Entity(places) => !places.is_empty(),
            Resolved::Member(declared) | Resolved::Type(declared) => *declared,
            Resolved::Procs | Resolved::Module | Resolved::Baked => true,
        };

        if !wanted {
            continue;
        }

        let locations = match session.definition(uri, at) {
            Ok(l) => l,
            Err(e) => {
                findings.push(Finding::new("b-definition", at, name, format!("{e:?}")));
                continue;
            }
        };

        if locations.is_empty() {
            findings.push(Finding::new(
                "b-definition",
                at,
                name,
                format!("{:?}", r.resolved),
            ));
            continue;
        }

        if let Resolved::Entity(places) = &r.resolved
            && !places.is_empty()
        {
            let hit = places.iter().any(|(path, first, last)| {
                let want = path.canonicalize().unwrap_or_else(|_| path.clone());

                locations.iter().any(|l| {
                    DocumentUri::parse(&l.uri).is_ok_and(|u| {
                        let got = std::path::Path::new(u.path());

                        got.canonicalize().unwrap_or_else(|_| got.to_path_buf()) == want
                            && (*first..=*last).contains(&l.range.start.line)
                    })
                })
            });

            if !hit {
                let got: Vec<String> = locations
                    .iter()
                    .map(|l| format!("{}:{}", l.uri, l.range.start.line + 1))
                    .collect();

                findings.push(Finding::new(
                    "b-definition-target",
                    at,
                    name,
                    format!(
                        "compiler: {}; server: {}",
                        places
                            .iter()
                            .map(|(path, first, _)| format!("{}:{}", path.display(), first + 1))
                            .collect::<Vec<_>>()
                            .join(", "),
                        got.join(", ")
                    ),
                ));
            }
        }
    }

    probed
}

/// (d): completion where the code already has an answer. At a member name after `.`, the list
/// holds that member; just inside a struct literal's `.{`, it holds the fields not yet named.
pub fn completions(
    session: &Session,
    uri: &DocumentUri,
    text: &str,
    oracle: &Oracle,
    cap: usize,
    findings: &mut Vec<Finding>,
) -> usize {
    let bytes = text.as_bytes();
    let members: Vec<_> = oracle
        .refs
        .iter()
        .filter(|r| r.start >= 2 && bytes[r.start - 1] == b'.' && bytes[r.start - 2] != b'.')
        // `.TAG,, field: T;` declares a tagged union's variant; nothing is completed there.
        .filter(|r| !text[r.end..].trim_start().starts_with(",,"))
        .collect();
    let mut probed = 0;

    for r in evenly(&members, cap) {
        probed += 1;
        let name = &text[r.start..r.end];
        let at = pos_of(text, r.start);

        match session.completion(uri, at) {
            // A list cut short (a module with thousands of names) is filtered as the user types.
            Ok(list) if list.is_incomplete || list.items.iter().any(|i| i.label == name) => {}
            Ok(list) => findings.push(Finding::new(
                "d-member",
                at,
                name,
                format!("{} items, {:?}", list.items.len(), r.resolved),
            )),
            Err(e) => findings.push(Finding::new("d-member", at, name, format!("{e:?}"))),
        }
    }

    for literal in evenly(&oracle.literals, cap) {
        probed += 1;
        let at = pos_of(text, literal.at);

        match session.completion(uri, at) {
            Ok(list) => {
                let missing: Vec<&str> = literal
                    .fields
                    .iter()
                    .filter(|f| !list.items.iter().any(|i| &i.label == *f))
                    .map(String::as_str)
                    .collect();

                if !missing.is_empty() {
                    findings.push(Finding::new(
                        "d-literal",
                        at,
                        &literal.type_name,
                        format!("missing fields {}", missing.join(", ")),
                    ));
                }
            }
            Err(e) => findings.push(Finding::new(
                "d-literal",
                at,
                &literal.type_name,
                format!("{e:?}"),
            )),
        }
    }

    probed
}

/// (e): go to definition on a note the compile-time code read lists each place that read it.
pub fn notes(
    session: &Session,
    uri: &DocumentUri,
    text: &str,
    oracle: &Oracle,
    findings: &mut Vec<Finding>,
) {
    let canonical = |p: &std::path::Path| p.canonicalize().unwrap_or_else(|_| p.to_path_buf());

    for note in &oracle.notes {
        let at = pos_of(text, note.at);
        let name: String = text[note.at..]
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        let locations = match session.definition(uri, at) {
            Ok(l) => l,
            Err(e) => {
                findings.push(Finding::new("e-note", at, &name, format!("{e:?}")));
                continue;
            }
        };
        let got: Vec<(std::path::PathBuf, u32)> = locations
            .iter()
            .filter_map(|l| {
                let u = DocumentUri::parse(&l.uri).ok()?;

                Some((
                    canonical(std::path::Path::new(u.path())),
                    l.range.start.line,
                ))
            })
            .collect();

        for (path, line) in &note.readers {
            if !got.contains(&(canonical(path), *line)) {
                findings.push(Finding::new(
                    "e-note-reader",
                    at,
                    &name,
                    format!("compiler: {}:{}", path.display(), line + 1),
                ));
            }
        }
    }
}

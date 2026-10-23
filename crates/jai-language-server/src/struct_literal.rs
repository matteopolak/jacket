//! Field names inside a struct literal are another completion source: `Foo.{ a = 1, | }` and
//! `.{ | }` (or `x: Foo = { | }`) offer the fields of the literal's type that are not assigned yet.
use crate::{CompletionItem, CompletionKind, DocumentUri, Session, analysis::Span};
use jaic::lexer::{Lexed, P, Tok};
use std::collections::BTreeSet;

/// The field-name position the cursor is at.
struct Site {
    /// The word at the cursor, empty when none is typed.
    word: std::ops::Range<usize>,
    /// The word is followed by `=` already.
    has_equals: bool,
    /// Field names the literal assigns elsewhere.
    assigned: BTreeSet<String>,
    /// Nothing comes before the cursor in the literal, so a positional value is possible too.
    first: bool,
}

fn is_open(p: P) -> bool {
    matches!(
        p,
        P::LParen | P::LBrace | P::LBracket | P::DotBrace | P::DotBracket
    )
}

fn is_close(p: P) -> bool {
    matches!(p, P::RParen | P::RBrace | P::RBracket)
}

/// Where the cursor at `byte` names a field of a struct literal, from the tokens alone.
fn site(text: &str, lexed: &Lexed, byte: usize) -> Option<Site> {
    let tokens = &lexed.tokens;
    let ident = jaic::lexicon::is_identifier_char;
    let start = text[..byte]
        .char_indices()
        .rfind(|&(_, c)| !ident(c))
        .map_or(0, |(i, c)| i + c.len_utf8());
    let end = byte
        + text[byte..]
            .find(|c: char| !ident(c))
            .unwrap_or(text.len() - byte);
    // The tokens before the word, and the first one after it.
    let before = tokens.partition_point(|t| t.span.end <= start as u32);
    let after = tokens.partition_point(|t| t.span.start < end as u32);
    let prev = before.checked_sub(1)?;
    if !matches!(
        tokens[prev].tok,
        Tok::Punct(P::Comma | P::DotBrace | P::LBrace)
    ) {
        return None;
    }
    // The literal's opening token, and the arguments before the cursor.
    let mut depth = 0usize;
    let mut open = None;
    for k in (0..before).rev() {
        match tokens[k].tok {
            Tok::Punct(p) if is_close(p) => depth += 1,
            Tok::Punct(p) if is_open(p) && depth > 0 => depth -= 1,
            Tok::Punct(P::DotBrace) => {
                open = Some(k);
                break;
            }
            // `x: Foo = { | }`: braces right after `=` are a literal typed by what is assigned.
            Tok::Punct(P::LBrace) if k > 0 && tokens[k - 1].tok == Tok::Punct(P::Eq) => {
                open = Some(k);
                break;
            }
            Tok::Punct(p) if is_open(p) => return None,
            _ => {}
        }
    }
    let open = open?;
    // The literal's arguments, as ranges of tokens.
    let mut args: Vec<std::ops::Range<usize>> = Vec::new();
    let mut depth = 0usize;
    let mut from = open + 1;
    for k in open + 1..=tokens.len() {
        match tokens.get(k).map(|t| &t.tok) {
            Some(Tok::Punct(p)) if is_open(*p) => depth += 1,
            Some(Tok::Punct(p)) if is_close(*p) && depth > 0 => depth -= 1,
            Some(Tok::Punct(P::Comma)) if depth == 0 => {
                args.push(from..k);
                from = k + 1;
            }
            Some(Tok::Punct(p)) if is_close(*p) => {
                args.push(from..k);
                break;
            }
            None => args.push(from..k),
            _ => {}
        }
    }
    let name_of = |range: &std::ops::Range<usize>| match tokens.get(range.start..range.start + 2) {
        Some([a, b]) if range.len() >= 2 && b.tok == Tok::Punct(P::Eq) => match &a.tok {
            Tok::Ident(n) => Some(n.as_str().to_owned()),
            _ => None,
        },
        _ => None,
    };
    // The argument holding the cursor: one per comma between the opening token and the word.
    let current = tokens[open + 1..before]
        .iter()
        .scan(0usize, |depth, t| {
            Some(match t.tok {
                Tok::Punct(p) if is_open(p) => {
                    *depth += 1;
                    false
                }
                Tok::Punct(p) if is_close(p) => {
                    *depth -= 1;
                    false
                }
                Tok::Punct(P::Comma) => *depth == 0,
                _ => false,
            })
        })
        .filter(|&comma| comma)
        .count();
    // `base.x = v` and `xs[1] = w` assign a field by a path; they are not positional values.
    let is_target = |range: &std::ops::Range<usize>| {
        let mut depth = 0usize;
        for (k, t) in tokens[range.clone()].iter().enumerate() {
            match t.tok {
                Tok::Punct(P::LBracket) => depth += 1,
                Tok::Punct(P::RBracket) if depth > 0 => depth -= 1,
                _ if depth > 0 => {}
                Tok::Ident(_) | Tok::Punct(P::Dot) => {}
                Tok::Punct(P::Eq) => return k > 0,
                _ => return false,
            }
        }
        false
    };
    // A positional value before the cursor: the values here are expressions too.
    if args[..current.min(args.len())]
        .iter()
        .any(|a| name_of(a).is_none() && !is_target(a))
    {
        return None;
    }
    let assigned = args
        .iter()
        .enumerate()
        .filter(|&(i, _)| i != current)
        .filter_map(|(_, a)| name_of(a))
        .collect();
    let has_equals = matches!(tokens.get(after), Some(t) if t.tok == Tok::Punct(P::Eq));
    Some(Site {
        word: start..end,
        has_equals,
        assigned,
        first: current == 0,
    })
}

const PLACEHOLDER: &str = "__jailsp_field";

impl Session {
    /// The unassigned fields of the struct literal around `byte`, matching the typed prefix, and
    /// whether the cursor may also hold a positional value.
    pub(crate) fn struct_literal_completions(
        &self,
        uri: &DocumentUri,
        byte: usize,
    ) -> Option<(Vec<CompletionItem>, bool)> {
        let doc = self.document(uri).ok()?;
        let lexed = Lexed::new(jaic::source::FileId(0), &doc.text);
        let site = site(&doc.text, &lexed, byte)?;
        let text = &doc.text;
        let prefix = text[site.word.start..byte].to_lowercase();
        let candidate = format!(
            "{}{PLACEHOLDER}{}{}",
            &text[..site.word.start],
            if site.has_equals {
                ""
            } else {
                " = 0"
            },
            &text[site.word.end..]
        );
        let fields = self.with_semantic(uri, &candidate, |a, path| {
            a.complete_literal_fields(path, site.word.start)
        })?;
        let range = doc
            .index
            .range(text, Span::new(site.word.start, site.word.end))
            .ok()?;
        // Fields keep their declaration order, and the first one left is selected.
        let mut items = fields
            .into_iter()
            .enumerate()
            .filter(|(_, f)| {
                f.name.to_lowercase().starts_with(&prefix) && !site.assigned.contains(&f.name)
            })
            .map(|(index, f)| CompletionItem {
                insert_text: Some(if site.has_equals {
                    f.name.clone()
                } else {
                    format!("{} = ", f.name)
                }),
                sort_text: Some(format!("!{index:05}")),
                label: f.name,
                kind: CompletionKind::Field,
                detail: f.detail,
                replace: Some(range),
                ..CompletionItem::default()
            })
            .take(self.limits.symbols)
            .collect::<Vec<_>>();
        if let Some(first) = items.first_mut() {
            first.preselect = true;
        }
        Some((items, site.first))
    }
}

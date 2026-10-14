//! Incremental recovery uses verified lexical boundaries, never old ASTs or types.
use crate::analysis::Span;
use jaic::source::FileId;

pub(crate) fn mask(text: &str, damage: &[Span]) -> Option<String> {
    if damage.is_empty() {
        return None;
    }
    let mut bytes = text.as_bytes().to_vec();
    for span in damage {
        for byte in &mut bytes[span.start..span.end] {
            if !matches!(*byte, b'\n' | b'\r') {
                *byte = b' ';
            }
        }
    }
    Some(String::from_utf8(bytes).expect("whole lexical regions lie on UTF-8 boundaries"))
}

/// Map old verified opaque regions through the actual changed bytes. If an edited
/// region no longer has the same lexical boundaries, omit just that region until
/// repaired. This cannot expose a declaration that used to be inside a comment.
pub(crate) fn update(old: &str, new: &str, previous: &[Span]) -> Vec<Span> {
    if jaic::parser::parse_file(FileId(0), new).is_ok() {
        return Vec::new();
    }
    let old_view = mask(old, previous);
    let mut regions: Vec<Span> =
        jaic::lexer::lexical_regions(FileId(0), old_view.as_deref().unwrap_or(old))
            .unwrap_or_default()
            .into_iter()
            .map(|s| Span::new(s.start as usize, s.end as usize))
            .collect();
    regions.extend_from_slice(previous);
    let mut start = old
        .bytes()
        .zip(new.bytes())
        .take_while(|(a, b)| a == b)
        .count();
    start = old.floor_char_boundary(start);
    let suffix = old[start..]
        .bytes()
        .rev()
        .zip(new[start..].bytes().rev())
        .take_while(|(a, b)| a == b)
        .count();
    let old_end = old.ceil_char_boundary(old.len() - suffix);
    let new_end = new.ceil_char_boundary(new.len() - suffix);
    let map_start = |byte: usize| {
        if byte <= start {
            byte
        } else if byte >= old_end {
            new_end + byte - old_end
        } else {
            start
        }
    };
    let map_end = |byte: usize| {
        if byte < start {
            byte
        } else if byte >= old_end {
            new_end + byte - old_end
        } else {
            new_end
        }
    };
    let current = jaic::lexer::lexical_regions(FileId(0), new).unwrap_or_default();
    let mut damage = Vec::new();
    for region in regions {
        let touched = region.start <= old_end && start <= region.end;
        let was_damaged = previous.contains(&region);
        if !touched && !was_damaged {
            continue;
        }
        let mapped = Span::new(map_start(region.start), map_end(region.end));
        if mapped.start >= mapped.end {
            continue;
        }
        if current
            .iter()
            .any(|s| s.start as usize == mapped.start && s.end as usize == mapped.end)
        {
            continue;
        }
        damage.push(mapped);
    }
    damage.sort_by_key(|s| (s.start, s.end));
    damage.dedup();
    damage
}

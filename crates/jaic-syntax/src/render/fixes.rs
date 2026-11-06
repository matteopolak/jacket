//! Edit-anchored fix previews with bounded neutral context.
use super::{CONTEXT_LINES, MAX_FIX_LINES};
use crate::source::LineIndex;

type Row = (String, char, String);
type Edit = (usize, usize, String);

struct Group<'a> {
    first: usize,
    last: usize,
    edits: Vec<&'a Edit>,
}

/// Show edits in original source order with one unchanged line around each hunk.
/// Neutral rows have a blank marker; `-` numbers refer to old lines, `+`/`~` to fixed
/// lines, and neutral numbers to old source lines. For example, deleting line 2 and
/// replacing old line 4 shows `2 - old`, neutral line 3, then `3 ~ replacement`.
/// Only edited line windows are copied; distant context gaps are elided.
pub fn changed_lines(text: &str, lines: &LineIndex, edits: &[Edit]) -> Vec<Row> {
    let mut sorted: Vec<_> = edits
        .iter()
        .filter(|(a, b, new)| text.get(*a..*b) != Some(new.as_str()))
        .collect();
    sorted.sort_by_key(|e| e.0);
    let mut groups: Vec<Group<'_>> = Vec::new();
    for edit in sorted {
        // Include complete neighbors before editing: deleting a boundary newline can
        // join the successor, and an inserted LF can complete a preceding CR.
        let first = lines.line_of(edit.0).saturating_sub(CONTEXT_LINES);
        let last = lines
            .line_of(edit.1.saturating_sub(1).max(edit.0))
            .saturating_add(CONTEXT_LINES)
            .min(lines.line_of(text.len().saturating_sub(1)));
        if let Some(group) = groups.last_mut()
            && first <= group.last.saturating_add(1)
        {
            group.last = group.last.max(last);
            group.edits.push(edit);
        } else {
            groups.push(Group {
                first,
                last,
                edits: vec![edit],
            });
        }
    }
    let mut out = Preview::default();
    let mut cursor = None;
    let mut shift = 0isize;
    for group in groups {
        let start = lines.line_range(group.first).unwrap().0;
        let end = lines.line_range(group.last + 1).map_or(text.len(), |r| r.0);
        let original = &text[start..end];
        let mut fixed = original.to_owned();
        for edit in group.edits.iter().rev() {
            fixed.replace_range(edit.0 - start..edit.1 - start, &edit.2);
        }
        if fixed == original {
            continue;
        }
        let old_index = LineIndex::new(original);
        let new_index = LineIndex::new(&fixed);
        // Full line bytes, including terminators, are the sole equality authority.
        // Complete neighboring windows make these real lines rather than cropped fragments.
        let old: Vec<_> = old_index
            .lines()
            .enumerate()
            .filter(|&(_, (a, _))| a < original.len())
            .map(|(n, _)| line_bytes(original, &old_index, n))
            .collect();
        let new: Vec<_> = new_index
            .lines()
            .enumerate()
            .filter(|&(_, (a, _))| a < fixed.len())
            .map(|(n, _)| line_bytes(&fixed, &new_index, n))
            .collect();
        let equal = |i: usize, j: usize| old[i] == new[j];
        let common = old.len().min(new.len());
        let prefix = (0..common).take_while(|&n| equal(n, n)).count();
        let suffix = (0..common - prefix)
            .take_while(|&n| equal(old.len() - n - 1, new.len() - n - 1))
            .count();
        let first = group.first + prefix;
        if let Some(from) = cursor {
            if first - from > CONTEXT_LINES * 2 {
                for n in from..from + CONTEXT_LINES {
                    context(text, lines, n, &mut out);
                }
                out.push(("...".into(), ' ', String::new()));
                for n in first - CONTEXT_LINES..first {
                    context(text, lines, n, &mut out);
                }
            } else {
                for n in from..first {
                    context(text, lines, n, &mut out);
                }
            }
        } else {
            for n in first.saturating_sub(CONTEXT_LINES)..first {
                context(text, lines, n, &mut out);
            }
        }
        let new_base = (first as isize + shift) as usize;
        cluster(
            &old[prefix..old.len() - suffix],
            &new[prefix..new.len() - suffix],
            first,
            new_base,
            &mut out,
        );
        shift +=
            new_index.line_of(fixed.len()) as isize - old_index.line_of(original.len()) as isize;
        cursor = Some(group.first + old.len() - suffix);
    }
    if let Some(from) = cursor {
        for n in from..from + CONTEXT_LINES {
            context(text, lines, n, &mut out);
        }
    }
    out.finish()
}

fn line_bytes<'a>(text: &'a str, index: &LineIndex, line: usize) -> &'a str {
    let start = index.line_range(line).unwrap().0;
    let end = index.line_range(line + 1).map_or(text.len(), |r| r.0);
    &text[start..end]
}

fn context(text: &str, lines: &LineIndex, n: usize, out: &mut Preview) {
    if let Some((a, b)) = lines.line_range(n)
        && a < text.len()
    {
        out.push(((n + 1).to_string(), ' ', text[a..b].trim_end().into()));
    }
}

fn cluster(old: &[&str], new: &[&str], old_base: usize, new_base: usize, out: &mut Preview) {
    let hunks = if old.len() == new.len() {
        vec![similar::DiffOp::Replace {
            old_index: 0,
            old_len: old.len(),
            new_index: 0,
            new_len: new.len(),
        }]
    } else {
        similar::capture_diff_slices(similar::Algorithm::Myers, old, new)
    };
    for hunk in hunks {
        let a = hunk.old_range();
        let b = hunk.new_range();
        if hunk.tag() == similar::DiffTag::Equal {
            for n in a {
                out.push((
                    (old_base + n + 1).to_string(),
                    ' ',
                    old[n].trim_end().into(),
                ));
            }
            continue;
        }
        // Pair replacement lines at their source position before extra deleted lines,
        // so deleting an increment does not move its preceding replacement below it.
        let paired = a.len().min(b.len());
        for offset in 0..paired {
            let (i, j) = (a.start + offset, b.start + offset);
            let mark = if old[i] == new[j] {
                ' '
            } else {
                '~'
            };
            let number = if mark == ' ' {
                old_base + i + 1
            } else {
                new_base + j + 1
            };
            out.push((number.to_string(), mark, new[j].trim_end().into()));
        }
        for (i, line) in old.iter().enumerate().take(a.end).skip(a.start + paired) {
            out.push(((old_base + i + 1).to_string(), '-', line.trim_end().into()));
        }
        for (j, line) in new.iter().enumerate().take(b.end).skip(b.start + paired) {
            out.push(((new_base + j + 1).to_string(), '+', line.trim_end().into()));
        }
    }
}

/// Keep only a bounded head/tail of neutral runs until their neighboring hunks are known.
#[derive(Default)]
struct Preview {
    rows: Vec<Row>,
    counts: [usize; 3],
    pending: Vec<Row>,
    pending_count: usize,
    changed: bool,
}

impl Preview {
    fn push(&mut self, row: Row) {
        if row.1 == ' ' {
            self.pending_count += 1;
            if self.pending.len() == CONTEXT_LINES * 2 {
                self.pending.remove(CONTEXT_LINES);
            }
            self.pending.push(row);
        } else {
            self.flush(false);
            self.emit(row);
            self.changed = true;
        }
    }

    fn flush(&mut self, trailing: bool) {
        let pending = std::mem::take(&mut self.pending);
        let count = std::mem::take(&mut self.pending_count);
        if !self.changed {
            let skip = pending.len().saturating_sub(CONTEXT_LINES);
            for row in pending.into_iter().skip(skip) {
                self.emit(row);
            }
        } else if trailing {
            for row in pending.into_iter().take(CONTEXT_LINES) {
                self.emit(row);
            }
        } else {
            for (i, row) in pending.into_iter().enumerate() {
                if count > CONTEXT_LINES * 2 && i == CONTEXT_LINES {
                    self.emit(("...".into(), ' ', String::new()));
                }
                self.emit(row);
            }
        }
    }

    fn finish(mut self) -> Vec<Row> {
        self.flush(true);
        self.rows
    }

    /// Removed and fixed rows retain separate limits; neutral context has the same bound.
    fn emit(&mut self, row: Row) {
        let bucket = match row.1 {
            '-' => 0,
            '+' | '~' => 1,
            _ => 2,
        };
        let count = &mut self.counts[bucket];
        if *count < MAX_FIX_LINES {
            self.rows.push(row);
        } else if *count == MAX_FIX_LINES {
            self.rows.push(("...".into(), row.1, String::new()));
        }
        *count += 1;
    }
}

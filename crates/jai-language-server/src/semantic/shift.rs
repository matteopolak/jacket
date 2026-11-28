//! Moving offsets between the text a compile saw and the text of the document now.

/// How one text became another: a few edited stretches (hunks), in order, with the text
/// between them unchanged. Offsets outside every hunk move by the change in length of the
/// hunks before them; offsets inside one have no counterpart.
///
/// ```text
/// old: aaa\nbbb\nccc\n     between(old, new) has two hunks, so `bbb`
/// new: aaaX\nbbb\ncc!c\n   still has a place in `new`: to_new(4) == Some(5)
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Shift {
    hunks: Vec<Hunk>,
}

/// One edited stretch: `old` in the old text became `new` in the new one (byte ranges).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Hunk {
    old: (usize, usize),
    new: (usize, usize),
}

impl Hunk {
    /// The hunk with old and new swapped.
    fn flipped(self) -> Self {
        Self {
            old: self.new,
            new: self.old,
        }
    }

    /// How much longer the new stretch is.
    fn growth(self) -> isize {
        (self.new.1 - self.new.0) as isize - (self.old.1 - self.old.0) as isize
    }
}

/// Edits farther apart than this many line edits are taken as one stretch: finding them all
/// costs time in proportion to the count, and that many pending edits are rare.
const MAX_LINE_EDITS: usize = 64;

impl Shift {
    /// The text is as it was.
    pub const fn none() -> Self {
        Self {
            hunks: Vec::new(),
        }
    }

    /// The edited stretches between `old` and `new`: the common prefix and suffix, and between
    /// them the lines both share (so two separate edits keep what lies between them), each
    /// remaining stretch narrowed to the bytes that differ.
    pub fn between(old: &[u8], new: &[u8]) -> Self {
        if old == new {
            return Self::none();
        }

        let (prefix, suffix) = common_ends(old, new);
        let (old_mid, new_mid) = ((prefix, old.len() - suffix), (prefix, new.len() - suffix));
        let old_lines = lines(old, old_mid);
        let new_lines = lines(new, new_mid);
        let hunks = match line_hunks((old, &old_lines, old_mid.1), (new, &new_lines, new_mid.1)) {
            Some(hunks) => hunks.into_iter().map(|h| narrow(old, new, h)).collect(),
            None => vec![Hunk {
                old: old_mid,
                new: new_mid,
            }],
        };

        Self {
            hunks,
        }
    }

    pub fn is_none(&self) -> bool {
        self.hunks.is_empty()
    }

    /// Where `new` (an offset in the current text) was in the old one, `None` inside an edit.
    pub fn to_old(&self, new: usize) -> Option<usize> {
        self.locate_old(new).ok()
    }

    /// Like `to_old`, an offset inside an edit becoming the start of it.
    pub fn to_old_or_edit(&self, new: usize) -> usize {
        self.locate_old(new).unwrap_or_else(|edit| edit)
    }

    /// `Ok` with where `new` was, else `Err` with where (in the old text) the edit holding it
    /// starts. An offset at the start of an edit is before it.
    fn locate_old(&self, new: usize) -> Result<usize, usize> {
        let mut growth = 0isize;

        for h in &self.hunks {
            if new <= h.new.0 {
                break;
            }

            if new < h.new.1 {
                return Err(h.old.0);
            }

            growth += h.growth();
        }

        Ok(new.wrapping_sub_signed(growth))
    }

    /// Where `old` (an offset in the old text) is now, `None` inside what was edited away.
    /// What starts where text was inserted moves with it.
    pub fn to_new(&self, old: usize) -> Option<usize> {
        let mut growth = 0isize;

        for h in &self.hunks {
            if old < h.old.0 {
                break;
            }

            if old < h.old.1 {
                return None;
            }

            growth += h.growth();
        }

        Some(old.wrapping_add_signed(growth))
    }

    /// Move a complete unchanged span. Its end stays before an insertion at that boundary;
    /// its start follows an insertion before the span. Spans crossing an edit are omitted.
    pub fn to_new_range(&self, start: usize, end: usize) -> Option<(usize, usize)> {
        move_range(self.hunks.iter().copied(), start, end)
    }

    /// The unchanged current span's coordinates in the checked text.
    pub fn to_old_range(&self, start: usize, end: usize) -> Option<(usize, usize)> {
        move_range(self.hunks.iter().map(|h| h.flipped()), start, end)
    }

    /// Like `to_new`, an offset inside an edit staying where it is.
    pub fn to_new_or_same(&self, old: usize) -> usize {
        self.to_new(old).unwrap_or(old)
    }
}

fn move_range(
    hunks: impl Iterator<Item = Hunk>,
    start: usize,
    end: usize,
) -> Option<(usize, usize)> {
    let mut growth = 0isize;

    for h in hunks {
        if end <= h.old.0 {
            break;
        }

        if start < h.old.1 {
            return None;
        }

        growth += h.growth();
    }

    Some((
        start.wrapping_add_signed(growth),
        end.wrapping_add_signed(growth),
    ))
}

/// The lengths of the longest common prefix of `old` and `new` and of the longest common
/// suffix of what follows it.
fn common_ends(old: &[u8], new: &[u8]) -> (usize, usize) {
    let prefix = old.iter().zip(new).take_while(|(a, b)| a == b).count();
    let room = old.len().min(new.len()) - prefix;
    let suffix = old
        .iter()
        .rev()
        .zip(new.iter().rev())
        .take(room)
        .take_while(|(a, b)| a == b)
        .count();

    (prefix, suffix)
}

/// A line as its byte range, newline included, and a hash of its bytes.
type Line = (usize, usize, u64);

/// The lines of `text[range]`, each with its newline, as byte ranges with a hash of their
/// bytes.
fn lines(text: &[u8], (start, end): (usize, usize)) -> Vec<Line> {
    use std::hash::Hasher;
    let mut out = Vec::new();
    let mut at = start;

    while at < end {
        let stop = text[at..end]
            .iter()
            .position(|&b| b == b'\n')
            .map_or(end, |n| at + n + 1);
        let mut h = jaic::fxhash::FxHasher::default();

        h.write(&text[at..stop]);
        out.push((at, stop, h.finish()));
        at = stop;
    }

    out
}

/// The stretches of lines that differ between `old_lines` and `new_lines` (lines of `old` and
/// `new` that end at `old_end` and `new_end`), by a shortest edit script (Myers' algorithm). `None` when that takes more than
/// `MAX_LINE_EDITS` line edits.
fn line_hunks(
    (old, old_lines, old_end): (&[u8], &[Line], usize),
    (new, new_lines, new_end): (&[u8], &[Line], usize),
) -> Option<Vec<Hunk>> {
    let (n, m) = (old_lines.len() as isize, new_lines.len() as isize);
    let same = |x: isize, y: isize| {
        let (a, b) = (old_lines[x as usize], new_lines[y as usize]);
        a.2 == b.2 && old[a.0..a.1] == new[b.0..b.1]
    };
    let offset = MAX_LINE_EDITS as isize;
    let at = |k: isize| (k + offset + 1) as usize;
    // `v[k]`: the furthest `x` reached on diagonal `k = x - y`; `trace[d]` is `v` before step d.
    let mut v = vec![0isize; 2 * MAX_LINE_EDITS + 3];
    let mut trace: Vec<Vec<isize>> = Vec::new();
    let mut found = None;

    'search: for d in 0..=MAX_LINE_EDITS as isize {
        trace.push(v.clone());
        let mut k = -d;

        while k <= d {
            let mut x = if k == -d || (k != d && v[at(k - 1)] < v[at(k + 1)]) {
                v[at(k + 1)]
            } else {
                v[at(k - 1)] + 1
            };
            let mut y = x - k;

            while x < n && y < m && same(x, y) {
                x += 1;
                y += 1;
            }

            v[at(k)] = x;

            if x >= n && y >= m {
                found = Some(d);
                break 'search;
            }

            k += 2;
        }
    }

    let d_end = found?;
    // Walk back from the end, collecting the matched line pairs of each step's snake.
    let mut matched = Vec::new();
    let (mut x, mut y) = (n, m);

    for d in (1..=d_end).rev() {
        let v = &trace[d as usize];
        let k = x - y;
        let down = k == -d || (k != d && v[at(k - 1)] < v[at(k + 1)]);
        let prev_k = if down {
            k + 1
        } else {
            k - 1
        };
        let (prev_x, prev_y) = (v[at(prev_k)], v[at(prev_k)] - prev_k);
        // The step itself: a line added (down) or removed (right), then the snake.
        let snake_x = if down {
            prev_x
        } else {
            prev_x + 1
        };

        while x > snake_x {
            x -= 1;
            y -= 1;
            matched.push((x, y));
        }

        (x, y) = (prev_x, prev_y);
    }

    while x > 0 && y > 0 {
        x -= 1;
        y -= 1;
        matched.push((x, y));
    }

    matched.reverse();
    // The gaps between matched lines are the hunks.
    let mut hunks = Vec::new();
    let (mut ox, mut ny) = (0isize, 0isize);

    for (mx, my) in matched.into_iter().chain(std::iter::once((n, m))) {
        if mx > ox || my > ny {
            hunks.push(Hunk {
                old: (
                    line_start(old_lines, ox, old_end),
                    line_start(old_lines, mx, old_end),
                ),
                new: (
                    line_start(new_lines, ny, new_end),
                    line_start(new_lines, my, new_end),
                ),
            });
        }

        (ox, ny) = (mx + 1, my + 1);
    }

    Some(hunks)
}

/// Where line `i` of `lines` starts, or `end` (where they end) when `i` is past them.
fn line_start(lines: &[Line], i: isize, end: usize) -> usize {
    lines.get(i as usize).map_or(end, |line| line.0)
}

/// `h` with the bytes its two sides share at either end left out.
fn narrow(old: &[u8], new: &[u8], h: Hunk) -> Hunk {
    let (prefix, suffix) = common_ends(&old[h.old.0..h.old.1], &new[h.new.0..h.new.1]);

    Hunk {
        old: (h.old.0 + prefix, h.old.1 - suffix),
        new: (h.new.0 + prefix, h.new.1 - suffix),
    }
}

#[cfg(test)]
mod tests {
    use super::Shift;

    #[test]
    fn offsets_move_past_the_edit() {
        let shift = Shift::between(b"abc def", b"abc\nx def");

        assert_eq!(shift.to_old(2), Some(2));
        assert_eq!(shift.to_old(7), Some(5));
        assert_eq!(shift.to_old(4), None);
        assert_eq!(shift.to_old_or_edit(4), 3);
        assert_eq!(shift.to_new(5), Some(7));
        assert_eq!(shift.to_new(0), Some(0));
        // What starts where text was inserted moves with it.
        assert_eq!(shift.to_new(3), Some(5));
    }

    #[test]
    fn the_same_text_moves_nothing() {
        let shift = Shift::between(b"abc", b"abc");

        assert!(shift.is_none());
        assert_eq!(shift.to_old(3), Some(3));
        assert_eq!(shift.to_new(3), Some(3));
    }

    /// Two edits on separate lines: the lines between them keep their place in both texts.
    #[test]
    fn text_between_two_edits_is_kept() {
        let old = b"aaa\nbbb\nccc\nddd\n";
        let new = b"aaaX\nbbb\nccc\nYddd\n";
        let shift = Shift::between(old, new);

        // `bbb` and `ccc`.
        assert_eq!(shift.to_new(4), Some(5));
        assert_eq!(shift.to_new_range(8, 11), Some((9, 12)));
        assert_eq!(shift.to_old(9), Some(8));
        assert_eq!(shift.to_old_range(5, 8), Some((4, 7)));
        // Before the first edit, and after the second.
        assert_eq!(shift.to_new(1), Some(1));
        assert_eq!(shift.to_new(13), Some(15));
        // `Y` inserted before `ddd` moves `ddd` along.
        assert_eq!(shift.to_new(12), Some(14));
        // A span over an edit has no place.
        assert_eq!(shift.to_new_range(2, 6), None);
    }

    #[test]
    fn lines_removed_and_added_apart() {
        let old = b"one\ntwo\nthree\nfour\nfive\n";
        let new = b"two\nthree\nfour\nfive\nsix\n";
        let shift = Shift::between(old, new);

        // `three` (old 8) is at 4 now.
        assert_eq!(shift.to_new_range(8, 13), Some((4, 9)));
        assert_eq!(shift.to_new(0), None);
        assert_eq!(shift.to_old(20), None);
        assert_eq!(shift.to_old(19), Some(23));
    }

    /// A byte that maps either way, outside every edit, is the same byte in both texts.
    #[test]
    fn mapped_offsets_hold_the_same_byte() {
        let texts: [&[u8]; 5] = [
            b"main :: () {\n    x := 1;\n    y := 2;\n}\nf :: () {}\n",
            b"main :: () {\n    x := 12;\n    y := 2;\n}\nf :: () { g(); }\n",
            b"// top\nmain :: () {\n    y := 2;\n}\nf :: () {}\n\n",
            b"",
            b"f :: () {}\n",
        ];

        for old in texts {
            for new in texts {
                let shift = Shift::between(old, new);

                for (i, &byte) in old.iter().enumerate() {
                    if let Some(j) = shift.to_new(i)
                        && j < new.len()
                        && shift.to_new_range(i, i + 1).is_some()
                    {
                        assert_eq!(byte, new[j], "{i} -> {j}");
                    }
                }

                for (j, &byte) in new.iter().enumerate() {
                    if let Some((i, _)) = shift.to_old_range(j, j + 1)
                        && i < old.len()
                    {
                        assert_eq!(old[i], byte, "{j} <- {i}");
                    }
                }
            }
        }
    }
}

//! "Did you mean" suggestions: the known name closest to a misspelled one.

/// Edit distance between `a` and `b` (insertions, deletions, substitutions and swaps of two
/// neighbouring characters each count one), or `None` once it exceeds `limit`.
fn distance(a: &str, b: &str, limit: usize) -> Option<usize> {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.len().abs_diff(b.len()) > limit {
        return None;
    }
    // Three rows of the dynamic-programming table: two back (for swaps), previous, current.
    let mut before: Vec<usize> = vec![0; b.len() + 1];
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    let mut current = vec![0; b.len() + 1];
    for i in 1..=a.len() {
        current[0] = i;
        let mut row_min = i;
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            let mut d = (previous[j] + 1)
                .min(current[j - 1] + 1)
                .min(previous[j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                d = d.min(before[j - 2] + 1);
            }
            current[j] = d;
            row_min = row_min.min(d);
        }
        if row_min > limit {
            return None;
        }
        std::mem::swap(&mut before, &mut previous);
        std::mem::swap(&mut previous, &mut current);
    }
    let d = previous[b.len()];
    (d <= limit).then_some(d)
}

/// The candidate closest to `wanted`: one differing only in case first, then the smallest
/// edit distance within a third of the name's length (at least one edit). Ties keep the
/// candidate first in sort order. `wanted` itself is never suggested.
pub fn closest<'a>(wanted: &str, candidates: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    let length = wanted.chars().count();
    let limit = if length < 3 {
        0
    } else {
        (length / 3).max(1)
    };
    let lower = wanted.to_lowercase();
    let mut best: Option<(usize, &'a str)> = None;
    for candidate in candidates {
        if candidate == wanted {
            continue;
        }
        let d = if candidate.to_lowercase() == lower {
            0
        } else {
            match distance(wanted, candidate, limit) {
                Some(d) => d,
                None => continue,
            }
        };
        if best.is_none_or(|best| (d, candidate) < best) {
            best = Some((d, candidate));
        }
    }
    best.map(|(_, name)| name)
}

#[cfg(test)]
mod tests {
    use super::closest;

    #[test]
    fn suggests_near_names_only() {
        let names = ["counter", "print", "Basic", "total"];
        assert_eq!(closest("countr", names), Some("counter"));
        assert_eq!(closest("prnit", names), Some("print"));
        assert_eq!(closest("basic", names), Some("Basic"));
        assert_eq!(closest("Basik", names), Some("Basic"));
        assert_eq!(closest("xyz", names), None);
        assert_eq!(closest("print", names), None);
        assert_eq!(closest("a", ["b"]), None);
        assert_eq!(closest("A", ["a"]), Some("a"));
    }
}

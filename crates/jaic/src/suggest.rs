//! "Did you mean" suggestions: the known name closest to a misspelled one.
use std::path::{Path, PathBuf};

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

/// For a file `name` that does not exist: the directory entry the user most likely meant,
/// `name.jai` when there is one (the extension was left off), else the closest name.
pub fn similar_entry<'a>(name: &str, entries: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    let entries: Vec<&'a str> = entries.into_iter().collect();
    let with_extension = format!("{name}.jai");
    entries
        .iter()
        .copied()
        .find(|e| *e == with_extension)
        .or_else(|| closest(name, entries.iter().copied()))
}

/// For a `path` that does not exist: the `similar_entry` among the entries of its directory
/// that `keep` accepts, spelled with `path`'s own directory (`foo` gives `foo.jai`, `./src/mian.jai`
/// gives `./src/main.jai`). For command lines; the compiler goes through its `FileSystem`.
pub fn similar_sibling(path: &Path, keep: impl Fn(&str) -> bool) -> Option<PathBuf> {
    let dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = path.file_name()?.to_string_lossy();
    let mut entries: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| keep(n))
        .collect();
    entries.sort();
    similar_entry(&name, entries.iter().map(String::as_str)).map(|near| path.with_file_name(near))
}

#[cfg(test)]
mod tests {
    use super::{closest, similar_entry};

    #[test]
    fn a_missing_file_suggests_its_jai_file_first() {
        let entries = ["build.jai", "main.jai", "mian"];
        assert_eq!(similar_entry("main", entries), Some("main.jai"));
        assert_eq!(similar_entry("bulid.jai", entries), Some("build.jai"));
        assert_eq!(similar_entry("other.jai", entries), None);
    }

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

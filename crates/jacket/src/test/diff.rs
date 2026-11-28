//! A line diff of expected and actual text, for golden and compile-fail tests.
use similar::{ChangeTag, TextDiff};

/// `want` and `got` line by line: `  ` both, `- ` only `want`, `+ ` only `got`. Lines are
/// compared without their line endings; a difference in the final line break gets a note.
///
/// ```text
/// --- want
/// +++ got
///   a
/// - b
/// + x
/// ```
pub fn lines(want: &str, got: &str) -> String {
    let w: Vec<&str> = want.lines().collect();
    let g: Vec<&str> = got.lines().collect();
    let mut out = String::from("--- want\n+++ got\n");
    for change in TextDiff::from_slices(&w, &g).iter_all_changes() {
        let mark = match change.tag() {
            ChangeTag::Equal => "  ",
            ChangeTag::Delete => "- ",
            ChangeTag::Insert => "+ ",
        };
        out.push_str(mark);
        out.push_str(change.value());
        out.push('\n');
    }
    if want.ends_with('\n') != got.ends_with('\n') {
        out.push_str(if got.ends_with('\n') {
            "(got ends with a line break; want does not)\n"
        } else {
            "(want ends with a line break; got does not)\n"
        });
    }
    out.truncate(out.trim_end().len());
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn changed_lines_are_marked() {
        assert_eq!(
            super::lines("a\nb\nc\n", "a\nx\nc\n"),
            "--- want\n+++ got\n  a\n- b\n+ x\n  c"
        );
        assert_eq!(
            super::lines("ok\n", "ok"),
            "--- want\n+++ got\n  ok\n(want ends with a line break; got does not)"
        );
        assert_eq!(
            super::lines("a\r\nb\n", "a\nb\n"),
            "--- want\n+++ got\n  a\n  b"
        );
    }

    #[test]
    fn a_long_output_is_still_a_diff() {
        // Far past what a table of every pair of lines could hold.
        let want: String = (0..5000).map(|i| format!("line {i}\n")).collect();
        let got = want.replace("line 2500\n", "line 2500 changed\n");
        let diff = super::lines(&want, &got);
        assert!(
            diff.contains("\n- line 2500\n+ line 2500 changed\n"),
            "{diff}"
        );
        assert_eq!(diff.lines().count(), 2 + 5001);
    }
}

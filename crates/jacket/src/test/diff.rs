//! A line diff of expected and actual text, for golden and compile-fail tests.

/// Larger inputs are shown whole: the table is quadratic.
const MAX_CELLS: usize = 4_000_000;

/// `want` and `got` line by line: `  ` both, `- ` only `want`, `+ ` only `got`.
pub fn lines(want: &str, got: &str) -> String {
    let w: Vec<&str> = want.lines().collect();
    let g: Vec<&str> = got.lines().collect();
    let mut out = String::from("--- want\n+++ got\n");
    if (w.len() + 1) * (g.len() + 1) > MAX_CELLS {
        out.push_str(&format!("want:\n{want}\ngot:\n{got}"));
        return out;
    }
    // common[i][j]: lines `w[i..]` and `g[j..]` have in common.
    let mut common = vec![vec![0u32; g.len() + 1]; w.len() + 1];
    for i in (0..w.len()).rev() {
        for j in (0..g.len()).rev() {
            common[i][j] = if w[i] == g[j] {
                common[i + 1][j + 1] + 1
            } else {
                common[i + 1][j].max(common[i][j + 1])
            };
        }
    }
    let (mut i, mut j) = (0, 0);
    while i < w.len() || j < g.len() {
        if i < w.len() && j < g.len() && w[i] == g[j] {
            out.push_str(&format!("  {}\n", w[i]));
            i += 1;
            j += 1;
        } else if j < g.len() && (i == w.len() || common[i][j + 1] >= common[i + 1][j]) {
            out.push_str(&format!("+ {}\n", g[j]));
            j += 1;
        } else {
            out.push_str(&format!("- {}\n", w[i]));
            i += 1;
        }
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
            "--- want\n+++ got\n  a\n+ x\n- b\n  c"
        );
        assert!(super::lines("ok\n", "ok").contains("want ends with a line break"));
    }
}

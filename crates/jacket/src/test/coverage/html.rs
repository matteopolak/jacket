//! `--coverage-format html`: one page with no outside files. The top is a table of modules
//! linking to their files; each file opens to its source, a line at a time with how often it
//! ran. See `mod.rs`.
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use super::report::{Data, Group, Tally, module_of, rows, total};

const STYLE: &str = "
:root { --bg: #fff; --fg: #1f2328; --muted: #59636e; --line: #d1d9e0; --hit: #dafbe1; --miss: #ffebe9; --head: #f6f8fa; --bar: #1a7f37; --track: #ffebe9; }
@media (prefers-color-scheme: dark) { :root { --bg: #0d1117; --fg: #e6edf3; --muted: #9198a1; --line: #3d444d; --hit: #12261e; --miss: #3c1618; --head: #151b23; --bar: #3fb950; --track: #5a1e22; } }
body { margin: 0 auto; max-width: 70rem; padding: 1.5rem 1rem 4rem; background: var(--bg); color: var(--fg); font: 15px/1.5 system-ui, sans-serif; }
h1 { margin: 0 0 .25rem; font-size: 1.5rem; } h2 { margin: 2rem 0 .5rem; font-size: 1.1rem; }
p.muted, td.muted { color: var(--muted); }
table.summary { width: 100%; border-collapse: collapse; margin: .5rem 0 1rem; }
table.summary th, table.summary td { padding: .3rem .6rem; border-bottom: 1px solid var(--line); text-align: right; }
table.summary th:first-child, table.summary td:first-child { text-align: left; }
table.summary th { background: var(--head); font-weight: 600; }
.bar { display: inline-block; width: 6rem; height: .6rem; background: var(--track); border-radius: 3px; overflow: hidden; vertical-align: middle; }
.bar > span { display: block; height: 100%; background: var(--bar); }
a { color: inherit; }
details { border: 1px solid var(--line); border-radius: 6px; margin: .5rem 0; }
summary { padding: .4rem .7rem; cursor: pointer; font-family: ui-monospace, monospace; font-size: .9rem; }
pre.src { margin: 0; padding: 0; overflow-x: auto; font: 13px/1.45 ui-monospace, monospace; border-top: 1px solid var(--line); }
pre.src span { display: block; white-space: pre; padding-right: 1rem; }
pre.src span.hit { background: var(--hit); } pre.src span.miss { background: var(--miss); }
pre.src i { display: inline-block; width: 4.5rem; padding-right: .75rem; text-align: right; color: var(--muted); font-style: normal; }
pre.src b { display: inline-block; width: 5rem; padding-right: .75rem; text-align: right; font-weight: 400; color: var(--muted); }
";

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

fn bar(tally: Tally) -> String {
    format!(
        "<span class=\"bar\"><span style=\"width:{:.1}%\"></span></span> {:.1}%",
        tally.percent(),
        tally.percent()
    )
}

fn cells(tally: Tally) -> String {
    format!(
        "<td>{}/{}</td><td>{}</td>",
        tally.hit,
        tally.total,
        bar(tally)
    )
}

/// One file: its source, line by line. A line the measurement has no count for is plain.
fn source(file: &Path, counts: &BTreeMap<u32, u64>) -> String {
    let text = std::fs::read_to_string(file).unwrap_or_default();
    let mut out = String::from("<pre class=\"src\">");
    let mut shown = 0;
    for (i, code) in text.lines().enumerate() {
        shown = i as u32 + 1;
        let (class, count) = match counts.get(&shown) {
            Some(0) => ("miss", "0".to_string()),
            Some(n) => ("hit", n.to_string()),
            None => ("", String::new()),
        };
        let _ = writeln!(
            out,
            "<span class=\"{class}\"><i>{shown}</i><b>{count}</b>{}</span>",
            escape(code)
        );
    }
    // Lines of a file that cannot be read (or has changed) are still listed.
    for (line, count) in counts.range(shown + 1..) {
        let class = if *count == 0 {
            "miss"
        } else {
            "hit"
        };
        let _ = writeln!(
            out,
            "<span class=\"{class}\"><i>{line}</i><b>{count}</b></span>"
        );
    }
    out.push_str("</pre>");
    out
}

/// The page for `data`, paths shown from `base`.
pub fn render(data: &Data, base: &Path) -> String {
    let modules = rows(data, base, Group::Module);
    let files = rows(data, base, Group::File);
    let (lines, functions) = total(&modules);
    let mut known = HashMap::new();
    let mut by_module: BTreeMap<String, Vec<&PathBuf>> = BTreeMap::new();
    for path in data.lines.keys() {
        by_module
            .entry(module_of(path, base, &mut known))
            .or_default()
            .push(path);
    }
    let mut page = format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>Test coverage</title>\n<style>{STYLE}</style>\n</head>\n<body>\n<h1>Test coverage</h1>\n<p class=\"muted\">{} of {} lines and {} of {} procedures ran ({:.1}% and {:.1}%).</p>\n",
        lines.hit,
        lines.total,
        functions.hit,
        functions.total,
        lines.percent(),
        functions.percent()
    );
    page.push_str("<table class=\"summary\">\n<tr><th>Module</th><th>Lines</th><th>Cover</th><th>Procedures</th><th>Cover</th></tr>\n");
    for (n, module) in modules.iter().enumerate() {
        let _ = writeln!(
            page,
            "<tr><td><a href=\"#m{n}\">{}</a></td>{}{}</tr>",
            escape(&module.name),
            cells(module.lines),
            cells(module.functions)
        );
    }
    page.push_str("</table>\n");
    let mut id = 0;
    for (n, module) in modules.iter().enumerate() {
        let _ = writeln!(page, "<h2 id=\"m{n}\">{}</h2>", escape(&module.name));
        for path in by_module.get(&module.name).into_iter().flatten() {
            let counts = &data.lines[*path];
            let name = path
                .strip_prefix(base)
                .unwrap_or(path)
                .display()
                .to_string();
            let row = files.iter().find(|r| r.name == name);
            let tally = row.map_or_else(Tally::default, |r| r.lines);
            id += 1;
            let _ = writeln!(
                page,
                "<details id=\"f{id}\"><summary>{} &mdash; {}/{} lines, {:.1}%</summary>\n{}</details>",
                escape(&name),
                tally.hit,
                tally.total,
                tally.percent(),
                source(path, counts)
            );
        }
    }
    page.push_str("</body>\n</html>\n");
    page
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_page_is_self_contained_and_marks_lines_by_whether_they_ran() {
        let dir = std::env::temp_dir().join(format!("jacket-cov-html-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("a.jai");
        std::fs::write(&file, "one :: () {}\nif a < b && c { }\n\nthree\n").unwrap();
        let mut data = Data::default();
        data.lines
            .insert(file, [(1, 3), (2, 0), (4, 1), (9, 0)].into());
        let page = render(&data, &dir);
        let _ = std::fs::remove_dir_all(&dir);
        assert!(page.starts_with("<!doctype html>"), "{page}");
        for outside in ["http://", "https://", "src=", "<link", "<script"] {
            assert!(!page.contains(outside), "{outside} in the page");
        }
        assert!(
            page.contains("<span class=\"hit\"><i>1</i><b>3</b>one :: () {}</span>"),
            "{page}"
        );
        assert!(
            page.contains(
                "<span class=\"miss\"><i>2</i><b>0</b>if a &lt; b &amp;&amp; c { }</span>"
            ),
            "{page}"
        );
        assert!(
            page.contains("<span class=\"\"><i>3</i><b></b></span>"),
            "{page}"
        );
        assert!(
            page.contains("<span class=\"miss\"><i>9</i><b>0</b></span>"),
            "{page}"
        );
        assert!(page.contains("a.jai &mdash; 2/4 lines, 50.0%"), "{page}");
    }
}

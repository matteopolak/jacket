//! Edits to `jacket.toml` that keep the rest of the file as it is: comments, order and spacing.
//! Only the `[dependencies]` entries and `[resolver] seed` are ever touched.
use jacket_config::Dependency;

fn quote(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Whether `name` can be written as a bare TOML key.
fn bare(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn key(name: &str) -> String {
    if bare(name) {
        name.to_string()
    } else {
        quote(name)
    }
}

/// `{ git = "...", version = "..." }`, keys in the order the plan documents.
pub fn inline(entry: &Dependency) -> String {
    let mut parts = vec![format!("git = {}", quote(&entry.git))];
    for (name, value) in [
        ("version", &entry.version),
        ("tag", &entry.tag),
        ("rev", &entry.rev),
        ("path", &entry.path),
        ("module", &entry.module),
        ("scheme", &entry.scheme),
        ("tag_prefix", &entry.tag_prefix),
        ("min_release_age", &entry.min_release_age),
    ] {
        if let Some(value) = value {
            parts.push(format!("{name} = {}", quote(value)));
        }
    }
    format!("{{ {} }}", parts.join(", "))
}

fn header(line: &str) -> Option<&str> {
    let line = line.trim();
    let inner = line.strip_prefix('[')?;
    if inner.starts_with('[') {
        return Some(line.trim_start_matches('[').trim_end_matches(']').trim());
    }
    Some(inner.split(']').next()?.trim())
}

/// The line range of table `name` (after its header, up to the next one).
fn table_span(lines: &[&str], name: &str) -> Option<(usize, usize)> {
    let start = lines.iter().position(|l| header(l) == Some(name))? + 1;
    let end = lines[start..]
        .iter()
        .position(|l| header(l).is_some())
        .map_or(lines.len(), |i| start + i);
    Some((start, end))
}

/// The index of the line `name = ...` within `lines[from..to]`.
fn entry_line(lines: &[&str], from: usize, to: usize, name: &str) -> Option<usize> {
    (from..to).find(|&i| {
        let line = lines[i].trim_start();
        let Some((k, _)) = line.split_once('=') else {
            return false;
        };
        k.trim().trim_matches('"') == name
    })
}

fn join(lines: &[String], text: &str) -> String {
    let mut out = lines.join("\n");
    if text.ends_with('\n') || text.is_empty() {
        out.push('\n');
    }
    out
}

/// Sets `name`'s entry under `[dependencies]`, replacing its line or adding one. An entry
/// written as its own `[dependencies.name]` table is an error: the file would have two.
pub fn set_dependency(text: &str, name: &str, entry: &Dependency) -> Result<String, String> {
    let lines: Vec<&str> = text.lines().collect();
    if lines
        .iter()
        .any(|l| header(l).is_some_and(|h| h == format!("dependencies.{name}")))
    {
        return Err(format!(
            "`[dependencies.{name}]` is written as its own table; rewrite it as `{name} = {{ ... }}` under `[dependencies]` first"
        ));
    }
    let line = format!("{} = {}", key(name), inline(entry));
    let mut out: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    match table_span(&lines, "dependencies") {
        Some((from, to)) => match entry_line(&lines, from, to, name) {
            Some(at) => out[at] = line,
            None => {
                let mut at = to;
                while at > from && lines[at - 1].trim().is_empty() {
                    at -= 1;
                }
                out.insert(at, line);
            }
        },
        None => {
            if out.last().is_some_and(|l| !l.trim().is_empty()) {
                out.push(String::new());
            }
            out.push("[dependencies]".into());
            out.push(line);
        }
    }
    Ok(join(&out, text))
}

/// Removes `name`'s entry; `false` when there was none.
pub fn remove_dependency(text: &str, name: &str) -> (String, bool) {
    let lines: Vec<&str> = text.lines().collect();
    let Some((from, to)) = table_span(&lines, "dependencies") else {
        return (text.to_string(), false);
    };
    let Some(at) = entry_line(&lines, from, to, name) else {
        return (text.to_string(), false);
    };
    let out: Vec<String> = lines
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != at)
        .map(|(_, l)| l.to_string())
        .collect();
    (join(&out, text), true)
}

/// Writes `seed = "..."` under `[resolver]`, replacing a seed already there.
pub fn set_seed(text: &str, seed: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let line = format!("seed = {}", quote(seed));
    let mut out: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    match table_span(&lines, "resolver") {
        Some((from, to)) => match entry_line(&lines, from, to, "seed") {
            Some(at) => out[at] = line,
            None => out.insert(from, line),
        },
        None => {
            if out.last().is_some_and(|l| !l.trim().is_empty()) {
                out.push(String::new());
            }
            out.push("[resolver]".into());
            out.push(line);
        }
    }
    join(&out, text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(git: &str, tag: &str) -> Dependency {
        Dependency {
            git: git.into(),
            tag: Some(tag.into()),
            ..Dependency::default()
        }
    }

    #[test]
    fn an_entry_is_added_under_a_new_table_at_the_end() {
        let text = "[project]\nentry = [\"main.jai\"]\n";
        let out = set_dependency(text, "jaison", &entry("a/jaison", "v1")).unwrap();
        assert_eq!(
            out,
            "[project]\nentry = [\"main.jai\"]\n\n[dependencies]\njaison = { git = \"a/jaison\", tag = \"v1\" }\n"
        );
    }

    #[test]
    fn an_entry_joins_its_table_keeps_comments_and_replaces_in_place() {
        let text = "[dependencies]\n# json\njaison = { git = \"a/jaison\", tag = \"v1\" }\n\n[lint]\nmax_nesting = 3\n";
        let out = set_dependency(text, "utf8", &entry("a/utf8", "v2")).unwrap();
        assert_eq!(
            out,
            "[dependencies]\n# json\njaison = { git = \"a/jaison\", tag = \"v1\" }\nutf8 = { git = \"a/utf8\", tag = \"v2\" }\n\n[lint]\nmax_nesting = 3\n"
        );
        let again = set_dependency(&out, "jaison", &entry("a/jaison", "v3")).unwrap();
        assert!(again.contains("jaison = { git = \"a/jaison\", tag = \"v3\" }"));
        assert_eq!(again.matches("jaison").count(), 2, "{again}");
    }

    #[test]
    fn a_dependency_written_as_its_own_table_is_refused() {
        let text = "[dependencies.jaison]\ngit = \"a/jaison\"\n";
        assert!(set_dependency(text, "jaison", &entry("a/jaison", "v1")).is_err());
    }

    #[test]
    fn removing_drops_one_line_only() {
        let text = "[dependencies]\na = { git = \"x/a\" }\nb = { git = \"x/b\" }\n";
        let (out, found) = remove_dependency(text, "a");
        assert!(found);
        assert_eq!(out, "[dependencies]\nb = { git = \"x/b\" }\n");
        assert!(!remove_dependency(text, "zzz").1);
    }

    #[test]
    fn the_seed_goes_under_resolver() {
        assert_eq!(set_seed("", "k"), "[resolver]\nseed = \"k\"\n");
        let text = "[resolver]\nmin_release_age = \"3d\"\n";
        assert_eq!(
            set_seed(text, "k"),
            "[resolver]\nseed = \"k\"\nmin_release_age = \"3d\"\n"
        );
        assert_eq!(
            set_seed("[resolver]\nseed = \"old\"\n", "new"),
            "[resolver]\nseed = \"new\"\n"
        );
    }
}

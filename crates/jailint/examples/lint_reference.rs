//! Prints `docs/reference/lint-rules.md` from the rule table (`jailint::RULES`) and each rule's
//! `### <rule>` section of `docs/tools/jailint.md`. `tools/generators/generate_reference_docs.py`
//! runs it. `jacket doc` turns each `###` into a page of its own.
use jaic::fxhash::HashMap;
use jailint::RULES;
use jailint::config::{Category, Level};
use std::fmt::Write;

/// The `### <name>` sections of the hand-written page, by name.
fn sections(md: &str) -> HashMap<String, String> {
    let mut out = HashMap::default();
    let mut current: Option<(String, String)> = None;
    let mut in_code = false;
    for line in md.lines() {
        if line.trim_start().starts_with("```") {
            in_code = !in_code;
        }
        if !in_code && line.starts_with('#') {
            if let Some((name, body)) = current.take() {
                out.insert(name, body);
            }
            if let Some(name) = line.strip_prefix("### ") {
                current = Some((name.trim().to_owned(), String::new()));
            }
            continue;
        }
        if let Some((_, body)) = &mut current {
            body.push_str(line);
            body.push('\n');
        }
    }
    if let Some((name, body)) = current {
        out.insert(name, body);
    }
    out
}

/// The repository root.
const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// The rule's example for its page: the code of `tests/lint/<rule>/example.jai`, the finding
/// `example.snap` records for it (the test in `crates/jailint/tests/rules.rs` keeps that
/// snapshot current), and the fixed code when there is one.
fn example(rule: &jailint::RuleInfo) -> String {
    let dir = format!("{ROOT}/tests/lint/{}", rule.name);
    let read = |name: &str| {
        std::fs::read_to_string(format!("{dir}/{name}"))
            .unwrap_or_else(|e| panic!("tests/lint/{}/{name}: {e}", rule.name))
    };
    let code = read("example.jai");
    let snap = read("example.snap");
    // The snapshot is an insta file: a front matter, then the text.
    let body = snap
        .strip_prefix("---\n")
        .and_then(|s| s.split_once("\n---\n"))
        .map_or(snap.as_str(), |(_, body)| body);
    let shown = format!("tests/lint/{}/example.jai", rule.name);
    let finding: Vec<String> = body
        .lines()
        .filter(|l| !l.trim_start().starts_with("= note: see http"))
        .map(|l| l.replace(&shown, "example.jai"))
        .collect();
    let intro = if rule.default() == Level::Allow {
        "Example (off by default; shown with the rule enabled):"
    } else {
        "Example:"
    };
    let mut out = format!("{intro}\n\n```jai\n{}\n```\n\n", code.trim_end());
    if let Ok(settings) = std::fs::read_to_string(format!("{dir}/example.toml")) {
        let _ = write!(
            out,
            "Shown with these settings:\n\n```toml\n{}\n```\n\n",
            settings.trim_end()
        );
    }
    let _ = write!(
        out,
        "```diagnostic\n{}\n```\n",
        finding.join("\n").trim_end()
    );
    if let Ok(fixed) = std::fs::read_to_string(format!("{dir}/example-fixed.jai")) {
        let _ = write!(out, "\nFixed:\n\n```jai\n{}\n```\n", fixed.trim_end());
    }
    out
}

/// Links in `docs/tools/jailint.md` text, made to work from `docs/reference/`: an anchor of
/// another rule stays on this page, any other anchor goes to the tools page.
fn relink(text: &str, rules: &[&str]) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = rest.find("](") {
        out.push_str(&rest[..at + 2]);
        rest = &rest[at + 2..];
        let end = rest.find(')').unwrap_or(rest.len());
        let dest = &rest[..end];
        if let Some(anchor) = dest.strip_prefix('#') {
            if rules.contains(&anchor) {
                out.push_str(dest);
            } else {
                let _ = write!(out, "../tools/jailint.md{dest}");
            }
        } else if dest.contains("://") || dest.starts_with("../") || dest.starts_with('/') {
            out.push_str(dest);
        } else {
            let _ = write!(out, "../tools/{dest}");
        }
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

fn title(word: &str) -> String {
    let mut c = word.chars();
    c.next()
        .map(|f| f.to_uppercase().chain(c).collect())
        .unwrap_or_default()
}

fn main() {
    let guide = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/tools/jailint.md"
    ))
    .expect("docs/tools/jailint.md");
    let prose = sections(&guide);
    let names: Vec<&str> = RULES.iter().map(|r| r.name).collect();
    let mut out = String::new();
    let _ = write!(
        out,
        "# Lint rules\n\n\
         The {} rules `jailint` checks, by category. [jailint](../tools/jailint.md) says how to \
         run it, configure rules and silence a finding.\n\n\
         A rule's category sets its default level. Name a category wherever a rule name works \
         (`[lint.rules]`, `-W`, `// jailint: allow(...)`).\n\n\
         | Category | Default | What |\n| --- | --- | --- |\n",
        RULES.len()
    );
    for category in Category::ALL {
        let _ = writeln!(
            out,
            "| [`{}`](#{}) | {} | {} |",
            category.as_str(),
            category.as_str(),
            category.level().as_str(),
            category.about()
        );
    }
    for category in Category::ALL {
        let mut rules: Vec<_> = RULES.iter().filter(|r| r.category == category).collect();
        if rules.is_empty() {
            continue;
        }
        rules.sort_by_key(|rule| rule.name);
        let _ = write!(
            out,
            "\n## {}\n\n{}. Default: `{}`.\n",
            title(category.as_str()),
            title(category.about()),
            category.level().as_str()
        );
        for rule in rules {
            let _ = write!(
                out,
                "\n### {}\n\nFinds {}.\n\nCategory `{}`, `{}` by default.\n",
                rule.name,
                rule.summary,
                rule.category.as_str(),
                rule.default().as_str(),
            );
            let _ = write!(out, "\n{}", example(rule));
            if let Some(text) = prose.get(rule.name) {
                let text = relink(text.trim(), &names);
                if !text.is_empty() {
                    let _ = write!(out, "\n{text}\n");
                }
            }
        }
    }
    print!("{out}");
}

//! Prints `docs/reference/lint-rules.md` from the rule table (`jailint::RULES`).
//! `tools/generators/generate_reference_docs.py` runs it.
use jailint::RULES;
use jailint::config::Category;
use std::fmt::Write;

fn main() {
    let mut out = String::new();
    out.push_str(
        "# Lint rules\n\n\
         Every rule `jailint` checks, with its category and default level. \
         [jailint](../tools/jailint.md) explains each rule and how to configure them.\n\n\
         ## Categories\n\n\
         A rule's category sets its default level. Name a category wherever a rule name works \
         (`[lint.rules]`, `-W`, `// jailint: allow(...)`).\n\n\
         | Category | Default | What |\n| --- | --- | --- |\n",
    );
    for category in Category::ALL {
        let _ = writeln!(
            out,
            "| `{}` | {} | {} |",
            category.as_str(),
            category.level().as_str(),
            category.about()
        );
    }
    let _ = write!(out, "\n## Rules\n\n{} rules.\n\n", RULES.len());
    out.push_str("| Rule | Category | Default | Finds |\n| --- | --- | --- | --- |\n");
    let mut rules: Vec<_> = RULES.iter().collect();
    rules.sort_by_key(|rule| rule.name);
    for rule in rules {
        let _ = writeln!(
            out,
            "| [`{name}`](../tools/jailint.md#{name}) | {} | {} | {} |",
            rule.category.as_str(),
            rule.default().as_str(),
            rule.summary.replace('|', "\\|"),
            name = rule.name,
        );
    }
    print!("{out}");
}

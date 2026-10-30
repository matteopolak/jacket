//! Prints `docs/reference/environment-variables.md` from the table in `jacket_config::env`.
//! `tools/generators/generate_reference_docs.py` runs it.
use jacket_config::env::{ALL, Group};
use std::fmt::Write;

fn main() {
    let mut out = String::new();
    out.push_str(
        "# Environment variables\n\n\
         Every environment variable `jaic`, `jacket`, `jailint` and the language server read. \
         See [the environment registry](../tools/environment.md).\n\n\
         Path lists use the platform separator (`:` on Linux and macOS, `;` on Windows). An empty value counts \
         as unset wherever a setting says so.\n",
    );
    for group in Group::ALL {
        let vars: Vec<_> = ALL.iter().filter(|var| var.group == group).collect();
        if vars.is_empty() {
            continue;
        }
        let _ = write!(out, "\n## {}\n\n", group.title());
        out.push_str("| Variable | Meaning | When unset | Read by |\n| --- | --- | --- | --- |\n");
        let mut vars = vars;
        vars.sort_by_key(|var| var.name.to_ascii_uppercase());
        for var in vars {
            let readers: Vec<_> = var.readers.iter().map(|reader| reader.label()).collect();
            let _ = writeln!(
                out,
                "| `{}` | {} | {} | {} |",
                var.name,
                var.summary,
                var.default,
                readers.join(", ")
            );
        }
    }
    print!("{out}");
}

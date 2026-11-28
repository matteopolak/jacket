//! The lifetime rules' work grows with a procedure's size, not with its square. Work is counted
//! (`JAILINT_LIFETIME_STATS`), not timed, so the checks hold on any machine.
use std::process::Command;

use test_support::TempDir;

/// A procedure of `blocks` sequential scopes, each allocating, deferring a free and asserting
/// on state shared by all of them.
fn procedure(blocks: usize) -> String {
    let mut text = String::from(
        "#import \"Basic\";\n\
         Item :: struct { data: [..] int; label: string; value: int; }\n\
         State :: struct { items: [..] Item; name: string; count: int; }\n\
         touch :: (s: *State, k: int) -> bool {\n\
         \x20   if k < 0 return false;\n\
         \x20   s.count += k;\n\
         \x20   return true;\n\
         }\n\
         big_check :: () {\n\
         \x20   s: State;\n\
         \x20   defer { for *it: s.items { array_free(it.data); free(it.label); } array_free(s.items); free(s.name); }\n",
    );

    for i in 0..blocks {
        text.push_str(&format!(
            "    {{\n\
             \x20       tmp := copy_string(\"item\");\n\
             \x20       defer free(tmp);\n\
             \x20       assert(touch(*s, {i}) && tmp.count == 4);\n\
             \x20       if s.count > 0 {{ s.count -= 1; }}\n\
             \x20   }}\n"
        ));
    }

    text.push_str("}\nmain :: () { big_check(); }\n");
    text
}

/// The statistics lines jailint prints for `blocks` scopes.
fn stats(blocks: usize) -> String {
    let dir = TempDir::new("lifetime-scaling");

    std::fs::write(dir.join("main.jai"), procedure(blocks)).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_jailint"))
        .arg("main.jai")
        .current_dir(&dir)
        .env("JAILINT_LIFETIME_STATS", "1")
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn field(line: &str, name: &str) -> usize {
    let at = line.find(&format!("{name}=")).unwrap() + name.len() + 1;

    line[at..]
        .split(|c: char| !c.is_ascii_digit())
        .next()
        .unwrap()
        .parse()
        .unwrap()
}

/// Blocks the per-candidate dataflow visited, over every lifetime rule.
fn swept(text: &str) -> usize {
    text.lines()
        .filter(|l| l.starts_with("lifetime-resource-stats") || l.starts_with("lifetime-stats"))
        .map(|l| field(l, "sweep_blocks"))
        .sum()
}

#[test]
fn a_procedure_four_times_longer_costs_about_four_times_the_dataflow() {
    let (small, large) = (swept(&stats(30)), swept(&stats(120)));

    assert!(small > 0, "no dataflow ran");
    // Linear growth is 4x; a sweep of the whole CFG per candidate is 16x.
    assert!(
        large < small * 6,
        "{small} blocks for 30 scopes, {large} for 120"
    );
}

#[test]
fn each_procedure_is_lowered_once_for_all_the_lifetime_rules() {
    let text = stats(10);
    let mut built: Vec<_> = text
        .lines()
        .filter(|l| l.starts_with("lifetime-cfg-build"))
        .map(|l| field(l, "proc"))
        .collect();

    assert!(!built.is_empty(), "no CFG was built");
    let all = built.len();

    built.sort_unstable();
    built.dedup();
    assert_eq!(built.len(), all, "{text}");
}

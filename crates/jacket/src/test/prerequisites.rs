//! Build outputs a checkout needs before some tests can run. A fresh checkout (a git worktree
//! above all) has none of them, and each missing one fails every test that uses it with a link
//! error or a failed `file_open`; this names them once, with the command that builds each.
use super::Outcome;
use std::collections::BTreeSet;

const DEBUG_HELPER: &str = "developer_tools_debug";

/// One line for each build output the failed tests were missing, in a stable order.
pub fn missing(outcomes: &[&Outcome]) -> Vec<String> {
    let mut libraries = BTreeSet::new();
    let (mut debug_helper, mut fixtures) = (false, false);
    for outcome in outcomes {
        let texts = outcome
            .problems
            .iter()
            .cloned()
            .chain([String::from_utf8_lossy(&outcome.stderr).into_owned()]);
        for text in texts {
            for name in missing_libraries(&text) {
                if name.ends_with(DEBUG_HELPER) {
                    debug_helper = true;
                } else {
                    libraries.insert(name);
                }
            }
            fixtures |= text.contains("file_open failed");
        }
    }
    let mut lines = Vec::new();
    if !libraries.is_empty() {
        let names: Vec<_> = libraries.into_iter().collect();
        lines.push(format!(
            "native libraries not built ({}): PYTHONPATH=tools python3 tools/native/build_native_libs.py",
            names.join(", ")
        ));
    }
    if debug_helper {
        lines.push(format!(
            "`{DEBUG_HELPER}` not built: PYTHONPATH=tools python3 tools/developer_checks/build_developer_debug.py"
        ));
    }
    if fixtures {
        lines.push(
            "a test could not open a file; if it reads generated fixtures, build them: PYTHONPATH=tools python3 tools/portable/build_portable.py data"
                .into(),
        );
    }
    lines
}

/// The names in `library 'NAME' not found` (the linker's and jaic's own wording).
fn missing_libraries(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find("library '") {
        rest = &rest[at + "library '".len()..];
        let Some(end) = rest.find('\'') else {
            break;
        };
        if rest[end + 1..].starts_with(" not found") {
            names.push(rest[..end].to_string());
        }
        rest = &rest[end..];
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_come_from_both_wordings() {
        let text = "ld: library 'mbedtls' not found\nerror: library 'debug-native/developer_tools_debug' not found: looked";
        assert_eq!(
            missing_libraries(text),
            ["mbedtls", "debug-native/developer_tools_debug"]
        );
        assert!(missing_libraries("library 'x' is fine").is_empty());
    }
}

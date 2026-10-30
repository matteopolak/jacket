//! jacket's own build status lines, such as the `Running` line after `jacket run` builds. They
//! follow the rules jaic's status uses (`jaic::progress::mode_with`), so they appear exactly when
//! jaic's `Compiling` and `Finished` lines do.
use std::ffi::OsString;

/// Whether status lines are printed for a jaic command line, and in colour: `None` for none.
pub fn lines(argv: &[OsString]) -> Option<bool> {
    use std::io::IsTerminal;
    lines_with(
        argv,
        std::io::stderr().is_terminal(),
        jacket_config::env::lookup,
    )
}

fn lines_with(
    argv: &[OsString],
    terminal: bool,
    env: impl Fn(&str) -> Option<String>,
) -> Option<bool> {
    let flags = argv.iter().take_while(|a| *a != "-" && *a != "--");
    let mut quiet = false;
    let mut no_color = false;
    for flag in flags {
        quiet |= flag == "-quiet" || flag == "--quiet";
        no_color |= flag == "-no_color" || flag == "-msvc_format";
    }
    if quiet {
        return None;
    }
    match env("JAIC_PROGRESS").as_deref() {
        Some("off") => return None,
        Some("plain") => return Some(false),
        Some("live") => return Some(true),
        _ => {}
    }
    if !terminal {
        return None;
    }
    let ansi_console = !cfg!(windows)
        || env("WT_SESSION").is_some()
        || env("ANSICON").is_some()
        || env("ConEmuANSI").as_deref() == Some("ON")
        || env("TERM").is_some();
    let ci = env("CI").is_some_and(|v| !v.is_empty() && v != "0" && v != "false");
    let plain = no_color
        || ci
        || !ansi_console
        || env("NO_COLOR").is_some()
        || env("TERM").as_deref() == Some("dumb");
    Some(!plain)
}

/// Prints `verb` right-aligned in 12 columns, then `text`, like jaic's status lines.
pub fn print(verb: &str, text: &str, color: bool) {
    if color {
        eprintln!("\x1b[1;32m{verb:>12}\x1b[0m {text}");
    } else {
        eprintln!("{verb:>12} {text}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(args: &[&str]) -> Vec<OsString> {
        args.iter().map(OsString::from).collect()
    }

    #[test]
    fn follows_jaic_rules() {
        let none = |_: &str| None;
        let term = |key: &str| (key == "TERM").then(|| "xterm".to_string());
        assert_eq!(lines_with(&argv(&["a.jai"]), false, none), None);
        assert_eq!(lines_with(&argv(&["a.jai"]), true, term), Some(true));
        assert_eq!(lines_with(&argv(&["a.jai", "-quiet"]), true, term), None);
        assert_eq!(
            lines_with(&argv(&["a.jai", "--", "-quiet"]), true, term),
            Some(true)
        );
        assert_eq!(
            lines_with(&argv(&["a.jai", "-no_color"]), true, term),
            Some(false)
        );
        let plain = |key: &str| (key == "JAIC_PROGRESS").then(|| "plain".to_string());
        assert_eq!(lines_with(&argv(&["a.jai"]), false, plain), Some(false));
    }
}

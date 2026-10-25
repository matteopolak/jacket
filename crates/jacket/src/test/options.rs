//! Per-test options: the notes on a `@test` procedure (`@timeout(5)`, `@only(.LINUX)`), and the
//! same options in a `// test: timeout(5) only(.LINUX)` comment at the top of a file, which sets
//! them for every test the file holds.

/// What a test asks for. `None` and empty lists leave the run-wide setting in charge.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TestOptions {
    /// Seconds before the test is killed; 0 for no limit.
    pub timeout: Option<u64>,
    /// Known to take long: started first, never reported as slow.
    pub slow: bool,
    /// Runs while no other test does.
    pub serial: bool,
    pub retries: Option<u32>,
    /// Targets (`linux`, `macos`, `windows`, `wasm`, `x64`, `arm64`, and `interp` or `native`
    /// for how it runs) the test runs on.
    pub only: Vec<String>,
    /// Targets the test is skipped on.
    pub skip_on: Vec<String>,
    /// Capabilities (`network`, `gpu`) the test is skipped without.
    pub needs: Vec<String>,
    pub interp_only: bool,
    pub native_only: bool,
    /// The program is expected to exit with this status, without a failed check.
    pub exit: Option<i32>,
    /// The test must fail; with text, a failure message or its standard error must contain it.
    pub should_fail: Option<String>,
    /// A known failure, with the reason. Passing is an error, so the note gets removed.
    pub xfail: Option<String>,
    pub allow_leaks: bool,
    /// A doc test marked `ignore`.
    pub ignore: bool,
}

/// Where tests run: the target and how.
#[derive(Clone, Debug, PartialEq)]
pub struct Target {
    pub os: String,
    pub cpu: String,
    pub interp: bool,
}

const OS_NAMES: [&str; 4] = ["linux", "macos", "windows", "wasm"];
const CPU_NAMES: [&str; 2] = ["x64", "arm64"];
const MODE_NAMES: [&str; 2] = ["interp", "native"];

fn is_target(name: &str) -> bool {
    OS_NAMES.contains(&name) || CPU_NAMES.contains(&name) || MODE_NAMES.contains(&name)
}

impl TestOptions {
    /// Applies one note's text (`timeout(5)`). Unknown notes are an error to warn about; the
    /// note itself is ignored.
    pub fn apply(&mut self, note: &str) -> Result<(), String> {
        let (name, args) = match note.split_once('(') {
            Some((name, rest)) => {
                let Some(args) = rest.strip_suffix(')') else {
                    return Err(format!("`@{note}` has no closing `)`"));
                };
                (name, Some(args))
            }
            None => (note, None),
        };
        let list = || arguments(args.unwrap_or(""));
        let number = |what: &str| -> Result<u64, String> {
            args.and_then(|a| a.trim().parse().ok()).ok_or_else(|| {
                format!("`@{name}` takes a whole number of {what}, as in `@{name}(5)`")
            })
        };
        match name {
            "timeout" => self.timeout = Some(number("seconds")?),
            "retries" => {
                self.retries = Some(
                    u32::try_from(number("retries")?)
                        .map_err(|_| "too many retries".to_string())?,
                );
            }
            "exit" => {
                let code = args.and_then(|a| a.trim().parse().ok());
                self.exit =
                    Some(code.ok_or("`@exit` takes the expected status, as in `@exit(3)`")?);
            }
            "slow" => self.slow = true,
            "serial" => self.serial = true,
            "interp_only" => self.interp_only = true,
            "native_only" => self.native_only = true,
            "allow_leaks" => self.allow_leaks = true,
            "only" | "skip_on" => {
                let targets = list();
                if let Some(bad) = targets.iter().find(|t| !is_target(t)) {
                    return Err(format!(
                        "`@{name}` does not know the target `{bad}`; use .LINUX, .MACOS, .WINDOWS, .WASM, .X64, .ARM64, .INTERP or .NATIVE"
                    ));
                }
                if targets.is_empty() {
                    return Err(format!("`@{name}` needs targets, as in `@{name}(.LINUX)`"));
                }
                if name == "only" {
                    self.only = targets;
                } else {
                    self.skip_on = targets;
                }
            }
            "needs" => self.needs = list(),
            "should_fail" => self.should_fail = Some(list().join(",")),
            "xfail" => self.xfail = Some(list().join(",")),
            // Discovery's own notes.
            "test" | "bench" => {}
            _ => return Err(format!("unknown note `@{note}` (ignored)")),
        }
        Ok(())
    }

    /// The options of a file's `// test:` comments, then `notes` over them. Problems come back
    /// as warnings.
    pub fn with_notes(&self, notes: &[String]) -> (TestOptions, Vec<String>) {
        let mut options = self.clone();
        let warnings = notes
            .iter()
            .filter_map(|n| options.apply(n).err())
            .collect();
        (options, warnings)
    }

    /// Why the test does not run on `target` with `have` capabilities, if it does not.
    pub fn skip_reason(&self, target: &Target, have: &[String]) -> Option<String> {
        if self.ignore {
            return Some("ignored (`jai,ignore`)".into());
        }
        if self.interp_only && !target.interp {
            return Some("interp only".into());
        }
        if self.native_only && target.interp {
            return Some("native only".into());
        }
        let mode = if target.interp {
            "interp"
        } else {
            "native"
        };
        let matches = |t: &String| *t == target.os || *t == target.cpu || t == mode;
        // Each kind of target `only` names must match: `only(.LINUX, .X64)` is Linux on x64.
        let misses = |names: &[&str]| {
            let named: Vec<&String> = self
                .only
                .iter()
                .filter(|t| names.contains(&t.as_str()))
                .collect();
            !named.is_empty() && !named.iter().any(|t| matches(t))
        };
        if misses(&OS_NAMES) || misses(&CPU_NAMES) || misses(&MODE_NAMES) {
            return Some(format!("only on {}", self.only.join(", ")));
        }
        if let Some(t) = self.skip_on.iter().find(|t| matches(t)) {
            return Some(format!("skipped on {t}"));
        }
        let missing: Vec<&String> = self.needs.iter().filter(|n| !have.contains(n)).collect();
        if !missing.is_empty() {
            let names: Vec<&str> = missing.iter().map(|s| s.as_str()).collect();
            return Some(format!(
                "needs {} (--needs {})",
                names.join(", "),
                names.join(",")
            ));
        }
        None
    }
}

/// `.LINUX, .MACOS` or `"text"` as lowercase names without the dots and quotes.
fn arguments(args: &str) -> Vec<String> {
    args.split(',')
        .map(|a| a.trim().trim_start_matches('.').trim_matches('"'))
        .filter(|a| !a.is_empty())
        .map(str::to_ascii_lowercase)
        .collect()
}

/// The notes of the `// test:` comments at the top of `source`, before its first line of code.
pub fn header_notes(source: &str) -> Vec<String> {
    let mut notes = Vec::new();
    for line in source.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some(comment) = line.strip_prefix("//") else {
            break;
        };
        if let Some(items) = comment.trim().strip_prefix("test:") {
            notes.extend(split_notes(items));
        }
    }
    notes
}

/// `timeout(10) only(.LINUX, .MACOS)` as `["timeout(10)", "only(.LINUX,.MACOS)"]`: spaces
/// inside parentheses do not split.
fn split_notes(items: &str) -> Vec<String> {
    let mut notes = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    for c in items.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ => {}
        }
        if c.is_whitespace() {
            if depth == 0 && !current.is_empty() {
                notes.push(std::mem::take(&mut current));
            }
            continue;
        }
        current.push(c);
    }
    if !current.is_empty() {
        notes.push(current);
    }
    notes
        .into_iter()
        .map(|n| n.trim_start_matches('@').to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(os: &str, interp: bool) -> Target {
        Target {
            os: os.into(),
            cpu: "arm64".into(),
            interp,
        }
    }

    #[test]
    fn notes_set_options() {
        let notes: Vec<String> = [
            "timeout(5)",
            "only(.LINUX,.MACOS)",
            "should_fail(bounds)",
            "serial",
            "exit(3)",
        ]
        .map(String::from)
        .to_vec();
        let (options, warnings) = TestOptions::default().with_notes(&notes);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(options.timeout, Some(5));
        assert_eq!(options.only, ["linux", "macos"]);
        assert_eq!(options.should_fail.as_deref(), Some("bounds"));
        assert!(options.serial);
        assert_eq!(options.exit, Some(3));
    }

    #[test]
    fn bad_notes_are_warnings() {
        let notes: Vec<String> = ["timeout", "only(.BEOS)", "flaky"]
            .map(String::from)
            .to_vec();
        let (options, warnings) = TestOptions::default().with_notes(&notes);
        assert_eq!(options, TestOptions::default());
        assert_eq!(warnings.len(), 3);
        assert!(warnings[2].contains("unknown note `@flaky`"));
    }

    #[test]
    fn targets_and_capabilities_decide_skips() {
        let mut options = TestOptions::default();
        options.apply("only(.LINUX, .MACOS)").unwrap();
        assert_eq!(options.skip_reason(&target("macos", false), &[]), None);
        assert!(
            options
                .skip_reason(&target("windows", false), &[])
                .is_some()
        );
        options.apply("skip_on(.ARM64)").unwrap();
        assert_eq!(
            options.skip_reason(&target("macos", false), &[]).as_deref(),
            Some("skipped on arm64")
        );
        let mut network = TestOptions::default();
        network.apply("needs(.NETWORK)").unwrap();
        assert!(network.skip_reason(&target("linux", false), &[]).is_some());
        assert_eq!(
            network.skip_reason(&target("linux", false), &["network".into()]),
            None
        );
        let mut native = TestOptions::default();
        native.apply("native_only").unwrap();
        assert!(native.skip_reason(&target("linux", true), &[]).is_some());
        let mut compiled = TestOptions::default();
        compiled.apply("skip_on(.INTERP)").unwrap();
        assert!(compiled.skip_reason(&target("linux", true), &[]).is_some());
        assert_eq!(compiled.skip_reason(&target("linux", false), &[]), None);
        let mut wasm_interp = TestOptions::default();
        wasm_interp.apply("only(.WASM, .INTERP)").unwrap();
        assert_eq!(wasm_interp.skip_reason(&target("wasm", true), &[]), None);
        assert!(
            wasm_interp
                .skip_reason(&target("wasm", false), &[])
                .is_some()
        );
    }

    #[test]
    fn header_comments_hold_notes() {
        let source = "// A golden test.\n// test: only(.LINUX, .MACOS) timeout(10)\n\n#import \"Basic\";\n// test: serial\n";
        assert_eq!(header_notes(source), ["only(.LINUX,.MACOS)", "timeout(10)"]);
    }
}

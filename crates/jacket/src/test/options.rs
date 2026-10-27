//! Per-test options: the notes on a `@test` procedure (`@timeout(5)`, `@only(.LINUX)`), and the
//! same options in a `// test: timeout(5) only(.LINUX)` comment at the top of a file, which sets
//! them for every test the file holds.

use std::fmt;

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
    /// One filter per `only(...)` call; the test runs on a target that passes every one.
    pub only: Vec<TargetFilter>,
    /// One filter per `skip_on(...)` call; the test is skipped on a target that matches any.
    pub skip_on: Vec<TargetFilter>,
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
    /// Golden tests: the program's command-line arguments. `$TEMP` is a fresh folder.
    pub args: Vec<String>,
    /// Golden tests: only type-check the program (`jaic -check`); it must pass.
    pub check_only: bool,
    /// Golden tests: the build output (compiled) or standard error (interpreted) must contain
    /// this compiler warning.
    pub warns: Option<String>,
    /// Compile-fail tests: the diagnostics need only contain this text, for output that depends
    /// on the host (the C compiler `BuildCpp` found).
    pub diagnostic_contains: Option<String>,
}

/// Where tests run: the target and how.
#[derive(Clone, Debug, PartialEq)]
pub struct Target {
    pub os: String,
    pub cpu: String,
    pub interp: bool,
}

const OS_NAMES: [&str; 5] = ["linux", "macos", "windows", "wasm", "android"];
const CPU_NAMES: [&str; 2] = ["x64", "arm64"];
const MODE_NAMES: [&str; 2] = ["interp", "native"];

/// The arguments of one `only(...)` or `skip_on(...)` call: target names (an OS, a CPU or a
/// mode, as in `.WASM`, `.ARM64`, `.INTERP`), and `mode = .NATIVE` or `cpu = .ARM64`, which
/// narrow the whole call.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TargetFilter {
    pub names: Vec<String>,
    pub mode: Option<String>,
    pub cpu: Option<String>,
}

impl TargetFilter {
    /// `.WASM, .WINDOWS, cpu = .ARM64` as a filter. `call` is `only` or `skip_on`.
    fn parse(call: &str, args: &str) -> Result<TargetFilter, String> {
        let mut filter = TargetFilter::default();
        for item in args.split(',').map(str::trim).filter(|i| !i.is_empty()) {
            if let Some((key, value)) = item.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_start_matches('.').to_ascii_lowercase();
                let (slot, allowed) = match key {
                    "mode" => (&mut filter.mode, &MODE_NAMES),
                    "cpu" => (&mut filter.cpu, &CPU_NAMES),
                    _ => {
                        return Err(format!(
                            "`@{call}` does not know the argument `{key}`; use `mode =` or `cpu =`"
                        ));
                    }
                };
                if !allowed.contains(&value.as_str()) {
                    return Err(format!(
                        "`@{call}` does not know the {key} `{value}`; use {}",
                        dotted(allowed)
                    ));
                }
                if slot.replace(value).is_some() {
                    return Err(format!("`@{call}` gives `{key} =` twice"));
                }
                continue;
            }
            let name = item.trim_start_matches('.').to_ascii_lowercase();
            if ![&OS_NAMES[..], &CPU_NAMES[..], &MODE_NAMES[..]]
                .iter()
                .any(|names| names.contains(&name.as_str()))
            {
                return Err(format!(
                    "`@{call}` does not know the target `{name}`; use .LINUX, .MACOS, .WINDOWS, .WASM, .ANDROID, .X64, .ARM64, .INTERP or .NATIVE, and `mode =` or `cpu =` to narrow them"
                ));
            }
            filter.names.push(name);
        }
        if filter.names.is_empty() && filter.mode.is_none() && filter.cpu.is_none() {
            return Err(format!("`@{call}` needs targets, as in `@{call}(.LINUX)`"));
        }
        Ok(filter)
    }

    fn named(target: &Target, mode: &str, name: &str) -> bool {
        name == target.os || name == target.cpu || name == mode
    }

    /// `mode =` and `cpu =`, which every target of the call must satisfy.
    fn narrowed(&self, target: &Target, mode: &str) -> bool {
        self.mode.as_deref().is_none_or(|m| m == mode)
            && self.cpu.as_deref().is_none_or(|c| c == target.cpu)
    }

    /// Whether `target` is one the call names: any of the names (any OS if there are none), and
    /// the mode and CPU.
    fn matches(&self, target: &Target, mode: &str) -> bool {
        (self.names.is_empty() || self.names.iter().any(|n| Self::named(target, mode, n)))
            && self.narrowed(target, mode)
    }

    /// Whether `target` passes an `only` call: each kind of name given (OS, CPU, mode) must
    /// match one of that kind, so `only(.LINUX, .X64)` is Linux on x64.
    fn allows(&self, target: &Target, mode: &str) -> bool {
        [&OS_NAMES[..], &CPU_NAMES[..], &MODE_NAMES[..]]
            .iter()
            .all(|kind| {
                let mut named = self
                    .names
                    .iter()
                    .filter(|n| kind.contains(&n.as_str()))
                    .peekable();
                named.peek().is_none() || named.any(|n| Self::named(target, mode, n))
            })
            && self.narrowed(target, mode)
    }
}

impl fmt::Display for TargetFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = self.names.clone();
        parts.extend(self.mode.iter().map(|m| format!("mode = {m}")));
        parts.extend(self.cpu.iter().map(|c| format!("cpu = {c}")));
        f.write_str(&parts.join(", "))
    }
}

fn dotted(names: &[&str]) -> String {
    let named: Vec<String> = names.iter().map(|n| n.to_ascii_uppercase()).collect();
    format!(".{}", named.join(" or ."))
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
                let filter = TargetFilter::parse(name, args.unwrap_or(""))?;
                if name == "only" {
                    self.only.push(filter);
                } else {
                    self.skip_on.push(filter);
                }
            }
            "needs" => self.needs = list(),
            "should_fail" => self.should_fail = Some(text(args)),
            "xfail" => self.xfail = Some(text(args)),
            "warns" => {
                let want = text(args);
                if want.is_empty() {
                    return Err(
                        "`@warns` needs the warning text, as in `@warns(\"not in scope\")`".into(),
                    );
                }
                self.warns = Some(want);
            }
            "args" => self.args = quoted_list(args.unwrap_or("")),
            "check_only" => self.check_only = true,
            "compile_fail" => {
                let want = text(args);
                if args.is_some() && want.is_empty() {
                    return Err(
                        "`compile_fail(text)` needs the text the diagnostics contain".into(),
                    );
                }
                self.diagnostic_contains = (!want.is_empty()).then_some(want);
            }
            // Discovery's own notes.
            "test" | "bench" | "cases" | "compile_time" | "generate" => {}
            // A bench's own notes; Discovery checks them and the program reads them.
            "samples" | "min_time" => {}
            "property" => {
                if args.is_some_and(|a| a.trim().parse::<u32>().unwrap_or(0) == 0) {
                    return Err(
                        "`@property` takes a number of trials, as in `@property(500)`, or none"
                            .into(),
                    );
                }
            }
            _ => return Err(format!("unknown note `@{note}` (ignored)")),
        }
        Ok(())
    }

    /// The options of a file's `// test:` comments, then `notes` over them. Problems come back
    /// as warnings.
    pub fn with_notes(&self, notes: &[String]) -> (TestOptions, Vec<String>) {
        let mut options = self.clone();
        // The first `only` or `skip_on` among `notes` replaces what `self` had; later ones add
        // to it, so a file can carry several `skip_on(...)` calls.
        let (mut only, mut skip_on) = (false, false);
        let warnings = notes
            .iter()
            .filter_map(|n| {
                let call = n.split('(').next().unwrap_or("");
                if call == "only" && !std::mem::replace(&mut only, true) {
                    options.only.clear();
                } else if call == "skip_on" && !std::mem::replace(&mut skip_on, true) {
                    options.skip_on.clear();
                }
                options.apply(n).err()
            })
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
        if let Some(filter) = self.only.iter().find(|f| !f.allows(target, mode)) {
            return Some(format!("only on {filter}"));
        }
        if let Some(filter) = self.skip_on.iter().find(|f| f.matches(target, mode)) {
            // Name the targets that matched, not every one the call lists.
            let matched = TargetFilter {
                names: filter
                    .names
                    .iter()
                    .filter(|n| TargetFilter::named(target, mode, n))
                    .cloned()
                    .collect(),
                ..filter.clone()
            };
            return Some(format!("skipped on {matched}"));
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

/// A note's argument as written (case kept), without the quotes around it.
fn text(args: Option<&str>) -> String {
    let args = args.unwrap_or("").trim();
    args.strip_prefix('"')
        .and_then(|a| a.strip_suffix('"'))
        .unwrap_or(args)
        .to_string()
}

/// `"a b", c` as `["a b", "c"]`: commas inside quotes do not split; case is kept. Inside quotes,
/// `\"` is a quote and `\\` a backslash, and `""` is an empty argument.
fn quoted_list(args: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut had_quote = false;
    let mut finish = |current: &mut String, had_quote: &mut bool| {
        let text = std::mem::take(current);
        if *had_quote {
            items.push(text);
        } else if !text.trim().is_empty() {
            items.push(text.trim().to_string());
        }
        *had_quote = false;
    };
    let mut chars = args.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' if quoted && matches!(chars.peek(), Some('"' | '\\')) => {
                current.extend(chars.next());
            }
            '"' => {
                quoted = !quoted;
                had_quote = true;
            }
            ',' if !quoted => finish(&mut current, &mut had_quote),
            _ if !quoted && c.is_whitespace() && (had_quote || current.is_empty()) => {}
            _ => current.push(c),
        }
    }
    finish(&mut current, &mut had_quote);
    items
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
        if c.is_whitespace() && depth == 0 {
            if !current.is_empty() {
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
        assert_eq!(options.only.len(), 1);
        assert_eq!(options.only[0].names, ["linux", "macos"]);
        assert_eq!(options.should_fail.as_deref(), Some("bounds"));
        assert!(options.serial);
        assert_eq!(options.exit, Some(3));
    }

    #[test]
    fn the_cases_note_is_discoverys() {
        let notes = ["cases(all_values(Color))".to_string(), "xfail".to_string()];
        let (options, warnings) = TestOptions::default().with_notes(&notes);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(options.xfail.as_deref(), Some(""));
    }

    #[test]
    fn property_and_compile_time_notes_are_discoverys() {
        let notes: Vec<String> = [
            "property",
            "property(500)",
            "generate(even)",
            "compile_time",
        ]
        .map(String::from)
        .to_vec();
        let (options, warnings) = TestOptions::default().with_notes(&notes);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(options, TestOptions::default());
    }

    #[test]
    fn a_property_count_is_a_positive_number() {
        for bad in ["property(0)", "property(many)", "property()"] {
            let (_, warnings) = TestOptions::default().with_notes(&[bad.to_string()]);
            assert_eq!(warnings.len(), 1, "{bad}: {warnings:?}");
            assert!(warnings[0].contains("number of trials"), "{warnings:?}");
        }
    }

    #[test]
    fn file_notes_keep_the_case_and_spaces_of_their_text() {
        let source = "// test: should_fail(\"Index out of range\") args(\"a b\", \"$TEMP/Data\") warns(\"not in scope\")\n// test: check_only compile_fail\nmain :: () {}\n";
        let notes = header_notes(source);
        let (options, warnings) = TestOptions::default().with_notes(&notes);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(options.should_fail.as_deref(), Some("Index out of range"));
        assert_eq!(options.args, ["a b", "$TEMP/Data"]);
        assert_eq!(options.warns.as_deref(), Some("not in scope"));
        assert!(options.check_only);
    }

    #[test]
    fn a_compile_fail_note_can_name_the_text_the_diagnostics_contain() {
        let notes = ["compile_fail(\"tool failed\")".to_string()];
        let (options, warnings) = TestOptions::default().with_notes(&notes);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(options.diagnostic_contains.as_deref(), Some("tool failed"));
        let (plain, _) = TestOptions::default().with_notes(&["compile_fail".to_string()]);
        assert_eq!(plain.diagnostic_contains, None);
        let (_, warnings) = TestOptions::default().with_notes(&["compile_fail()".to_string()]);
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn a_target_can_name_the_os_and_cpu_together() {
        let notes = ["skip_on(.WINDOWS, cpu = .ARM64)".to_string()];
        let (options, warnings) = TestOptions::default().with_notes(&notes);
        assert!(warnings.is_empty(), "{warnings:?}");
        let on = |os: &str, cpu: &str| Target {
            os: os.into(),
            cpu: cpu.into(),
            interp: false,
        };
        assert!(options.skip_reason(&on("windows", "arm64"), &[]).is_some());
        assert_eq!(options.skip_reason(&on("windows", "x64"), &[]), None);
        assert_eq!(options.skip_reason(&on("macos", "arm64"), &[]), None);
        let mut only = TestOptions::default();
        only.apply("only(.WINDOWS, cpu = .ARM64)").unwrap();
        assert_eq!(only.skip_reason(&on("windows", "arm64"), &[]), None);
        assert!(only.skip_reason(&on("windows", "x64"), &[]).is_some());
    }

    #[test]
    fn a_target_can_name_the_os_and_mode_together() {
        let mut options = TestOptions::default();
        options.apply("skip_on(.WASM, mode = .NATIVE)").unwrap();
        assert!(options.skip_reason(&target("wasm", false), &[]).is_some());
        assert_eq!(options.skip_reason(&target("wasm", true), &[]), None);
        assert_eq!(options.skip_reason(&target("linux", false), &[]), None);
        let mut only = TestOptions::default();
        only.apply("only(.WINDOWS, mode = .NATIVE)").unwrap();
        assert_eq!(only.skip_reason(&target("windows", false), &[]), None);
        assert!(only.skip_reason(&target("windows", true), &[]).is_some());
    }

    #[test]
    fn mode_and_cpu_narrow_the_whole_call() {
        let on = |os: &str, cpu: &str, interp: bool| Target {
            os: os.into(),
            cpu: cpu.into(),
            interp,
        };
        // Any of the names, and the mode.
        let mut options = TestOptions::default();
        options
            .apply("skip_on(.WASM, .WINDOWS, mode = .NATIVE)")
            .unwrap();
        assert!(
            options
                .skip_reason(&on("wasm", "wasm", false), &[])
                .is_some()
        );
        assert!(
            options
                .skip_reason(&on("windows", "x64", false), &[])
                .is_some()
        );
        assert_eq!(options.skip_reason(&on("windows", "x64", true), &[]), None);
        assert_eq!(options.skip_reason(&on("linux", "x64", false), &[]), None);
        // No names: any OS.
        let mut any = TestOptions::default();
        any.apply("skip_on(mode = .NATIVE)").unwrap();
        assert!(any.skip_reason(&on("linux", "x64", false), &[]).is_some());
        assert_eq!(any.skip_reason(&on("linux", "x64", true), &[]), None);
        let mut cpu = TestOptions::default();
        cpu.apply("only(cpu = .X64)").unwrap();
        assert_eq!(cpu.skip_reason(&on("linux", "x64", false), &[]), None);
        assert!(cpu.skip_reason(&on("linux", "arm64", false), &[]).is_some());
        // Both together.
        let mut both = TestOptions::default();
        both.apply("only(.MACOS, mode = .INTERP, cpu = .ARM64)")
            .unwrap();
        assert_eq!(both.skip_reason(&on("macos", "arm64", true), &[]), None);
        assert!(
            both.skip_reason(&on("macos", "arm64", false), &[])
                .is_some()
        );
        assert!(both.skip_reason(&on("macos", "x64", true), &[]).is_some());
    }

    #[test]
    fn several_skip_on_calls_each_skip() {
        let notes = [
            "skip_on(.WASM)".to_string(),
            "skip_on(.WINDOWS, cpu = .ARM64)".to_string(),
        ];
        let (options, warnings) = TestOptions::default().with_notes(&notes);
        assert!(warnings.is_empty(), "{warnings:?}");
        let on = |os: &str, cpu: &str| Target {
            os: os.into(),
            cpu: cpu.into(),
            interp: false,
        };
        assert!(options.skip_reason(&on("wasm", "wasm"), &[]).is_some());
        assert!(options.skip_reason(&on("windows", "arm64"), &[]).is_some());
        assert_eq!(options.skip_reason(&on("windows", "x64"), &[]), None);
        // A later batch of notes replaces them.
        let (later, _) = options.with_notes(&["skip_on(.LINUX)".to_string()]);
        assert_eq!(later.skip_on.len(), 1);
    }

    #[test]
    fn bad_target_arguments_are_warnings() {
        for (note, expect) in [
            (
                "skip_on(.WASM-NATIVE)",
                "does not know the target `wasm-native`",
            ),
            (
                "only(.WINDOWS-ARM64)",
                "does not know the target `windows-arm64`",
            ),
            (
                "skip_on(.WASM, os = .LINUX)",
                "does not know the argument `os`",
            ),
            (
                "skip_on(.WASM, mode = .ARM64)",
                "does not know the mode `arm64`",
            ),
            (
                "skip_on(.WASM, cpu = .NATIVE)",
                "does not know the cpu `native`",
            ),
            ("skip_on(cpu = .X64, cpu = .ARM64)", "gives `cpu =` twice"),
            ("skip_on()", "needs targets"),
        ] {
            let (options, warnings) = TestOptions::default().with_notes(&[note.to_string()]);
            assert_eq!(options, TestOptions::default(), "{note}");
            assert_eq!(warnings.len(), 1, "{note}: {warnings:?}");
            assert!(warnings[0].contains(expect), "{note}: {warnings:?}");
        }
    }

    #[test]
    fn quoted_arguments_keep_quotes_backslashes_and_empty_strings() {
        assert_eq!(
            quoted_list(r#""a b", "", "q\" and s\\", plain"#),
            ["a b", "", "q\" and s\\", "plain"]
        );
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
        assert_eq!(
            header_notes(source),
            ["only(.LINUX, .MACOS)", "timeout(10)"]
        );
    }
}

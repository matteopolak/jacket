//! Per-test options: the notes on a `@test` procedure (`@timeout(5)`, `@only(.LINUX)`), and the
//! same options in a `// test: timeout(5) only(.LINUX)` comment at the top of a file, which sets
//! them for every test the file holds.

pub use jaic::test_notes::{Target, TargetNotes, header_notes, text};

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
    /// Where the test runs: `only`, `skip_on`, `interp_only` and `native_only`.
    pub target: TargetNotes,
    /// Capabilities (`network`, `gpu`) the test is skipped without.
    pub needs: Vec<String>,
    /// The program is expected to exit with this status, without a failed check.
    pub exit: Option<i32>,
    /// The test must fail; with texts, a failure message or its standard error must contain one.
    pub should_fail: Option<Vec<String>>,
    /// A known failure, with the reason. Passing is an error, so the note gets removed.
    pub xfail: Option<String>,
    pub allow_leaks: bool,
    /// A doc test marked `ignore`.
    pub ignore: bool,
    /// Golden tests: the program's command-line arguments. `$TEMP` is a fresh folder.
    pub args: Vec<String>,
    /// Environment variables the test skips without, when its `args` use them (`${NAME}`); any
    /// other unset variable fails the test.
    pub optional: Vec<String>,
    /// Golden tests: only type-check the program (`jaic -check`); it must pass.
    pub check_only: bool,
    /// Golden tests: the build output (compiled) or standard error (interpreted) must contain
    /// this compiler warning.
    pub warns: Option<String>,
    /// Compile-fail tests: the diagnostics need only contain this text, for output that depends
    /// on the host (the C compiler `BuildCpp` found).
    pub diagnostic_contains: Option<String>,
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
            "allow_leaks" => self.allow_leaks = true,
            "only" | "skip_on" | "interp_only" | "native_only" => {
                self.target.apply(note)?;
            }
            "needs" => self.needs = list(),
            "should_fail" => {
                self.should_fail = Some(match args {
                    Some(args) if args.trim_start().starts_with('"') => quoted_list(args),
                    _ => Some(text(args))
                        .filter(|t| !t.is_empty())
                        .into_iter()
                        .collect(),
                });
            }
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
            "optional" => {
                self.optional = args
                    .unwrap_or("")
                    .split(',')
                    .map(|name| name.trim().to_string())
                    .filter(|name| !name.is_empty())
                    .collect();
                if self.optional.is_empty() {
                    return Err(
                        "`optional` needs variable names, as in `optional(JAIC_TEST_FONT_PATH)`"
                            .into(),
                    );
                }
            }
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
                    options.target.only.clear();
                } else if call == "skip_on" && !std::mem::replace(&mut skip_on, true) {
                    options.target.skip_on.clear();
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
        if let Some(reason) = self.target.skip_reason(target) {
            return Some(reason);
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
        assert_eq!(options.target.only.len(), 1);
        assert_eq!(options.target.only[0].names, ["linux", "macos"]);
        assert_eq!(options.should_fail, Some(vec!["bounds".to_string()]));
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
        assert_eq!(
            options.should_fail,
            Some(vec!["Index out of range".to_string()])
        );
        assert_eq!(options.args, ["a b", "$TEMP/Data"]);
        assert_eq!(options.warns.as_deref(), Some("not in scope"));
        assert!(options.check_only);
    }

    #[test]
    fn a_note_with_spaces_in_its_parentheses_applies() {
        let notes = [
            "skip_on(.MACOS, reason = \"needs a real display\")".to_string(),
            "only(.LINUX, .MACOS)".to_string(),
        ];
        let (options, warnings) = TestOptions::default().with_notes(&notes);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(options.target.skip_on.len(), 1);
        assert_eq!(
            options.target.skip_on[0].reason.as_deref(),
            Some("needs a real display")
        );
        assert_eq!(options.target.only[0].names, ["linux", "macos"]);
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
    fn a_reason_is_shown_with_the_skip_and_may_hold_commas_and_parentheses() {
        let mut options = TestOptions::default();
        let notes = super::header_notes(
            "// test: skip_on(.WASM, reason = \"no threads (WASI), no fibers\") serial\n",
        );
        assert_eq!(notes.len(), 2);
        let (with, warnings) = options.with_notes(&notes);
        assert_eq!(warnings, Vec::<String>::new());
        assert_eq!(
            with.skip_reason(&target("wasm", false), &[]).as_deref(),
            Some("skipped on wasm: no threads (WASI), no fibers")
        );
        assert_eq!(with.skip_reason(&target("linux", false), &[]), None);
        assert!(options.apply("skip_on(.WASM, reason = \"\")").is_err());
        assert!(
            options
                .apply("skip_on(.WASM, reason = \"a\", reason = \"b\")")
                .is_err()
        );
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
        assert_eq!(later.target.skip_on.len(), 1);
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
    fn should_fail_takes_one_or_several_texts() {
        let parse = |note: &str| {
            let (options, warnings) = TestOptions::default().with_notes(&[note.to_string()]);
            assert!(warnings.is_empty(), "{warnings:?}");
            options.should_fail
        };
        assert_eq!(parse("should_fail"), Some(vec![]));
        assert_eq!(parse("should_fail(want:)"), Some(vec!["want:".to_string()]));
        assert_eq!(
            parse(r#"should_fail("a, b", "AddressSanitizer")"#),
            Some(vec!["a, b".to_string(), "AddressSanitizer".to_string()])
        );
        // Quotes inside an unquoted text are part of it.
        assert_eq!(
            parse(r#"should_fail(holds("Q"))"#),
            Some(vec![r#"holds("Q")"#.to_string()])
        );
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

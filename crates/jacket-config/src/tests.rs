use super::*;

/// A directory tree in memory.
struct Mem(Vec<(&'static str, &'static str)>);

impl Files for Mem {
    fn is_file(&self, path: &Path) -> bool {
        self.0.iter().any(|(p, _)| Path::new(p) == path)
    }

    fn read(&self, path: &Path) -> std::io::Result<String> {
        self.0
            .iter()
            .find(|(p, _)| Path::new(p) == path)
            .map(|(_, t)| t.to_string())
            .ok_or_else(|| std::io::ErrorKind::NotFound.into())
    }
}

/// Exists, but reading it fails.
struct Unreadable;

impl Files for Unreadable {
    fn is_file(&self, path: &Path) -> bool {
        path.ends_with(FILE_NAME)
    }

    fn read(&self, _: &Path) -> std::io::Result<String> {
        Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "no access for the test",
        ))
    }
}

#[test]
fn an_unreadable_file_is_an_error_not_defaults() {
    let error = load(&Unreadable, Path::new("/p")).unwrap_err();
    assert_eq!(error.path, Path::new("/p").join(FILE_NAME));
    assert!(
        error.message.contains("no access for the test"),
        "{}",
        error.message
    );
}

#[test]
fn the_nearest_directory_wins_and_old_file_names_mean_nothing() {
    let fs = Mem(vec![
        ("/p/jacket.toml", "[project]\nentry = [\"a.jai\"]\n"),
        ("/p/sub/jacket.toml", "[lint]\nmax-nesting = 3\n"),
        ("/q/jai.toml", "build_files = [\"b.jai\"]\n"),
    ]);
    let here = load(&fs, Path::new("/p/deep")).unwrap().unwrap();
    assert_eq!(here.config.project.entry.0, ["a.jai"]);
    assert!(here.notes.is_empty(), "{:?}", here.notes);
    let sub = load(&fs, Path::new("/p/sub/x")).unwrap().unwrap();
    assert_eq!(sub.location.dir, Path::new("/p/sub"));
    assert_eq!(sub.config.lint.max_nesting, Some(3));
    assert!(load(&fs, Path::new("/q")).unwrap().is_none());
}

#[test]
fn entries_follow_the_configured_then_the_inferred_order() {
    let project = Project {
        entry: schema::Strings(vec!["x.jai".into()]),
        ..Project::default()
    };
    let configured = entry_files(Path::new("/p"), Some(&project));
    assert!(configured.declared);
    assert_eq!(configured.paths, [PathBuf::from("/p/x.jai")]);
    let inferred = entry_files(Path::new("/p"), None);
    assert!(!inferred.declared);
    let names: Vec<_> = inferred
        .paths
        .iter()
        .map(|p| p.strip_prefix("/p").unwrap().to_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "build.jai",
            "first.jai",
            "main.jai",
            "src/main.jai",
            "module.jai"
        ]
    );
    let fs = Mem(vec![("/p/module.jai", ""), ("/p/main.jai", "")]);
    assert_eq!(
        first_entry(&fs, Path::new("/p"), None),
        Some(PathBuf::from("/p/main.jai"))
    );
}

#[test]
fn experimental_and_toolchain_parse() {
    let config = parse("[toolchain]\nversion = \"0.20.0\"\n[experimental]\nparse-threads = 3\nserial-parse = true\ncodegen-units = 8\n").unwrap();
    assert_eq!(config.toolchain.version.as_deref(), Some("0.20.0"));
    assert_eq!(config.experimental.parse_threads, Some(3));
    assert_eq!(config.experimental.serial_parse, Some(true));
    assert_eq!(config.experimental.codegen_units, Some(8));
    assert!(parse("[project]\nentry = 3\n").is_err());
    let postgres = parse("[postgres]\nmanaged = true\n").unwrap().postgres;
    assert_eq!(postgres.managed, Some(true));
    assert_eq!(postgres.managed_version, None);
    assert_eq!(line_of("a = 1\n[rules]\n  b = 2", "b"), Some(3));
}

#[test]
fn unknown_keys_are_dropped_and_named_by_path() {
    let (config, unknown) = parse_lenient(
        "[project]\nentry = \"a.jai\"\nflavour = 1\n[toolchain]\nversionn = \"1\"\n\
         [lint]\nmax-nestng = 3\nexclude = [\"x\"]\n[format]\nindent = 2\n\
         [experimental]\nturbo = true\n[extras]\nx = 1\n",
    )
    .unwrap();
    assert_eq!(
        unknown,
        [
            "experimental.turbo",
            "extras",
            "format.indent",
            "lint.max-nestng",
            "project.flavour",
            "toolchain.versionn",
        ]
    );
    assert_eq!(config.project.entry.0, ["a.jai"]);
    assert_eq!(config.lint.exclude[0].as_str(), "x");
    assert_eq!(config.lint.max_nesting, None);
}

#[test]
fn a_nested_unknown_table_is_named_at_its_first_unknown_key() {
    let (_, unknown) =
        parse_lenient("[lint.extra.deep]\nx = 1\n[lint.rules]\nfoo = \"warn\"\n").unwrap();
    assert_eq!(unknown, ["lint.extra"]);
}

#[test]
fn known_settings_load_without_warnings() {
    let (config, unknown) = parse_lenient(
        "[project]\nentry = [\"a.jai\"]\nimport-path = \"v\"\n[toolchain]\nversion = \"1.2.3\"\n\
         [lint]\nmax-nesting = 3\n[lint.rules]\nfloat_equality = \"deny\"\n\
         [format]\nindent-width = 2\n[experimental]\nparse-threads = 4\n",
    )
    .unwrap();
    assert!(unknown.is_empty(), "{unknown:?}");
    assert_eq!(config.project.import_path.0, ["v"]);
    assert_eq!(config.toolchain.version.as_deref(), Some("1.2.3"));
    assert_eq!(config.lint.max_nesting, Some(3));
    assert_eq!(config.lint.rules["float_equality"], "deny");
    assert_eq!(config.format.indent_width, Some(2));
    assert_eq!(config.experimental.parse_threads, Some(4));
}

#[test]
fn the_warning_suggests_the_near_key() {
    assert_eq!(
        unknown_note("lint.max-nestng"),
        "unknown setting `lint.max-nestng` in jacket.toml\nhelp: did you mean `lint.max-nesting`?"
    );
    assert_eq!(
        unknown_note("lint.colour"),
        "unknown setting `lint.colour` in jacket.toml"
    );
    assert!(unknown_note("lnt").ends_with("did you mean `lint`?"));
}

#[test]
fn a_top_level_key_of_a_section_points_at_it() {
    assert_eq!(
        unknown_note("import-path"),
        "unknown setting `import-path` in jacket.toml\nhelp: it belongs under `[project]`"
    );
    assert!(unknown_note("max-nesting").ends_with("it belongs under `[lint]`"));
    // The old snake_case spelling names the new one.
    assert_eq!(
        unknown_note("import_path"),
        "unknown setting `import_path` in jacket.toml\nhelp: use `import-path` under `[project]`"
    );
    assert_eq!(
        unknown_note("package.requires_jacket"),
        "unknown setting `package.requires_jacket` in jacket.toml\nhelp: `requires_jacket` is now `package.jacket-version`"
    );
    assert!(unknown_note("lint.max_nesting").ends_with("did you mean `lint.max-nesting`?"));
    assert_eq!(
        unknown_note("build_files"),
        "unknown setting `build_files` in jacket.toml\nhelp: use `entry` under `[project]`"
    );
    // Only the top level is misplaced; a section's own typo keeps the did-you-mean.
    assert!(!unknown_note("lint.entry").contains("belongs"));
}

#[test]
fn aliases_name_real_keys() {
    for (alias, table, key) in schema::ALIASES {
        assert!(schema::known_keys(table).contains(key), "{alias}");
    }
}

#[test]
fn an_unknown_key_is_an_error_naming_the_file_line_and_nearest_key() {
    let fs = Mem(vec![("/p/jacket.toml", "[lint]\nmax-nestng = 3\n")]);
    let error = load(&fs, Path::new("/p")).unwrap_err();
    assert_eq!(error.path, Path::new("/p/jacket.toml"));
    assert_eq!(
        error.message,
        "unknown setting `lint.max-nestng` in jacket.toml (line 2)\nhelp: did you mean `lint.max-nesting`?"
    );
    let several = parse("[lint]\nmax-nestng = 3\n[format]\nindent = 2\ntyop = 1\n").unwrap_err();
    assert!(several.contains("\nnote: and 2 more: `"), "{several}");
    assert!(parse("[lint]\nmax-nesting = 3\n").is_ok());
}

#[test]
fn known_keys_match_the_structs() {
    let full = toml::to_string(&toml::Value::try_from(Config::default()).unwrap()).unwrap();
    assert!(full.is_empty());
    let sections = [
        ("project", "entry = [\"a\"]\nimport-path = [\"b\"]\n"),
        ("toolchain", "version = \"1\"\n"),
        (
            "lint",
            "exclude = [\"a\"]\nmax-nesting = 1\nmax-complexity = 9\nmax-arguments = 5\nmax-bools = 2\nmax-stack-array-bytes = 4096\nmagic-number-allow = [60, 0.5]\n\
             magic-number-min = 4\nmagic-number-in-tests = true\n[lint.rules]\na = \"b\"\n\
             [[lint.overrides]]\npaths = [\"c\"]\n[lint.overrides.rules]\nd = \"e\"\n\
             [lint.lifetimes]\nmode = \"strict\"\n\
             [lint.freestanding]\nmax-frame-bytes = 512\n\
             [lint.portability]\ntargets = [\"linux\"]\nportable = [\"p/**\"]\n[lint.portability.platforms]\n\"q/**\" = [\"linux\"]\n",
        ),
        (
            "format",
            "indent-width = 1\ncase-indent = 1\ncase-body-indent = 1\nmax-blank-lines = 1\nmax-width = 1\n\
             brace-style = \"a\"\nblank-lines-between-items = true\nblank-line-after-declarations = true\n\
             blank-lines-around-blocks = true\nblank-line-between-cases = true\n\
             blank-line-before-comments = true\nblank-line-after-guards = true\n\
             blank-line-before-return = true\nformat-doc-examples = true\nignore = [\"a\"]\n",
        ),
        (
            "experimental",
            "parse-threads = 1\nserial-parse = true\ncodegen-units = 1\n",
        ),
        (
            "resolver",
            "min-release-age = \"3d\"\nrelease-age-jitter = \"2d\"\nseed = \"k\"\ndedupe = true\n",
        ),
        (
            "package",
            "name = \"a\"\nversion = \"1\"\nmodule = \"src\"\njacket-version = \">=0.40\"\n\
             platforms = [\"macos\"]\nimport-path = [\"x\"]\n",
        ),
        ("postgres", "managed = true\nmanaged-version = \"17.6.0\"\n"),
        (
            "test",
            "paths = [\"a\"]\njobs = 2\ntimeout = 5\nslow = 1\nretries = 1\nfail-fast = true\n\
             flaky-fails = true\nrunner = \"node\"\nneeds = [\"network\"]\n\
             [test.profile.ci]\njobs = 1\n",
        ),
        (
            "doc",
            "url = \"n\"\nog-image = \"o\"\nsite-name = \"p\"\ncanonical-version = \"q\"\ntheme = \"r\"\n\
             path = \"a\"\nname = \"b\"\nbreadcrumb = \"m\"\ndescription = \"c\"\nlogo = \"d\"\nicon = \"v\"\nhome = \"w\"\nrepo = \"e\"\nplayground = \"f\"\n\
             since = \"g\"\npages-title = \"h\"\nfiles = [\"u\"]\n[[doc.category]]\nname = \"i\"\n[[doc.group]]\nname = \"j\"\n\
             [[doc.page]]\nfile = \"k\"\nname = \"l\"\n[[doc.section]]\nfile = \"s\"\nname = \"t\"\n\
             [doc.format]\nindent-width = 2\n",
        ),
    ];
    let mut tops: Vec<&str> = Vec::new();
    for (name, body) in sections {
        let config = parse(&format!("[{name}]\n{body}")).unwrap();
        let value = toml::Value::try_from(&config).unwrap();
        let table = value[name].as_table().unwrap();
        let mut keys: Vec<&str> = table.keys().map(String::as_str).collect();
        let mut known = schema::known_keys(name).to_vec();
        keys.sort_unstable();
        known.sort_unstable();
        assert_eq!(keys, known, "[{name}]");
        tops.push(name);
    }
    let config =
        parse("[target.pico]\ntriple = \"t\"\nboard = \"b\"\nrunner = \"r {exe}\"\n").unwrap();
    let value = toml::Value::try_from(&config).unwrap();
    let mut keys: Vec<&str> = value["target"]["pico"]
        .as_table()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    let mut known = schema::known_keys("target.pico").to_vec();
    keys.sort_unstable();
    known.sort_unstable();
    assert_eq!(keys, known, "[target.<name>]");
    tops.push("target");
    let config = parse(
        "[dependencies.d]\ngit = \"a/b\"\nversion = \"^1\"\ntag = \"t\"\nrev = \"r\"\npath = \"p\"\n\
         module = \"m\"\nscheme = \"semver\"\ntag-prefix = \"v\"\nmin-release-age = \"0\"\nshared = true\n",
    )
    .unwrap();
    let value = toml::Value::try_from(&config).unwrap();
    let mut keys: Vec<&str> = value["dependencies"]["d"]
        .as_table()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    let mut known = schema::known_keys("dependencies.d").to_vec();
    keys.sort_unstable();
    known.sort_unstable();
    assert_eq!(keys, known, "[dependencies.<name>]");
    tops.push("dependencies");
    let mut known = schema::known_keys("").to_vec();
    tops.sort_unstable();
    known.sort_unstable();
    assert_eq!(tops, known);
}

#[test]
fn test_profiles_lay_over_the_test_section() {
    let (config, unknown) = parse_lenient(
        "[test]\njobs = 4\nretries = 1\n[test.profile.ci]\nretries = 3\nfail-fast = true\njbos = 2\n",
    )
    .unwrap();
    assert_eq!(unknown, ["test.profile.ci.jbos"]);
    assert!(unknown_note(&unknown[0]).contains("did you mean `test.profile.ci.jobs`?"));
    let (_, unknown) = parse_lenient("[test]\njbos = 2\n").unwrap();
    assert_eq!(unknown, ["test.jbos"]);
    let ci = config.test.run().overlaid(&config.test.profile["ci"]);
    assert_eq!(
        (ci.jobs, ci.retries, ci.fail_fast),
        (Some(4), Some(3), Some(true))
    );
    let all = TestRun {
        jobs: Some(1),
        timeout: Some(1),
        slow: Some(1),
        retries: Some(1),
        fail_fast: Some(true),
        flaky_fails: Some(true),
        runner: Some("r".into()),
        needs: vec!["n".into()],
    };
    let value = toml::Value::try_from(all).unwrap();
    let mut keys: Vec<&str> = value
        .as_table()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    let mut known = schema::known_keys("test.profile.x").to_vec();
    keys.sort_unstable();
    known.sort_unstable();
    assert_eq!(keys, known);
}

#[test]
fn checks_run_at_load_and_their_problems_are_recorded_not_fatal() {
    fn max_nesting_is_small(config: &Config, dir: &Path) -> Result<(), String> {
        match config.lint.max_nesting {
            Some(n) if n > 3 => Err(format!(
                "`max-nesting` {n} in {} is too deep",
                dir.display()
            )),
            _ => Ok(()),
        }
    }
    let files = Mem(vec![("/p/jacket.toml", "[lint]\nmax-nesting = 9\n")]);
    let loaded = load_checked(&files, Path::new("/p"), &[max_nesting_is_small])
        .unwrap()
        .unwrap();
    assert_eq!(loaded.problems, ["`max-nesting` 9 in /p is too deep"]);
    assert_eq!(loaded.config.lint.max_nesting, Some(9));
    let unchecked = load(&files, Path::new("/p")).unwrap().unwrap();
    assert!(unchecked.problems.is_empty());
}

#[test]
fn only_jacket_toml_is_a_settings_file() {
    assert!(is_config_file(Path::new("/p/jacket.toml")));
    assert!(!is_config_file(Path::new("/p/other.toml")));
}

#[test]
fn doc_format_overrides_only_what_it_sets() {
    let (config, unknown) = parse_lenient(
        "[format]\nblank-line-after-declarations = true\nindent-width = 2\n\
         [doc.format]\nblank-line-after-declarations = false\nbrace-style = \"preserve\"\n",
    )
    .unwrap();
    assert!(unknown.is_empty(), "{unknown:?}");
    config.doc.format.validate_doc().unwrap();
    assert_eq!(
        config.doc.format.overrides(),
        [
            ("brace-style", "\"preserve\"".to_string()),
            ("blank-line-after-declarations", "false".to_string()),
        ]
    );
    assert_eq!(config.format.indent_width, Some(2));
    assert!(
        parse("[format]\nindent-width = 2\n")
            .unwrap()
            .doc
            .format
            .overrides()
            .is_empty()
    );
}

#[test]
fn doc_format_is_checked_like_format() {
    let bad = parse("[doc.format]\nindent-width = 17\n").unwrap();
    assert_eq!(
        bad.doc.format.validate_doc().unwrap_err(),
        "indent-width must be an integer from 1 to 16"
    );
    let ignore = parse("[doc.format]\nignore = [\"a\"]\n").unwrap();
    assert_eq!(
        ignore.doc.format.validate_doc().unwrap_err(),
        "doc.format cannot set ignore"
    );
    let examples = parse("[doc.format]\nformat-doc-examples = false\n").unwrap();
    assert_eq!(
        examples.doc.format.validate_doc().unwrap_err(),
        "doc.format cannot set format-doc-examples"
    );
    let (_, unknown) = parse_lenient("[doc.format]\nindent-widht = 2\n").unwrap();
    assert_eq!(unknown, ["doc.format.indent-widht"]);
    assert_eq!(
        unknown_note("doc.format.indent-widht"),
        "unknown setting `doc.format.indent-widht` in jacket.toml\nhelp: did you mean `doc.format.indent-width`?"
    );
}

/// A target table says what the project is, never how one machine builds it: a key that names a
/// linker, archiver, sysroot or builtins file is unknown, so `jacket.toml` stays portable
/// (`docs/native/toolchain-discovery.md`).
#[test]
fn a_target_table_has_no_keys_that_name_a_machines_tools() {
    let text = r#"
[target.pi]
triple = "armv7-unknown-linux-gnueabihf"
linker = "arm-linux-gnueabihf-gcc"
ar = "llvm-ar"
sysroot = "/opt/sysroot"
compiler_rt = "/opt/rt.a"
"#;
    let (config, unknown) = parse_lenient(text).unwrap();
    assert_eq!(
        unknown,
        [
            "target.pi.ar",
            "target.pi.compiler_rt",
            "target.pi.linker",
            "target.pi.sysroot"
        ]
    );
    let target = serde_json::to_value(&config.target["pi"]).unwrap();
    let mut keys: Vec<_> = target.as_object().unwrap().keys().cloned().collect();
    keys.sort();
    assert_eq!(keys, ["triple"]);
}

/// Every key a target table accepts, written out: the settings that mean something on any
/// machine.
#[test]
fn a_target_table_accepts_only_portable_settings() {
    let full = schema::Target {
        triple: Some("t".into()),
        board: Some("b".into()),
        runner: Some("r {sysroot} {exe}".into()),
        stack_size: Some(1),
        entry: Some("e".into()),
        gc_sections: Some(true),
    };
    let value = serde_json::to_value(&full).unwrap();
    let mut keys: Vec<_> = value.as_object().unwrap().keys().cloned().collect();
    keys.sort();
    assert_eq!(
        keys,
        [
            "board",
            "entry",
            "gc-sections",
            "runner",
            "stack-size",
            "triple"
        ]
    );
}

#[test]
fn dependencies_read_inline_tables_and_the_resolver_policy() {
    let (config, unknown) = parse_lenient(
        "[dependencies]\njaison = { git = \"github.com/someone/jaison\", version = \"^1.2\" }\n\
         utf8 = { git = \"x/utf8\", tag = \"v0.4.1\", min-release-age = \"0\", tag-prefix = \"v\" }\n\
         [resolver]\nmin-release-age = \"3d\"\nseed = \"k3v9q2xh\"\nddedupe = true\n",
    )
    .unwrap();
    assert_eq!(unknown, ["resolver.ddedupe"]);
    assert_eq!(
        config.dependencies["jaison"].version.as_deref(),
        Some("^1.2")
    );
    assert_eq!(
        config.dependencies["utf8"].min_release_age.as_deref(),
        Some("0")
    );
    assert_eq!(config.resolver.seed.as_deref(), Some("k3v9q2xh"));
    assert!(unknown_note(&unknown[0]).contains("did you mean `resolver.dedupe`?"));
}

fn slice<'a>(text: &'a str, issue: &Issue) -> &'a str {
    &text[issue.span.clone().expect("the issue has a place")]
}

#[test]
fn an_unknown_key_is_an_issue_at_the_key() {
    let text = "[lint]\nmax-nestng = 3\n[resolver]\nseeed = \"k\"\n";
    let found = issues(text, Path::new("/p"), &[]);
    assert_eq!(found.len(), 2);
    assert_eq!(slice(text, &found[0]), "max-nestng");
    assert!(
        found[0]
            .message
            .contains("did you mean `lint.max-nesting`?")
    );
    assert_eq!(slice(text, &found[1]), "seeed");
}

#[test]
fn a_syntax_or_type_error_is_an_issue_at_the_fault() {
    let text = "[lint]\nmax-nesting = \"deep\"\n";
    let found = issues(text, Path::new("/p"), &[]);
    assert_eq!(found.len(), 1);
    assert_eq!(slice(text, &found[0]), "\"deep\"");
    let text = "[lint\nmax-nesting = 3\n";
    let found = issues(text, Path::new("/p"), &[]);
    assert_eq!(found.len(), 1);
    assert!(found[0].span.is_some());
}

#[test]
fn a_failed_check_is_an_issue_at_the_key_it_names() {
    fn rules(config: &Config, _: &Path) -> Result<(), String> {
        match config.lint.rules.keys().next() {
            Some(name) => Err(format!("unknown rule `{name}`")),
            None => Ok(()),
        }
    }
    let text = "[lint.rules]\nunused_varible = \"deny\"\n";
    let found = issues(text, Path::new("/p"), &[rules]);
    assert_eq!(found.len(), 1);
    assert_eq!(slice(text, &found[0]), "unused_varible");
    assert_eq!(found[0].message, "unknown rule `unused_varible`");
}

#[test]
fn a_valid_file_has_no_issues() {
    assert!(issues("[project]\nentry = [\"main.jai\"]\n", Path::new("/p"), &[]).is_empty());
}

#[test]
fn every_key_has_a_description() {
    for table in schema::tables() {
        for key in schema::known_keys(&table) {
            assert!(
                schema::describe(&table, key).is_some(),
                "`{table}` `{key}` has no description"
            );
        }
    }
}

#[test]
fn a_description_is_found_through_a_table_name_that_stands_for_many() {
    assert!(schema::describe("dependencies.jaison", "git").is_some());
    assert!(schema::describe("test.profile.ci", "jobs").is_some());
    assert!(schema::describe("target.pico", "triple").is_some());
    assert!(schema::describe("lint", "no-such-key").is_none());
}

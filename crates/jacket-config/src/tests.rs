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
        ("/p/sub/jacket.toml", "[lint]\nmax_nesting = 3\n"),
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
    let config = parse("[toolchain]\nversion = \"0.20.0\"\n[experimental]\nparse_threads = 3\nserial_parse = true\ncodegen_units = 8\n").unwrap();
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
    let (config, unknown) = parse_noting(
        "[project]\nentry = \"a.jai\"\nflavour = 1\n[toolchain]\nversionn = \"1\"\n\
         [lint]\nmax_nestng = 3\nexclude = [\"x\"]\n[format]\nindent = 2\n\
         [experimental]\nturbo = true\n[extras]\nx = 1\n",
    )
    .unwrap();
    assert_eq!(
        unknown,
        [
            "experimental.turbo",
            "extras",
            "format.indent",
            "lint.max_nestng",
            "project.flavour",
            "toolchain.versionn",
        ]
    );
    assert_eq!(config.project.entry.0, ["a.jai"]);
    assert_eq!(config.lint.exclude, ["x"]);
    assert_eq!(config.lint.max_nesting, None);
}

#[test]
fn a_nested_unknown_table_is_named_at_its_first_unknown_key() {
    let (_, unknown) =
        parse_noting("[lint.extra.deep]\nx = 1\n[lint.rules]\nfoo = \"warn\"\n").unwrap();
    assert_eq!(unknown, ["lint.extra"]);
}

#[test]
fn known_settings_load_without_warnings() {
    let (config, unknown) = parse_noting(
        "[project]\nentry = [\"a.jai\"]\nimport_path = \"v\"\n[toolchain]\nversion = \"1.2.3\"\n\
         [lint]\nmax_nesting = 3\n[lint.rules]\nfloat_equality = \"deny\"\n\
         [format]\nindent_width = 2\n[experimental]\nparse_threads = 4\n",
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
        unknown_note("lint.max_nestng"),
        "warning: unknown setting `lint.max_nestng` in jacket.toml (ignored); did you mean `lint.max_nesting`?"
    );
    assert_eq!(
        unknown_note("lint.colour"),
        "warning: unknown setting `lint.colour` in jacket.toml (ignored)"
    );
    assert!(unknown_note("lnt").ends_with("did you mean `lint`?"));
}

#[test]
fn a_top_level_key_of_a_section_points_at_it() {
    assert_eq!(
        unknown_note("import_path"),
        "warning: unknown setting `import_path` in jacket.toml (ignored); it belongs under `[project]`"
    );
    assert!(unknown_note("max_nesting").ends_with("it belongs under `[lint]`"));
    assert_eq!(
        unknown_note("build_files"),
        "warning: unknown setting `build_files` in jacket.toml (ignored); use `entry` under `[project]`"
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
fn load_reports_unknown_keys_as_notes() {
    let fs = Mem(vec![("/p/jacket.toml", "[lint]\nmax_nestng = 3\n")]);
    let loaded = load(&fs, Path::new("/p")).unwrap().unwrap();
    assert_eq!(loaded.notes, [unknown_note("lint.max_nestng")]);
}

#[test]
fn known_keys_match_the_structs() {
    let full = toml::to_string(&toml::Value::try_from(Config::default()).unwrap()).unwrap();
    assert!(full.is_empty());
    let sections = [
        ("project", "entry = [\"a\"]\nimport_path = [\"b\"]\n"),
        ("toolchain", "version = \"1\"\n"),
        (
            "lint",
            "exclude = [\"a\"]\nmax_nesting = 1\nmax_complexity = 9\nmax_arguments = 5\nmax_bools = 2\nmax_stack_array_bytes = 4096\nmagic_number_allow = [60, 0.5]\n\
             magic_number_min = 4\nmagic_number_in_tests = true\n[lint.rules]\na = \"b\"\n\
             [[lint.overrides]]\npaths = [\"c\"]\n[lint.overrides.rules]\nd = \"e\"\n\
             [lint.portability]\ntargets = [\"linux\"]\nportable = [\"p/**\"]\n[lint.portability.platforms]\n\"q/**\" = [\"linux\"]\n",
        ),
        (
            "format",
            "indent_width = 1\ncase_indent = 1\ncase_body_indent = 1\nmax_blank_lines = 1\nmax_width = 1\n\
             brace_style = \"a\"\nblank_lines_between_items = true\nblank_line_after_declarations = true\n\
             blank_lines_around_blocks = true\nblank_line_between_cases = true\n\
             blank_line_before_comments = true\nblank_line_after_guards = true\n\
             blank_line_before_return = true\nignore = [\"a\"]\n",
        ),
        (
            "experimental",
            "parse_threads = 1\nserial_parse = true\ncodegen_units = 1\n",
        ),
        ("postgres", "managed = true\nmanaged_version = \"17.6.0\"\n"),
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
    let mut known = schema::known_keys("").to_vec();
    tops.sort_unstable();
    known.sort_unstable();
    assert_eq!(tops, known);
}

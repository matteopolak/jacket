use super::*;

fn edit(start: usize, end: usize, text: &str) -> (Edit, &'static str) {
    (
        Edit {
            start,
            end,
            text: text.into(),
        },
        "m",
    )
}

#[test]
fn overlapping_edits_wait_for_the_next_pass() {
    let chosen = non_overlapping(vec![edit(4, 8, "x"), edit(0, 5, "y"), edit(9, 9, "z")]);
    let starts: Vec<usize> = chosen.iter().map(|(e, _)| e.start).collect();

    assert_eq!(starts, [0, 9]);
}

#[test]
fn offsets_map_back_through_earlier_edits() {
    // "Context" (7 bytes) became "#Context" (8) at 10; a later edit at 30 was at 29.
    let shifts = [(10, 1)];

    assert_eq!(original_offset(&shifts, 30), 29);
    assert_eq!(original_offset(&shifts, 5), 5);
}

#[test]
fn errors_are_matched_by_file_code_and_message() {
    let e = |start, message: &str| CompileError {
        path: "a.jai".into(),
        start,
        end: start + 1,
        code: "unknown-identifier",
        message: message.into(),
    };
    let before = [
        e(3, "unknown identifier `x`"),
        e(9, "unknown identifier `y`"),
    ];
    let after = [
        e(4, "unknown identifier `x`"),
        e(1, "unknown identifier `z`"),
    ];
    let (introduced, remaining) = compare(&before, &after);

    assert_eq!(introduced, [e(1, "unknown identifier `z`")]);
    assert_eq!(remaining, [e(4, "unknown identifier `x`")]);
}

#[test]
fn ambiguous_migrations_run_only_when_named() {
    let all = Selection::default().migrations().unwrap();

    assert!(all.iter().all(|m| !m.ambiguous));
    let err = Selection {
        only: Some(vec!["context-type-directiv".into()]),
    }
    .migrations()
    .err()
    .unwrap();

    assert!(
        err.contains("did you mean `context-type-directive`"),
        "{err}"
    );
}

#[test]
fn migration_names_are_unique_kebab_case() {
    let mut names: Vec<&str> = MIGRATIONS.iter().map(|m| m.name).collect();

    for name in &names {
        assert!(
            name.bytes()
                .all(|b| b.is_ascii_lowercase() || b == b'-' || b.is_ascii_digit()),
            "{name}"
        );
    }

    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), MIGRATIONS.len());
}

/// Rewrites every integer literal `42` to a name that does not exist.
static BREAKS_CODE: Migration = Migration {
    name: "breaks-code",
    summary: "test",
    ambiguous: false,
    detect: Detect::Tokens(break_forty_two),
};

/// Edits that replace each token reading `word` with `new`.
fn replace_word(source: &Source, word: &str, new: &str) -> Vec<Answer> {
    source
        .tokens
        .iter()
        .filter(|t| source.slice(t.span.start as usize, t.span.end as usize) == word)
        .map(|t| Answer::Edit {
            start: t.span.start as usize,
            end: t.span.end as usize,
            text: new.into(),
        })
        .collect()
}

fn break_forty_two(source: &Source) -> Vec<Answer> {
    replace_word(source, "42", "no_such_name")
}

/// Renames the procedure `before` to `after`.
static RENAMES_BEFORE: Migration = Migration {
    name: "renames-before",
    summary: "test",
    ambiguous: false,
    detect: Detect::Tokens(rename_before),
};

fn rename_before(source: &Source) -> Vec<Answer> {
    replace_word(source, "before", "after")
}

fn options_for(path: &Path) -> Options {
    Options {
        compiler_options: jaic::sema::Options::host(),
        source: None,
        paths: vec![path.to_path_buf()],
        import_dirs: Vec::new(),
        stdlib: Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"),
        config: crate::Config::default(),
        jobs: 1,
        verbose: false,
        add_sources: Vec::new(),
        limits: jaic::ide::lint_compile::CheckLimits::default(),
        overlay: BTreeMap::new(),
    }
}

#[test]
fn a_migration_that_adds_an_error_is_left_out_and_the_rest_still_apply() {
    let dir = test_support::TempDir::new("migrate-partial");

    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("main.jai");

    std::fs::write(
        &file,
        "before :: () {}\nmain :: () {\n    x := 42;\n    before();\n}\n",
    )
    .unwrap();
    let file = std::fs::canonicalize(&file).unwrap();
    let plan = plan_with(options_for(&file), &[&BREAKS_CODE, &RENAMES_BEFORE]).unwrap();

    assert_eq!(plan.rejected.len(), 1, "{plan:?}");
    assert_eq!(plan.rejected[0].migration, "breaks-code");
    assert_eq!(plan.rejected[0].errors[0].line, 3);
    let [migrated] = plan.files.as_slice() else {
        panic!("{plan:?}");
    };

    assert_eq!(
        migrated.after,
        "after :: () {}\nmain :: () {\n    x := 42;\n    after();\n}\n"
    );
}

#[test]
fn a_file_that_is_not_utf8_is_skipped_and_the_others_are_migrated() {
    let dir = test_support::TempDir::new("migrate-not-utf8");

    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("main.jai"),
        "#load \"other.jai\";\nbefore :: () {}\nmain :: () { before(); }\n",
    )
    .unwrap();
    std::fs::write(dir.join("other.jai"), b"x :: \"\xff\";\n").unwrap();
    let main = std::fs::canonicalize(dir.join("main.jai")).unwrap();
    let plan = plan_with(options_for(&main), &[&RENAMES_BEFORE]).unwrap();

    assert_eq!(plan.skipped.len(), 1, "{plan:?}");
    assert!(plan.skipped[0].ends_with("other.jai"));
    assert_eq!(plan.files.len(), 1, "{plan:?}");
}

#[test]
fn a_file_is_lexed_again_only_when_its_text_changes() {
    let mut cache = TokenCache::default();
    let path = Path::new("a.jai");
    let first = cache.tokens(path, "x := 1;").len();

    assert_eq!(cache.tokens(path, "x := 1;").len(), first);
    assert_eq!(cache.lexed, 1, "an unchanged file keeps its tokens");
    // Another file has tokens of its own.
    cache.tokens(Path::new("b.jai"), "x := 1;");
    assert_eq!(cache.lexed, 2);
    // A rewrite that changes the text makes the next lookup lex it afresh.
    assert!(cache.tokens(path, "x := 1; y := 2;").len() > first);
    assert_eq!(cache.lexed, 3);
}

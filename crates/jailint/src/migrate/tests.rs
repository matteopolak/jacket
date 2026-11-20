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

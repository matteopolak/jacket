use super::*;

#[test]
fn every_code_has_a_name_of_its_own() {
    let mut names: Vec<_> = Code::ALL.iter().map(|c| c.name()).collect();
    names.sort_unstable();
    let before = names.len();
    names.dedup();
    assert_eq!(names.len(), before, "two codes share a name");
    assert!(names.iter().all(|n| {
        !n.is_empty()
            && !n.starts_with('-')
            && !n.ends_with('-')
            && n.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    }));
}

#[test]
fn names_parse_back_to_their_codes() {
    for &code in Code::ALL {
        assert_eq!(Code::parse(code.name()), Some(code));
    }
    assert_eq!(Code::parse("no-such-code"), None);
}

#[test]
fn leniencies_are_warnings() {
    for code in Code::LENIENCIES {
        assert_eq!(code.severity(), Severity::Warning, "{code}");
    }
}

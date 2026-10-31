use super::api::{CPU_TAGS, OS_TAGS, PlatformTarget};

const PLATFORM: &str = include_str!("../../../../stdlib/prelude/platform.jai");

/// The member names of `name :: enum ... { ... }` in the prelude, in declaration order.
fn enum_members(name: &str) -> Vec<&'static str> {
    let header = format!("{name} :: enum");
    let body = &PLATFORM[PLATFORM.find(&header).expect("enum is declared")..];
    let body = &body[body.find('{').unwrap() + 1..];
    let body = &body[..body.find('}').unwrap()];
    body.lines()
        .filter_map(|line| line.split("::").next())
        .map(str::trim)
        .filter(|member| !member.is_empty())
        .collect()
}

#[test]
fn tag_tables_match_the_prelude_enums() {
    assert_eq!(OS_TAGS, enum_members("Operating_System_Tag"));
    assert_eq!(CPU_TAGS, enum_members("CPU_Tag"));
}

#[test]
fn compare_decides_known_tags_only() {
    let target = PlatformTarget {
        os: "LINUX",
        cpu: "X64",
    };
    assert_eq!(target.compare("OS", "LINUX", true), Some(true));
    assert_eq!(target.compare("OS", "LINUX", false), Some(false));
    assert_eq!(target.compare("CPU", "ARM64", true), Some(false));
    assert_eq!(target.compare("OS", "BOGUS", true), None);
    assert_eq!(target.compare("ARCH", "X64", true), None);
}

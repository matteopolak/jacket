//! Version schemes, tag parsing and ranges.
//!
//! Tags come from `git ls-remote --tags`. A scheme decides which tags are versions and how they
//! order; a range picks among them. Semver uses Cargo's range spelling (`^1.2`, `~1.2.3`,
//! `>=1.0, <2.0`, `=1.4.0`, `*`); calendar and integer versions take only comparisons.
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Scheme {
    /// `1.2.3`, `v1.2.3`, `1.2.3-rc.1`; two-part tags read as `x.y.0`.
    #[default]
    Semver,
    /// `2024.05.01`, `2024.5`, `24.05`, compared field by field.
    Calendar,
    /// `42`, `r42`, `v42`.
    Integer,
}

impl Scheme {
    pub fn parse(text: &str) -> Result<Self, String> {
        match text {
            "semver" => Ok(Self::Semver),
            "calendar" => Ok(Self::Calendar),
            "integer" => Ok(Self::Integer),
            other => Err(format!(
                "unknown scheme `{other}`: use `semver`, `calendar` or `integer`"
            )),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Version {
    Semver(semver::Version),
    /// Calendar fields, or the one integer.
    Parts(Vec<u64>),
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Semver(a), Self::Semver(b)) => a.cmp(b),
            (Self::Parts(a), Self::Parts(b)) => cmp_parts(a, b),
            (Self::Semver(_), Self::Parts(_)) => Ordering::Less,
            (Self::Parts(_), Self::Semver(_)) => Ordering::Greater,
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn cmp_parts(a: &[u64], b: &[u64]) -> Ordering {
    (0..a.len().max(b.len()))
        .map(|i| {
            a.get(i)
                .copied()
                .unwrap_or(0)
                .cmp(&b.get(i).copied().unwrap_or(0))
        })
        .find(|o| o.is_ne())
        .unwrap_or(Ordering::Equal)
}

impl Version {
    pub fn is_prerelease(&self) -> bool {
        matches!(self, Self::Semver(v) if !v.pre.is_empty())
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Semver(v) => write!(f, "{v}"),
            Self::Parts(p) => {
                let text: Vec<String> = p.iter().map(u64::to_string).collect();
                write!(f, "{}", text.join("."))
            }
        }
    }
}

fn numbers(text: &str) -> Option<Vec<u64>> {
    text.split('.')
        .map(|part| {
            (!part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
                .then(|| part.parse().ok())
                .flatten()
        })
        .collect()
}

/// The version a tag names under `scheme`, after stripping `prefix`; `None` for a tag that is
/// not one.
pub fn parse_tag(scheme: Scheme, prefix: Option<&str>, tag: &str) -> Option<Version> {
    let rest = match prefix {
        Some(prefix) => tag.strip_prefix(prefix)?,
        None => tag,
    };
    match scheme {
        Scheme::Semver => {
            let rest = rest.strip_prefix(['v', 'V']).unwrap_or(rest);
            let (core, tail) = rest.split_at(rest.find(['-', '+']).unwrap_or(rest.len()));
            let fields = numbers(core)?;
            if !(2..=3).contains(&fields.len()) {
                return None;
            }
            let full = format!(
                "{}{tail}",
                (0..3)
                    .map(|i| fields.get(i).copied().unwrap_or(0).to_string())
                    .collect::<Vec<_>>()
                    .join(".")
            );
            semver::Version::parse(&full).ok().map(Version::Semver)
        }
        Scheme::Calendar => {
            let rest = rest.strip_prefix(['v', 'V']).unwrap_or(rest);
            let fields = numbers(rest)?;
            (2..=4)
                .contains(&fields.len())
                .then_some(Version::Parts(fields))
        }
        Scheme::Integer => {
            let rest = rest.strip_prefix(['v', 'V', 'r', 'R']).unwrap_or(rest);
            let fields = numbers(rest)?;
            (fields.len() == 1).then_some(Version::Parts(fields))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Eq,
    Gt,
    Ge,
    Lt,
    Le,
}

#[derive(Clone, Debug)]
pub enum Range {
    Semver(semver::VersionReq),
    /// Comparisons that must all hold; empty is `*`.
    Compare(Vec<(Op, Vec<u64>)>),
}

pub fn parse_range(scheme: Scheme, text: &str) -> Result<Range, String> {
    let text = text.trim();
    match scheme {
        Scheme::Semver => semver::VersionReq::parse(text)
            .map(Range::Semver)
            .map_err(|e| format!("`{text}` is not a version range: {e}")),
        Scheme::Calendar | Scheme::Integer => {
            if text == "*" {
                return Ok(Range::Compare(Vec::new()));
            }
            let mut all = Vec::new();
            for part in text.split(',') {
                let part = part.trim();
                let (op, rest) = [
                    (">=", Op::Ge),
                    ("<=", Op::Le),
                    (">", Op::Gt),
                    ("<", Op::Lt),
                    ("=", Op::Eq),
                ]
                .into_iter()
                .find_map(|(sign, op)| part.strip_prefix(sign).map(|r| (op, r.trim())))
                .unwrap_or((Op::Eq, part));
                let rest = rest.strip_prefix(['v', 'r']).unwrap_or(rest);
                let fields = numbers(rest).ok_or_else(|| {
                    format!(
                        "`{part}` is not a comparison: calendar and integer versions take `>=`, `>`, `<=`, `<`, `=` and `*`"
                    )
                })?;
                all.push((op, fields));
            }
            Ok(Range::Compare(all))
        }
    }
}

impl Range {
    pub fn matches(&self, version: &Version) -> bool {
        match (self, version) {
            (Self::Semver(req), Version::Semver(v)) => req.matches(v),
            (Self::Compare(all), Version::Parts(parts)) => all.iter().all(|(op, wanted)| {
                let order = cmp_parts(parts, wanted);
                match op {
                    Op::Eq => order.is_eq(),
                    Op::Gt => order.is_gt(),
                    Op::Ge => order.is_ge(),
                    Op::Lt => order.is_lt(),
                    Op::Le => order.is_le(),
                }
            }),
            _ => false,
        }
    }
}

/// A tag read as a version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tagged {
    pub tag: String,
    pub commit: String,
    pub version: Version,
}

/// The tags that are versions under the scheme, newest first. Ties (two tags naming one
/// version, as `1.0` and `v1.0.0` do) keep the order they came in.
pub fn versions(
    scheme: Scheme,
    prefix: Option<&str>,
    tags: &[super::git::RemoteTag],
) -> Vec<Tagged> {
    let mut found: Vec<Tagged> = tags
        .iter()
        .filter_map(|t| {
            parse_tag(scheme, prefix, &t.name).map(|version| Tagged {
                tag: t.name.clone(),
                commit: t.commit.clone(),
                version,
            })
        })
        .collect();
    found.sort_by(|a, b| b.version.cmp(&a.version));
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dep::git::RemoteTag;

    fn v(scheme: Scheme, prefix: Option<&str>, tag: &str) -> Option<String> {
        parse_tag(scheme, prefix, tag).map(|v| v.to_string())
    }

    #[test]
    fn semver_reads_prefixes_two_part_tags_and_prereleases() {
        let s = Scheme::Semver;
        assert_eq!(v(s, None, "1.2.3").as_deref(), Some("1.2.3"));
        assert_eq!(v(s, None, "v1.2.3").as_deref(), Some("1.2.3"));
        assert_eq!(v(s, None, "0.8").as_deref(), Some("0.8.0"));
        assert_eq!(v(s, None, "v2.1").as_deref(), Some("2.1.0"));
        assert_eq!(v(s, None, "1.2.3-rc.1").as_deref(), Some("1.2.3-rc.1"));
        assert_eq!(
            v(s, Some("alpha-"), "alpha-0.0.1").as_deref(),
            Some("0.0.1")
        );
        assert_eq!(
            v(s, Some("version-"), "version-1.0").as_deref(),
            Some("1.0.0")
        );
        for bad in [
            "latest",
            "v1",
            "release-1.2",
            "1.2.3.4",
            "1..2",
            "nightly-2024",
        ] {
            assert_eq!(v(s, None, bad), None, "{bad}");
        }
        assert_eq!(
            v(s, Some("alpha-"), "1.0.0"),
            None,
            "the prefix is required when set"
        );
    }

    #[test]
    fn calendar_and_integer_versions_compare_field_by_field() {
        let c = Scheme::Calendar;
        assert!(parse_tag(c, None, "2024.5").unwrap() < parse_tag(c, None, "2024.05.01").unwrap());
        assert!(parse_tag(c, None, "24.05").unwrap() < parse_tag(c, None, "24.10").unwrap());
        assert_eq!(v(c, None, "v1"), None);
        let i = Scheme::Integer;
        for tag in ["42", "r42", "v42"] {
            assert_eq!(v(i, None, tag).as_deref(), Some("42"), "{tag}");
        }
        assert!(parse_tag(i, None, "r9").unwrap() < parse_tag(i, None, "r10").unwrap());
        assert_eq!(v(i, None, "1.2"), None);
    }

    #[test]
    fn semver_ranges_follow_cargo_including_prerelease_rules() {
        let s = Scheme::Semver;
        let pick = |range: &str, tag: &str| {
            parse_range(s, range)
                .unwrap()
                .matches(&parse_tag(s, None, tag).unwrap())
        };
        assert!(pick("^1.2", "1.9.0") && !pick("^1.2", "2.0.0") && !pick("^1.2", "1.1.9"));
        assert!(pick("~1.2.3", "1.2.9") && !pick("~1.2.3", "1.3.0"));
        assert!(pick(">=1.0, <2.0", "1.5.0") && !pick(">=1.0, <2.0", "2.0.0"));
        assert!(pick("=1.4.0", "1.4.0") && !pick("=1.4.0", "1.4.1"));
        assert!(pick("*", "7.0.0") && !pick("*", "7.0.0-rc.1"));
        assert!(
            !pick("^1.2", "1.3.0-rc.1"),
            "prereleases only when the range names one"
        );
        assert!(pick("^1.3.0-rc.1", "1.3.0-rc.2"));
        assert!(pick("^0.4.1", "0.4.9") && !pick("^0.4.1", "0.5.0"));
        assert!(parse_range(s, "latest").is_err());
    }

    #[test]
    fn calendar_ranges_take_comparisons_only() {
        let c = Scheme::Calendar;
        let range = parse_range(c, ">=2024.1, <2025.1").unwrap();
        assert!(range.matches(&parse_tag(c, None, "2024.05.01").unwrap()));
        assert!(!range.matches(&parse_tag(c, None, "2025.01.01").unwrap()));
        assert!(parse_range(c, "^2024.1").is_err());
        assert!(
            parse_range(Scheme::Integer, "*")
                .unwrap()
                .matches(&Version::Parts(vec![3]))
        );
        assert!(Scheme::parse("roman").is_err());
    }

    #[test]
    fn versions_are_newest_first_and_skip_what_is_not_one() {
        let tags: Vec<RemoteTag> = ["v1.2.0", "docs", "v1.10.0", "v1.9.1", "1.10.0-rc.1"]
            .iter()
            .map(|t| RemoteTag {
                name: t.to_string(),
                commit: format!("c-{t}"),
            })
            .collect();
        let names: Vec<String> = versions(Scheme::Semver, None, &tags)
            .into_iter()
            .map(|t| t.tag)
            .collect();
        assert_eq!(names, ["v1.10.0", "1.10.0-rc.1", "v1.9.1", "v1.2.0"]);
    }
}

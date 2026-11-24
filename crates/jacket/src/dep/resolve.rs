//! Choosing a commit for an entry: what was asked, and which tags can answer it.
use super::git::RemoteTag;
use super::version::{self, Range, Scheme, Tagged, Version};
use jacket_config::Dependency;

/// How an entry reads tags.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Reading {
    pub scheme: Scheme,
    pub prefix: Option<String>,
}

impl Reading {
    pub fn of(entry: &Dependency) -> Result<Self, String> {
        Ok(Self {
            scheme: entry
                .scheme
                .as_deref()
                .map(Scheme::parse)
                .transpose()?
                .unwrap_or_default(),
            prefix: entry.tag_prefix.clone(),
        })
    }

    pub fn parse(&self, tag: &str) -> Option<Version> {
        version::parse_tag(self.scheme, self.prefix.as_deref(), tag)
    }

    pub fn versions(&self, tags: &[RemoteTag]) -> Vec<Tagged> {
        version::versions(self.scheme, self.prefix.as_deref(), tags)
    }

    pub fn range(&self, text: &str) -> Result<Range, String> {
        version::parse_range(self.scheme, text)
    }
}

/// What `dep add` was told after the `@`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ask {
    /// A tag of the repository, named exactly.
    Tag(String),
    /// A range: the newest tag it admits.
    Range(String),
    /// A commit.
    Rev(String),
    /// Nothing: the newest version, else the default branch.
    Latest,
}

/// Whether `text` could be a commit hash (7 to 40 hex digits).
pub fn is_rev(text: &str) -> bool {
    (7..=40).contains(&text.len()) && text.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Reads `at` against the tags the remote has. A tag named exactly wins; a version written in
/// full (`1.2.3`) names that version's tag; anything else that parses as a range is one, and
/// hex is a commit.
pub fn classify(reading: &Reading, tags: &[RemoteTag], at: Option<&str>) -> Result<Ask, String> {
    let Some(at) = at else {
        return Ok(Ask::Latest);
    };
    if tags.iter().any(|t| t.name == at) {
        return Ok(Ask::Tag(at.to_string()));
    }
    let full = reading.parse(at).filter(|_| match reading.scheme {
        Scheme::Semver => at
            .trim_start_matches(['v', 'V'])
            .split(['-', '+'])
            .next()
            .is_some_and(|c| c.split('.').count() == 3),
        _ => true,
    });
    if let Some(wanted) = full
        && let Some(found) = reading
            .versions(tags)
            .into_iter()
            .find(|t| t.version == wanted)
    {
        return Ok(Ask::Tag(found.tag));
    }
    if is_rev(at) {
        return Ok(Ask::Rev(at.to_string()));
    }
    match reading.range(at) {
        Ok(_) => Ok(Ask::Range(at.to_string())),
        // Something written like a range that is not one says why; anything else is taken as a
        // tag name, which the remote then lacks.
        Err(error) if at.starts_with(|c: char| c.is_ascii_digit() || "^~<>=*".contains(c)) => {
            Err(error)
        }
        Err(_) => Ok(Ask::Tag(at.to_string())),
    }
}

/// The tags a range admits, newest first. Prereleases are admitted only when the range names
/// one (semver's own rule); with no range, never.
pub fn matching(reading: &Reading, tags: &[RemoteTag], range: Option<&Range>) -> Vec<Tagged> {
    reading
        .versions(tags)
        .into_iter()
        .filter(|t| match range {
            Some(range) => range.matches(&t.version),
            None => !t.version.is_prerelease(),
        })
        .collect()
}

/// The range an entry records for a freshly picked version: caret for semver, which the
/// resolver reads back; calendar and integer schemes have no caret, so the exact tag is pinned.
pub fn recorded(reading: &Reading, picked: &Tagged) -> Recorded {
    match (&picked.version, reading.scheme) {
        (Version::Semver(v), Scheme::Semver) => Recorded::Version(format!("^{v}")),
        _ => Recorded::Tag(picked.tag.clone()),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Recorded {
    Version(String),
    Tag(String),
}

/// The tag of `tags` at `commit` that names the resolved version: the highest one that reads as
/// a version, else the last by name.
pub fn resolved<'a>(reading: &Reading, at_commit: &'a [String]) -> Option<&'a String> {
    at_commit
        .iter()
        .filter_map(|t| reading.parse(t).map(|v| (v, t)))
        .max_by(|a, b| a.0.cmp(&b.0))
        .map(|(_, t)| t)
        .or_else(|| at_commit.iter().max())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tags(names: &[&str]) -> Vec<RemoteTag> {
        names
            .iter()
            .map(|n| RemoteTag {
                name: n.to_string(),
                commit: format!("c-{n}"),
            })
            .collect()
    }

    #[test]
    fn at_is_a_tag_a_full_version_a_range_or_a_commit() {
        let r = Reading::default();
        let t = tags(&["v1.2.0", "v1.3.0", "1.0.0", "docs"]);
        assert_eq!(classify(&r, &t, None), Ok(Ask::Latest));
        assert_eq!(
            classify(&r, &t, Some("1.0.0")),
            Ok(Ask::Tag("1.0.0".into()))
        );
        assert_eq!(
            classify(&r, &t, Some("1.2.0")),
            Ok(Ask::Tag("v1.2.0".into()))
        );
        assert_eq!(
            classify(&r, &t, Some("^1.2")),
            Ok(Ask::Range("^1.2".into()))
        );
        assert_eq!(classify(&r, &t, Some("1.2")), Ok(Ask::Range("1.2".into())));
        assert_eq!(
            classify(&r, &t, Some(">=1.0, <2")),
            Ok(Ask::Range(">=1.0, <2".into()))
        );
        assert_eq!(
            classify(&r, &t, Some("3f9c2a1")),
            Ok(Ask::Rev("3f9c2a1".into()))
        );
        assert_eq!(classify(&r, &t, Some("docs")), Ok(Ask::Tag("docs".into())));
        assert_eq!(
            classify(&r, &t, Some("nonsense!")),
            Ok(Ask::Tag("nonsense!".into()))
        );
        assert!(classify(&r, &t, Some("^x")).is_err());
    }

    #[test]
    fn matching_skips_prereleases_unless_the_range_names_one() {
        let r = Reading::default();
        let t = tags(&["v1.0.0", "v1.1.0-rc.1", "v1.1.0", "v2.0.0-rc.1"]);
        let names = |range: Option<&str>| -> Vec<String> {
            let range = range.map(|x| r.range(x).unwrap());
            matching(&r, &t, range.as_ref())
                .into_iter()
                .map(|t| t.tag)
                .collect()
        };
        assert_eq!(names(None), ["v1.1.0", "v1.0.0"]);
        assert_eq!(names(Some("^1")), ["v1.1.0", "v1.0.0"]);
        assert_eq!(names(Some(">=1.1.0-rc.0, <2")), ["v1.1.0", "v1.1.0-rc.1"]);
    }

    #[test]
    fn a_pick_is_recorded_as_a_caret_range_or_an_exact_tag() {
        let semver = Reading::default();
        let picked = semver.versions(&tags(&["v1.3.2"])).remove(0);
        assert_eq!(
            recorded(&semver, &picked),
            Recorded::Version("^1.3.2".into())
        );
        let integer = Reading {
            scheme: Scheme::Integer,
            prefix: None,
        };
        let picked = integer.versions(&tags(&["r42"])).remove(0);
        assert_eq!(recorded(&integer, &picked), Recorded::Tag("r42".into()));
    }

    #[test]
    fn the_resolved_tag_is_the_highest_version_at_a_commit() {
        let r = Reading::default();
        let at = vec![
            "latest".to_string(),
            "v1.0.0".to_string(),
            "v1.0.1".to_string(),
        ];
        assert_eq!(resolved(&r, &at), Some(&"v1.0.1".to_string()));
        assert_eq!(
            resolved(&r, &["zeta".to_string(), "beta".to_string()]),
            Some(&"zeta".to_string())
        );
        assert_eq!(resolved(&r, &[]), None);
    }
}

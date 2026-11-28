//! Choosing a commit for an entry: what was asked, and which tags can answer it.
use jacket_config::Dependency;

use super::git::RemoteTag;
use super::spec::At;
use super::types::{RefName, Rev};
use super::version::{self, Range, Requirement, Scheme, Tagged, Version};

/// How an entry reads tags.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Reading {
    pub scheme: Scheme,
    /// Text stripped from the front of a tag before it is read as a version.
    pub prefix: Option<String>,
}

impl Reading {
    /// How `entry` reads tags; fails when its `scheme` is not one jacket knows.
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

    /// The version `tag` names, or `None` when it does not read as one.
    pub fn parse(&self, tag: &str) -> Option<Version> {
        version::parse_tag(self.scheme, self.prefix.as_deref(), tag)
    }

    /// The tags that read as versions, with the version each names; the rest are dropped.
    pub fn versions(&self, tags: &[RemoteTag]) -> Vec<Tagged> {
        version::versions(self.scheme, self.prefix.as_deref(), tags)
    }

    /// Reads `text` as a range of versions under this scheme.
    pub fn range(&self, text: &str) -> Result<Range, String> {
        version::parse_range(self.scheme, text)
    }

    /// Reads `text` as a range under this scheme, keeping the text as written.
    pub fn requirement(&self, text: &str) -> Result<Requirement, String> {
        Requirement::parse(self.scheme, text)
    }
}

/// What `dep add` was told after the `@`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ask {
    /// A tag of the repository, named exactly.
    Tag(RefName),
    /// A range: the newest tag it admits.
    Range(Requirement),
    /// A commit.
    Rev(Rev),
    /// Nothing: the newest version, else the default branch.
    Latest,
}

/// What an entry of `jacket.toml` asks for: at most one of a version range, a tag or a commit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Asked {
    /// The newest tag the range admits.
    Range(Requirement),
    /// A tag, named exactly.
    Tag(RefName),
    /// A commit.
    Rev(Rev),
    /// Nothing: the newest version, else the default branch.
    Unpinned,
}

impl Asked {
    /// What `entry` asks for; fails when it sets more than one of `version`, `tag` and `rev`, or
    /// the one it sets is not valid.
    pub fn of(entry: &Dependency) -> Result<Self, String> {
        match (&entry.version, &entry.tag, &entry.rev) {
            (Some(range), None, None) => Ok(Self::Range(Reading::of(entry)?.requirement(range)?)),
            (None, Some(tag), None) => Ok(Self::Tag(RefName::parse(tag)?)),
            (None, None, Some(rev)) => Ok(Self::Rev(Rev::parse(rev)?)),
            (None, None, None) => Ok(Self::Unpinned),
            _ => Err("it sets more than one of `version`, `tag` and `rev`; keep one".into()),
        }
    }
}

/// Reads `at` against the tags the remote has. A tag named exactly wins; a version written in
/// full (`1.2.3`) names that version's tag; anything else that parses as a range is one, and
/// hex is a commit.
pub fn classify(reading: &Reading, tags: &[RemoteTag], at: Option<&At>) -> Result<Ask, String> {
    let Some(at) = at else {
        return Ok(Ask::Latest);
    };
    let at = at.as_str();

    if tags.iter().any(|t| t.name.as_str() == at) {
        return Ok(Ask::Tag(RefName::parse(at)?));
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
        return Ok(Ask::Tag(RefName::parse(&found.tag)?));
    }

    if let Ok(rev) = Rev::parse(at) {
        return Ok(Ask::Rev(rev));
    }

    match reading.requirement(at) {
        Ok(requirement) => Ok(Ask::Range(requirement)),
        // Something written like a range that is not one says why; anything else is taken as a
        // tag name, which the remote then lacks.
        Err(error) if at.starts_with(|c: char| c.is_ascii_digit() || "^~<>=*".contains(c)) => {
            Err(error)
        }
        Err(_) => Ok(Ask::Tag(RefName::parse(at)?)),
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

/// The setting a chosen version is written to `jacket.toml` as.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Recorded {
    /// A `version` range.
    Version(String),
    /// A `tag`.
    Tag(String),
}

/// The tag of `tags` at `commit` that names the resolved version: the highest one that reads as
/// a version, else the last by name.
pub fn resolved<'a>(reading: &Reading, at_commit: &'a [RefName]) -> Option<&'a RefName> {
    at_commit
        .iter()
        .filter_map(|t| reading.parse(t.as_str()).map(|v| (v, t)))
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
                name: crate::dep::types::RefName::parse(n).unwrap(),
                commit: crate::dep::types::CommitId::named(&format!("c-{n}")),
            })
            .collect()
    }

    fn at(text: &str) -> At {
        At::parse(text).unwrap()
    }

    fn requirement(text: &str) -> Requirement {
        Requirement::parse(Scheme::Semver, text).unwrap()
    }

    #[test]
    fn an_entry_asks_for_one_thing() {
        let entry = |version: Option<&str>, tag: Option<&str>, rev: Option<&str>| Dependency {
            git: "a/b".into(),
            version: version.map(Into::into),
            tag: tag.map(Into::into),
            rev: rev.map(Into::into),
            ..Dependency::default()
        };

        assert_eq!(
            Asked::of(&entry(Some("^1.2"), None, None)),
            Ok(Asked::Range(requirement("^1.2")))
        );
        assert_eq!(
            Asked::of(&entry(None, Some("v1"), None)),
            Ok(Asked::Tag(RefName::parse("v1").unwrap()))
        );
        assert_eq!(
            Asked::of(&entry(None, None, Some("3f9c2a1"))),
            Ok(Asked::Rev(Rev::parse("3f9c2a1").unwrap()))
        );
        assert_eq!(Asked::of(&entry(None, None, None)), Ok(Asked::Unpinned));
        assert!(Asked::of(&entry(Some("^1"), Some("v1"), None)).is_err());
        assert!(Asked::of(&entry(None, None, Some("zz"))).is_err());
        assert!(Asked::of(&entry(Some("not a range"), None, None)).is_err());
    }

    #[test]
    fn at_is_a_tag_a_full_version_a_range_or_a_commit() {
        let r = Reading::default();
        let t = tags(&["v1.2.0", "v1.3.0", "1.0.0", "docs"]);

        assert_eq!(classify(&r, &t, None), Ok(Ask::Latest));
        assert_eq!(
            classify(&r, &t, Some(&at("1.0.0"))),
            Ok(Ask::Tag(RefName::parse("1.0.0").unwrap()))
        );
        assert_eq!(
            classify(&r, &t, Some(&at("1.2.0"))),
            Ok(Ask::Tag(RefName::parse("v1.2.0").unwrap()))
        );
        assert_eq!(
            classify(&r, &t, Some(&at("^1.2"))),
            Ok(Ask::Range(requirement("^1.2")))
        );
        assert_eq!(
            classify(&r, &t, Some(&at("1.2"))),
            Ok(Ask::Range(requirement("1.2")))
        );
        assert_eq!(
            classify(&r, &t, Some(&at(">=1.0, <2"))),
            Ok(Ask::Range(requirement(">=1.0, <2")))
        );
        assert_eq!(
            classify(&r, &t, Some(&at("3f9c2a1"))),
            Ok(Ask::Rev(Rev::parse("3f9c2a1").unwrap()))
        );
        assert_eq!(
            classify(&r, &t, Some(&at("docs"))),
            Ok(Ask::Tag(RefName::parse("docs").unwrap()))
        );
        assert_eq!(
            classify(&r, &t, Some(&at("nonsense!"))),
            Ok(Ask::Tag(RefName::parse("nonsense!").unwrap()))
        );
        assert!(classify(&r, &t, Some(&at("^x"))).is_err());
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
        let name = |t: &str| RefName::parse(t).unwrap();
        let at = vec![name("latest"), name("v1.0.0"), name("v1.0.1")];

        assert_eq!(resolved(&r, &at), Some(&name("v1.0.1")));
        assert_eq!(
            resolved(&r, &[name("zeta"), name("beta")]),
            Some(&name("zeta"))
        );
        assert_eq!(resolved(&r, &[]), None);
    }
}

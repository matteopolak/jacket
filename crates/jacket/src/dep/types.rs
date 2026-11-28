//! Values with a meaning of their own, parsed once where they enter `jacket dep`. Functions take
//! these types instead of `String`, so none of them checks its input again.
use std::fmt;

/// A full commit id: 40 (SHA-1) or 64 (SHA-256) lowercase hex digits.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommitId(String);

impl CommitId {
    /// Reads a full id, in either case.
    pub fn parse(text: &str) -> Result<Self, String> {
        let text = text.trim();
        if matches!(text.len(), 40 | 64) && text.bytes().all(|b| b.is_ascii_hexdigit()) {
            Ok(Self(text.to_ascii_lowercase()))
        } else {
            Err(format!(
                "`{text}` is not a full commit id (40 or 64 hex digits)"
            ))
        }
    }

    /// A distinct valid id for tests, from a number.
    #[cfg(test)]
    pub fn numbered(n: u64) -> Self {
        Self(format!("{n:040x}"))
    }

    /// A distinct valid id for tests, from a label.
    #[cfg(test)]
    pub fn named(label: &str) -> Self {
        let hex: String = label.bytes().map(|b| format!("{b:02x}")).collect();
        Self(format!("{hex:0>40}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The first seven digits, for messages.
    pub fn short(&self) -> &str {
        &self.0[..7]
    }

    /// Whether `rev`, a full or abbreviated id, names this commit.
    pub fn starts_with(&self, rev: &Rev) -> bool {
        self.0.starts_with(rev.as_str())
    }

    /// Whether `rev` and this commit agree on every digit they both have.
    pub fn agrees_with(&self, rev: &Rev) -> bool {
        self.starts_with(rev) || rev.as_str().starts_with(&self.0)
    }
}

impl AsRef<str> for CommitId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CommitId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A commit as a user writes it in `rev = "..."`: 7 to 64 hex digits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rev(String);

impl Rev {
    pub fn parse(text: &str) -> Result<Self, String> {
        let text = text.trim();
        if (7..=64).contains(&text.len()) && text.bytes().all(|b| b.is_ascii_hexdigit()) {
            Ok(Self(text.to_ascii_lowercase()))
        } else {
            Err(format!("`{text}` is not a commit (7 to 64 hex digits)"))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Rev {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&CommitId> for Rev {
    fn from(id: &CommitId) -> Self {
        Self(id.0.clone())
    }
}

/// The commit a pin checks out: a full id, or the abbreviated one a user wrote in `rev`, which
/// git resolves when it checks it out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Commit(CommitId),
    Rev(Rev),
}

impl Target {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Commit(id) => id.as_str(),
            Self::Rev(rev) => rev.as_str(),
        }
    }
}

impl AsRef<str> for Target {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<CommitId> for Target {
    fn from(id: CommitId) -> Self {
        Self::Commit(id)
    }
}

/// A tag or branch name, valid as a git ref component (see `git check-ref-format`).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RefName(String);

impl RefName {
    pub fn parse(text: &str) -> Result<Self, String> {
        let bad = |why: &str| Err(format!("`{text}` is not a tag or branch name: {why}"));
        if text.is_empty() {
            return bad("it is empty");
        }
        if text.starts_with(['-', '/', '.'])
            || text.ends_with(['/', '.'])
            || text.ends_with(".lock")
        {
            return bad("it starts or ends with a character git refuses there");
        }
        if text.contains("..") || text.contains("//") || text.contains("@{") || text == "@" {
            return bad("it has a sequence git refuses");
        }
        if text
            .chars()
            .any(|c| c.is_control() || c.is_whitespace() || "~^:?*[\\".contains(c))
        {
            return bad("it has a character git refuses");
        }
        Ok(Self(text.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for RefName {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RefName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A folder inside the project, written with `/`: relative, with no `.` or `..` component, so it
/// can never name anything outside the project.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubPath(String);

impl SubPath {
    pub fn parse(text: &str) -> Result<Self, String> {
        let bad = |why: &str| {
            Err(format!(
                "`{text}` is not a folder inside the project: {why}"
            ))
        };
        if text.is_empty() {
            return bad("it is empty");
        }
        if text.starts_with('/') || text.contains('\\') || text.as_bytes().get(1) == Some(&b':') {
            return bad("it is absolute or uses `\\`");
        }
        if text.chars().any(char::is_control) {
            return bad("it has a control character");
        }
        if text.split('/').any(|part| matches!(part, "" | "." | "..")) {
            return bad("it has an empty, `.` or `..` part");
        }
        Ok(Self(text.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// This folder under `base`.
    pub fn under(&self, base: &std::path::Path) -> std::path::PathBuf {
        base.join(&self.0)
    }

    /// `inner`, a folder of the repository checked out here, as a folder of the project.
    #[must_use]
    pub fn join(&self, inner: &SubPath) -> SubPath {
        Self(format!("{}/{}", self.0, inner.0))
    }

    /// The folder holding this one, if it is not at the top.
    pub fn parent(&self) -> Option<&str> {
        self.0.rsplit_once('/').map(|(parent, _)| parent)
    }

    /// The last part.
    pub fn name(&self) -> &str {
        self.0.rsplit('/').next().unwrap_or(&self.0)
    }
}

impl std::ops::Deref for SubPath {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl AsRef<std::path::Path> for SubPath {
    fn as_ref(&self) -> &std::path::Path {
        std::path::Path::new(&self.0)
    }
}

impl fmt::Display for SubPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Something git can resolve to a commit inside a clone: a full id, a user's abbreviation or a
/// tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Revision {
    Commit(CommitId),
    Rev(Rev),
    Tag(RefName),
}

impl Revision {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Commit(id) => id.as_str(),
            Self::Rev(rev) => rev.as_str(),
            Self::Tag(tag) => tag.as_str(),
        }
    }
}

impl From<&Target> for Revision {
    fn from(target: &Target) -> Self {
        match target {
            Target::Commit(id) => Self::Commit(id.clone()),
            Target::Rev(rev) => Self::Rev(rev.clone()),
        }
    }
}

impl From<&CommitId> for Revision {
    fn from(id: &CommitId) -> Self {
        Self::Commit(id.clone())
    }
}

impl From<&RefName> for Revision {
    fn from(tag: &RefName) -> Self {
        Self::Tag(tag.clone())
    }
}

/// A release-age floor in whole seconds: `3d`, `12h`, `90m`, `30s`, `2w` or `0`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Age(i64);

impl Age {
    pub const ZERO: Self = Self(0);

    pub fn parse(text: &str) -> Result<Self, String> {
        let text = text.trim();
        let bad = || {
            format!(
                "`{text}` is not a duration: write a number and a unit (`3d`, `12h`, `90m`, `2w`), or `0`"
            )
        };
        if text == "0" {
            return Ok(Self::ZERO);
        }
        let scale: i64 = match text.chars().last().ok_or_else(bad)? {
            's' => 1,
            'm' => 60,
            'h' => 3600,
            'd' => 86_400,
            'w' => 604_800,
            _ => return Err(bad()),
        };
        let n: i64 = text[..text.len() - 1].trim().parse().map_err(|_| bad())?;
        if n < 0 {
            return Err(bad());
        }
        n.checked_mul(scale).map(Self).ok_or_else(bad)
    }

    pub fn seconds(self) -> i64 {
        self.0
    }
}

/// In the largest unit that divides it exactly: `3d`, `36h`, `2w`, `0`.
impl fmt::Display for Age {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (scale, unit) in [(604_800, 'w'), (86_400, 'd'), (3600, 'h'), (60, 'm')] {
            if self.0 != 0 && self.0 % scale == 0 {
                return write!(f, "{}{unit}", self.0 / scale);
            }
        }
        write!(
            f,
            "{}",
            if self.0 == 0 {
                "0".into()
            } else {
                format!("{}s", self.0)
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_ids_are_full_and_lowercase() {
        let id = CommitId::parse(&"AB".repeat(20)).unwrap();
        assert_eq!(id.as_str(), "ab".repeat(20));
        assert_eq!(id.short(), "abababa");
        assert!(CommitId::parse("3f9c2a1").is_err());
        assert!(CommitId::parse(&"g".repeat(40)).is_err());
        assert!(CommitId::parse(&"a".repeat(64)).is_ok());
    }

    #[test]
    fn revs_may_be_abbreviated() {
        assert!(Rev::parse("3f9c2a1").is_ok());
        assert!(Rev::parse("3f9c2a").is_err());
        let id = CommitId::parse(&"3f9c2a1".repeat(6)[..40]).unwrap();
        assert!(id.starts_with(&Rev::parse("3f9c2a1").unwrap()));
    }

    #[test]
    fn ref_names_follow_git() {
        for ok in ["v1.2.0", "release/1.0", "alpha-3", "feature_x"] {
            assert!(RefName::parse(ok).is_ok(), "{ok}");
        }
        for bad in [
            "", "-x", "a..b", "a b", "a:b", "x.lock", "a//b", "/a", "a/", "a^", "@",
        ] {
            assert!(RefName::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn sub_paths_stay_inside_the_project() {
        for ok in ["modules/a", "vendor/x/y", "a"] {
            assert!(SubPath::parse(ok).is_ok(), "{ok}");
        }
        for bad in [
            "", "/etc", "../x", "a/../b", "a/./b", "a//b", "a/", "C:/x", "a\\b", "a\nb",
        ] {
            assert!(SubPath::parse(bad).is_err(), "{bad:?}");
        }
        let path = SubPath::parse("modules/a").unwrap();
        assert_eq!(path.parent(), Some("modules"));
        assert_eq!(path.name(), "a");
        assert_eq!(
            path.join(&SubPath::parse("lib/b").unwrap()).as_str(),
            "modules/a/lib/b"
        );
    }

    #[test]
    fn ages_read_units() {
        assert_eq!(Age::parse("3d").unwrap().seconds(), 259_200);
        assert_eq!(Age::parse("0").unwrap(), Age::ZERO);
        assert_eq!(Age::parse("2w").unwrap().seconds(), 1_209_600);
        assert_eq!(Age::parse("12h").unwrap().seconds(), 43_200);
        assert_eq!(Age::parse("36h").unwrap().to_string(), "36h");
        assert_eq!(Age::parse("3d").unwrap().to_string(), "3d");
        assert_eq!(Age::parse("90s").unwrap().to_string(), "90s");
        assert_eq!(Age::ZERO.to_string(), "0");
        for bad in ["", "3", "d", "-1d", "3x", "1.5d", "3 days"] {
            assert!(Age::parse(bad).is_err(), "{bad}");
        }
    }
}

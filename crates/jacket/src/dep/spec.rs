//! What `jacket dep add` is given: a repository and what to pin.

use std::fmt;

/// A repository as `jacket.toml` or `dep add` spells it: `user/repo`, a host path, a URL, a
/// `git@host:path` address or a local path.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Source(String);

/// The address git fetches for a [`Source`].
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RepoUrl(String);

impl Source {
    /// Reads a repository address. Fails on empty text, text that starts with `-` (git would
    /// read it as an option), and text with whitespace or control characters; nothing else is
    /// checked.
    pub fn parse(text: &str) -> Result<Self, String> {
        let text = text.trim();

        if text.is_empty() {
            return Err("a repository is empty".into());
        }

        if text.starts_with('-') || text.chars().any(|c| c.is_control() || c.is_whitespace()) {
            return Err(format!("`{text}` is not a repository address"));
        }

        Ok(Self(text.to_string()))
    }

    /// The URL git fetches: `user/repo` is GitHub, `host/path` is HTTPS, anything else is used
    /// as written.
    pub fn url(&self) -> RepoUrl {
        let git = self.0.as_str();

        if git.contains("://") || git.starts_with("git@") || RepoUrl(git.to_string()).is_local() {
            return RepoUrl(git.to_string());
        }

        let first = git.split('/').next().unwrap_or("");

        if first.contains('.') && git.contains('/') {
            return RepoUrl(format!("https://{git}"));
        }

        RepoUrl(format!("https://github.com/{git}"))
    }

    /// With a relative local path made absolute from `base`, so every command reaches the same
    /// repository from any folder (git would otherwise resolve `./x` against a remote for a
    /// submodule and against the current folder for everything else).
    #[must_use]
    pub fn resolved(&self, base: &std::path::Path) -> Self {
        if self.0.starts_with("./") || self.0.starts_with("../") {
            let mut path = base.to_path_buf();

            for part in std::path::Path::new(&self.0).components() {
                match part {
                    std::path::Component::ParentDir => {
                        path.pop();
                    }
                    std::path::Component::CurDir => {}
                    other => path.push(other),
                }
            }

            Self(path.display().to_string())
        } else {
            self.clone()
        }
    }

    /// The folder-safe name a repository suggests: its last path segment without `.git`.
    pub fn name(&self) -> String {
        let trimmed = self.0.trim_end_matches('/');
        let last = trimmed.rsplit(['/', ':']).next().unwrap_or(trimmed);

        last.strip_suffix(".git").unwrap_or(last).to_string()
    }

    /// Whether two spellings name one repository.
    pub fn same_repo(&self, other: &Self) -> bool {
        let (a, b) = (self.url(), other.url());

        match (a.host_and_repo(), b.host_and_repo()) {
            (Some(x), Some(y)) => x == y,
            _ => a.0.trim_end_matches('/') == b.0.trim_end_matches('/'),
        }
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl RepoUrl {
    /// The address as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether the URL names a folder on this machine.
    pub fn is_local(&self) -> bool {
        let url = self.0.as_str();

        url.starts_with('/')
            || url.starts_with("./")
            || url.starts_with("../")
            || url.starts_with("file://")
            || (url.len() > 2
                && url.as_bytes()[1] == b':'
                && url.as_bytes()[0].is_ascii_alphabetic())
    }

    /// The forge host and `owner/name` path, when it is an HTTPS or SSH address.
    pub fn host_and_repo(&self) -> Option<(String, String)> {
        let url = self.0.as_str();
        let rest = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .or_else(|| url.strip_prefix("ssh://git@"))
            .or_else(|| url.strip_prefix("git@"))?;
        let rest = rest.split_once('@').map_or(rest, |(_, r)| r);
        let (host, path) = rest.split_once(['/', ':'])?;
        let path = path.trim_matches('/');
        let path = path.strip_suffix(".git").unwrap_or(path);

        (!path.is_empty()).then(|| (host.to_ascii_lowercase(), path.to_string()))
    }
}

impl fmt::Display for RepoUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// What follows the `@` of a spec, or `--to`: a tag, a version, a range or a commit, as the user
/// wrote it. `resolve::classify` reads it against the remote's tags.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct At(String);

impl At {
    /// Reads what follows an `@`. Fails on empty text, text that starts with `-`, and text with
    /// control characters.
    pub fn parse(text: &str) -> Result<Self, String> {
        let text = text.trim();

        if text.is_empty() || text.starts_with('-') || text.chars().any(char::is_control) {
            return Err(format!("`{text}` is not a tag, version, range or commit"));
        }

        Ok(Self(text.to_string()))
    }

    /// The text as written, trimmed.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for At {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A parsed `<spec>`: `user/repo`, a host path or a URL, with an optional `@<what>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spec {
    /// The repository.
    pub git: Source,
    /// What follows the `@`.
    pub at: Option<At>,
}

/// Splits `<spec>` at its `@`. `git@host:path` addresses keep theirs.
pub fn parse(spec: &str) -> Result<Spec, String> {
    let from = if spec.starts_with("git@") || spec.contains("://") && spec.contains('@') {
        spec.find(['/', ':']).map_or(0, |i| i + 1)
    } else {
        0
    };
    let (git, at) = match spec[from..].rfind('@') {
        Some(at) => (
            &spec[..from + at],
            Some(&spec[from + at + 1..]).filter(|s| !s.is_empty()),
        ),
        None => (spec, None),
    };

    Ok(Spec {
        git: Source::parse(git)?,
        at: at.map(At::parse).transpose()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specs_split_at_the_version_not_at_the_ssh_user() {
        let table = [
            ("a/b", "a/b", None),
            ("a/b@v1.2.0", "a/b", Some("v1.2.0")),
            ("a/b@^1.2", "a/b", Some("^1.2")),
            ("git@github.com:a/b.git", "git@github.com:a/b.git", None),
            (
                "git@github.com:a/b.git@v2",
                "git@github.com:a/b.git",
                Some("v2"),
            ),
            (
                "https://u@host.io/a/b@3f9c2a1",
                "https://u@host.io/a/b",
                Some("3f9c2a1"),
            ),
            (
                "codeberg.org/x/y@>=1, <2",
                "codeberg.org/x/y",
                Some(">=1, <2"),
            ),
        ];

        for (spec, git, at) in table {
            assert_eq!(
                parse(spec).unwrap(),
                Spec {
                    git: Source::parse(git).unwrap(),
                    at: at.map(|a| At::parse(a).unwrap())
                },
                "{spec}"
            );
        }
    }

    fn source(text: &str) -> Source {
        Source::parse(text).unwrap()
    }

    #[test]
    fn relative_local_paths_are_made_absolute_once() {
        let base = std::path::Path::new("/work/project");
        let own = |text: &str| Source::parse(text).unwrap().resolved(base);

        assert_eq!(own("../lib").to_string(), "/work/lib");
        assert_eq!(own("./lib").to_string(), "/work/project/lib");

        for same in ["/abs/lib", "a/b", "https://h/a/b", "git@h:a/b.git"] {
            assert_eq!(own(same).to_string(), same);
        }
    }

    #[test]
    fn repositories_get_urls_names_and_hosts() {
        assert_eq!(source("a/b").url().as_str(), "https://github.com/a/b");
        assert_eq!(
            source("codeberg.org/x/y").url().as_str(),
            "https://codeberg.org/x/y"
        );
        assert_eq!(source("git@h:a/b.git").url().as_str(), "git@h:a/b.git");
        assert_eq!(source("/tmp/r.git").url().as_str(), "/tmp/r.git");
        assert_eq!(source("git@h:a/b.git").name(), "b");
        assert_eq!(source("github.com/a/jaison/").name(), "jaison");
        assert_eq!(
            source("git@github.com:a/b.git").url().host_and_repo(),
            Some(("github.com".into(), "a/b".into()))
        );
        assert_eq!(
            source("https://gitlab.com/g/s/p").url().host_and_repo(),
            Some(("gitlab.com".into(), "g/s/p".into()))
        );
        assert!(source("a/b").same_repo(&source("https://github.com/a/b.git")));
        assert!(!source("a/b").same_repo(&source("a/c")));
        assert!(source("/tmp/r").url().is_local());
        assert!(!source("a/b").url().is_local());
    }

    #[test]
    fn sources_refuse_what_cannot_be_an_address() {
        for bad in ["", "  ", "--upload-pack=x", "a b", "a\nb"] {
            assert!(Source::parse(bad).is_err(), "{bad:?}");
        }

        assert!(parse("--x@v1").is_err());
    }
}

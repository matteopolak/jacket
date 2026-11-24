//! What `jacket dep add` is given: a repository and what to pin.

/// A parsed `<spec>`: `user/repo`, a host path or a URL, with an optional `@<what>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spec {
    /// As the user wrote it (and as `jacket.toml` records it).
    pub git: String,
    /// What follows the `@`.
    pub at: Option<String>,
}

/// Splits `<spec>` at its `@`. `git@host:path` addresses keep theirs.
pub fn parse(spec: &str) -> Spec {
    let from = if spec.starts_with("git@") || spec.contains("://") && spec.contains('@') {
        spec.find(['/', ':']).map_or(0, |i| i + 1)
    } else {
        0
    };
    match spec[from..].rfind('@') {
        Some(at) => Spec {
            git: spec[..from + at].to_string(),
            at: Some(spec[from + at + 1..].to_string()).filter(|s| !s.is_empty()),
        },
        None => Spec {
            git: spec.to_string(),
            at: None,
        },
    }
}

/// The URL git fetches for `git` as `jacket.toml` spells it: `user/repo` is GitHub, `host/path`
/// is HTTPS, anything else is used as written.
pub fn url(git: &str) -> String {
    if git.contains("://") || git.starts_with("git@") || super::git::is_local(git) {
        return git.to_string();
    }
    let first = git.split('/').next().unwrap_or("");
    if first.contains('.') && git.contains('/') {
        return format!("https://{git}");
    }
    format!("https://github.com/{git}")
}

/// The folder-safe name a repository suggests: its last path segment without `.git`.
pub fn repo_name(git: &str) -> String {
    let trimmed = git.trim_end_matches('/');
    let last = trimmed.rsplit(['/', ':']).next().unwrap_or(trimmed);
    last.strip_suffix(".git").unwrap_or(last).to_string()
}

/// The forge host and `owner/name` path of a repository URL, when it is an HTTPS or SSH address.
pub fn host_and_repo(url: &str) -> Option<(String, String)> {
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

/// Whether two spellings name one repository.
pub fn same_repo(a: &str, b: &str) -> bool {
    let (a, b) = (url(a), url(b));
    match (host_and_repo(&a), host_and_repo(&b)) {
        (Some(x), Some(y)) => x == y,
        _ => a.trim_end_matches('/') == b.trim_end_matches('/'),
    }
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
                parse(spec),
                Spec {
                    git: git.into(),
                    at: at.map(Into::into)
                },
                "{spec}"
            );
        }
    }

    #[test]
    fn repositories_get_urls_names_and_hosts() {
        assert_eq!(url("a/b"), "https://github.com/a/b");
        assert_eq!(url("codeberg.org/x/y"), "https://codeberg.org/x/y");
        assert_eq!(url("git@h:a/b.git"), "git@h:a/b.git");
        assert_eq!(url("/tmp/r.git"), "/tmp/r.git");
        assert_eq!(repo_name("git@h:a/b.git"), "b");
        assert_eq!(repo_name("github.com/a/jaison/"), "jaison");
        assert_eq!(
            host_and_repo("git@github.com:a/b.git"),
            Some(("github.com".into(), "a/b".into()))
        );
        assert_eq!(
            host_and_repo("https://gitlab.com/g/s/p"),
            Some(("gitlab.com".into(), "g/s/p".into()))
        );
        assert!(same_repo("a/b", "https://github.com/a/b.git"));
        assert!(!same_repo("a/b", "a/c"));
    }
}

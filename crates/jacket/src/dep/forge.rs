//! Publish times from a forge's own records, behind a trait so unit tests never touch the
//! network.
//!
//! Only release records count: GitHub and GitLab releases, and Gitea, Forgejo and Codeberg
//! releases. Commit and tagger dates never do, because whoever made the commit wrote them. A
//! tag without a release has no checkable age.
//!
//! Lookups are cached for good, keyed by host, repository, tag and commit, so a tag pushed again
//! to another commit misses the cache. Release lists are fetched with `If-None-Match`, which
//! GitHub does not count against its rate limit when nothing changed.
use super::policy::parse_time;
use serde_json::Value;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Status, lower-cased headers and body of an HTTP answer.
type Response = (u16, Vec<(String, String)>, String);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    GitHub,
    GitLab,
    /// Gitea, Forgejo and Codeberg.
    Gitea,
}

/// The forge a host is, when it keeps release times.
pub fn kind_of(host: &str) -> Option<Kind> {
    match host {
        "github.com" => Some(Kind::GitHub),
        "gitlab.com" => Some(Kind::GitLab),
        "codeberg.org" => Some(Kind::Gitea),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    pub tag: String,
    /// Unix seconds.
    pub published: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fetched {
    Fresh {
        etag: Option<String>,
        releases: Vec<Release>,
    },
    NotModified,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ForgeError {
    /// The forge refused for now: the text says until when, or how to lift the limit.
    RateLimited(String),
    Failed(String),
}

impl std::fmt::Display for ForgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RateLimited(text) | Self::Failed(text) => f.write_str(text),
        }
    }
}

/// A repository on a forge.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Repo {
    pub host: String,
    pub path: String,
}

impl Repo {
    pub fn key(&self) -> String {
        format!("{}/{}", self.host, self.path)
    }
}

pub trait Forge {
    /// The repository's releases, or `NotModified` when `etag` still matches.
    fn releases(&self, kind: Kind, repo: &Repo, etag: Option<&str>) -> Result<Fetched, ForgeError>;

    /// The releases of several repositories in as few requests as the forge allows. The default
    /// asks one by one; GitHub overrides it with a single GraphQL query.
    fn releases_many(&self, repos: &[(Kind, Repo)]) -> Vec<(Repo, Result<Fetched, ForgeError>)> {
        repos
            .iter()
            .map(|(kind, repo)| (repo.clone(), self.releases(*kind, repo, None)))
            .collect()
    }
}

/// What is remembered between runs.
#[derive(Default, Debug)]
pub struct Cache {
    file: Option<PathBuf>,
    /// `host/path#tag@commit` to Unix seconds.
    times: BTreeMap<String, i64>,
    /// `host/path` to the etag of the list and the list.
    lists: BTreeMap<String, (Option<String>, Vec<Release>)>,
    /// The lists fetched or revalidated during this run, which need no second request.
    fresh: std::collections::BTreeSet<String>,
}

impl Cache {
    #[cfg(test)]
    pub fn in_memory() -> Self {
        Self::default()
    }

    /// The cache file under the user cache folder.
    pub fn open() -> Self {
        let file = jacket_config::user_cache_dir()
            .ok()
            .map(|dir| dir.join("jacket").join("dep-release-times.json"));
        let mut cache = Self {
            file: file.clone(),
            ..Self::default()
        };
        let parsed: Option<Value> = file
            .and_then(|f| std::fs::read_to_string(f).ok())
            .and_then(|t| serde_json::from_str(&t).ok());
        if let Some(Value::Object(root)) = parsed {
            if let Some(Value::Object(times)) = root.get("times") {
                for (k, v) in times {
                    if let Some(t) = v.as_i64() {
                        cache.times.insert(k.clone(), t);
                    }
                }
            }
            if let Some(Value::Object(lists)) = root.get("lists") {
                for (k, v) in lists {
                    let etag = v.get("etag").and_then(Value::as_str).map(str::to_string);
                    let releases = v
                        .get("releases")
                        .and_then(Value::as_array)
                        .map(|a| {
                            a.iter()
                                .filter_map(|r| {
                                    Some(Release {
                                        tag: r.get(0)?.as_str()?.to_string(),
                                        published: r.get(1)?.as_i64()?,
                                    })
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    cache.lists.insert(k.clone(), (etag, releases));
                }
            }
        }
        cache
    }

    pub fn save(&self) {
        let Some(file) = &self.file else {
            return;
        };
        let lists: serde_json::Map<String, Value> = self
            .lists
            .iter()
            .map(|(k, (etag, releases))| {
                let rows: Vec<Value> = releases
                    .iter()
                    .map(|r| serde_json::json!([r.tag, r.published]))
                    .collect();
                (
                    k.clone(),
                    serde_json::json!({ "etag": etag, "releases": rows }),
                )
            })
            .collect();
        let body = serde_json::json!({ "times": self.times, "lists": lists });
        if let Some(parent) = file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(file, body.to_string());
    }

    fn time_key(repo: &Repo, tag: &str, commit: &str) -> String {
        format!("{}#{tag}@{commit}", repo.key())
    }

    /// A time already known for `tag` at `commit`, without asking anyone.
    pub fn known(&self, repo: &Repo, tag: &str, commit: &str) -> Option<i64> {
        self.times.get(&Self::time_key(repo, tag, commit)).copied()
    }

    pub fn remember_list(&mut self, repo: &Repo, etag: Option<String>, releases: Vec<Release>) {
        self.fresh.insert(repo.key());
        self.lists.insert(repo.key(), (etag, releases));
    }

    fn list(&self, repo: &Repo) -> Option<&(Option<String>, Vec<Release>)> {
        self.lists.get(&repo.key())
    }
}

/// Why a version's age is unknown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unknown {
    /// The host keeps no release times, or the tag has no release.
    NoRecord(String),
    /// The forge could not be asked.
    Failed(String),
}

impl std::fmt::Display for Unknown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoRecord(text) | Self::Failed(text) => f.write_str(text),
        }
    }
}

/// When `tag` (at `commit`) was published, from the cache or the forge.
pub fn published(
    forge: &dyn Forge,
    cache: &mut Cache,
    repo: &Repo,
    tag: &str,
    commit: &str,
) -> Result<i64, Unknown> {
    if let Some(time) = cache.known(repo, tag, commit) {
        return Ok(time);
    }
    let Some(kind) = kind_of(&repo.host) else {
        return Err(Unknown::NoRecord(format!(
            "{} keeps no release times jacket can read",
            repo.host
        )));
    };
    let held = cache.list(repo).cloned();
    let releases = match held {
        Some((_, list)) if cache.fresh.contains(&repo.key()) => list,
        held => {
            let etag = held.as_ref().and_then(|(etag, _)| etag.clone());
            match forge.releases(kind, repo, etag.as_deref()) {
                Ok(Fetched::Fresh {
                    etag,
                    releases,
                }) => {
                    cache.remember_list(repo, etag, releases.clone());
                    releases
                }
                Ok(Fetched::NotModified) => {
                    cache.fresh.insert(repo.key());
                    held.map(|(_, r)| r).unwrap_or_default()
                }
                Err(error) => return Err(Unknown::Failed(error.to_string())),
            }
        }
    };
    match releases.iter().find(|r| r.tag == tag) {
        Some(release) => {
            cache
                .times
                .insert(Cache::time_key(repo, tag, commit), release.published);
            Ok(release.published)
        }
        None => Err(Unknown::NoRecord(format!(
            "{} has no release for {tag}, so there is no recorded publish time",
            repo.key()
        ))),
    }
}

/// Fills the cache's lists for `repos` in as few requests as possible.
pub fn prefetch(forge: &dyn Forge, cache: &mut Cache, repos: &[Repo]) {
    let asked: Vec<(Kind, Repo)> = repos
        .iter()
        .filter_map(|r| kind_of(&r.host).map(|k| (k, r.clone())))
        .filter(|(_, r)| cache.list(r).is_none())
        .collect();
    if asked.len() < 2 {
        return;
    }
    for (repo, result) in forge.releases_many(&asked) {
        if let Ok(Fetched::Fresh {
            etag,
            releases,
        }) = result
        {
            cache.remember_list(&repo, etag, releases);
        }
    }
}

// ---- the real forges --------------------------------------------------------------------------

/// The forges over `curl`, as `jacket upgrade` does, so no HTTP stack is linked in.
pub struct Http {
    base: Option<String>,
    github_token: Option<String>,
    gitlab_token: Option<String>,
}

impl Http {
    pub fn from_env() -> Self {
        let token = |var: &jacket_config::env::Var| {
            var.get()
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
        };
        Self {
            base: jacket_config::env::FORGE_API
                .get()
                .filter(|b| !b.is_empty()),
            github_token: token(&jacket_config::env::GITHUB_TOKEN),
            gitlab_token: token(&jacket_config::env::GITLAB_TOKEN),
        }
    }

    fn api(&self, default: &str) -> String {
        self.base.as_deref().map_or_else(
            || default.to_string(),
            |b| b.trim_end_matches('/').to_string(),
        )
    }

    fn request(
        &self,
        url: &str,
        etag: Option<&str>,
        token: Option<(&str, &str)>,
        post: Option<&str>,
    ) -> Result<Response, ForgeError> {
        let mut command = Command::new("curl");
        command
            .args(["--silent", "--show-error", "--include", "--max-time", "30"])
            .args([
                "--user-agent",
                concat!("jacket-dep/", env!("CARGO_PKG_VERSION")),
            ])
            .args(["--header", "Accept: application/json"]);
        if self.base.is_none() {
            command.args(["--proto", "=https", "--tlsv1.2"]);
        }
        if let Some(etag) = etag {
            command.args(["--header", &format!("If-None-Match: {etag}")]);
        }
        let mut config = String::new();
        if let Some((header, value)) = token {
            if value.contains(['"', '\\', '\n', '\r']) {
                return Err(ForgeError::Failed(
                    "a token has characters a token never has".into(),
                ));
            }
            // Through stdin, so the token is never on a command line others can list.
            config = format!("header = \"{header}: {value}\"\n");
            command.args(["--config", "-"]);
        }
        if let Some(body) = post {
            command.args([
                "--header",
                "Content-Type: application/json",
                "--data-binary",
                body,
            ]);
        }
        let mut child = command
            .arg(url)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ForgeError::Failed(format!("could not run `curl`: {e}")))?;
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(config.as_bytes());
        }
        let output = child
            .wait_with_output()
            .map_err(|e| ForgeError::Failed(format!("curl failed: {e}")))?;
        if !output.status.success() {
            return Err(ForgeError::Failed(format!(
                "could not reach {url}: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }
        parse_response(&String::from_utf8_lossy(&output.stdout)).ok_or_else(|| {
            ForgeError::Failed(format!("{url} answered with something that is not HTTP"))
        })
    }
}

/// Status, lower-cased headers and body of a `curl --include` answer.
fn parse_response(text: &str) -> Option<Response> {
    let mut rest = text;
    loop {
        let (head, body) = rest
            .split_once("\r\n\r\n")
            .or_else(|| rest.split_once("\n\n"))?;
        let mut lines = head.lines();
        let status: u16 = lines.next()?.split_whitespace().nth(1)?.parse().ok()?;
        if (100..200).contains(&status) {
            rest = body;
            continue;
        }
        let headers = lines
            .filter_map(|l| l.split_once(':'))
            .map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_string()))
            .collect();
        return Some((status, headers, body.to_string()));
    }
}

fn header<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.as_str())
}

impl Http {
    fn finish(
        &self,
        kind: Kind,
        status: u16,
        headers: &[(String, String)],
        body: &str,
    ) -> Result<Fetched, ForgeError> {
        match status {
            304 => Ok(Fetched::NotModified),
            200 => {
                let value: Value = serde_json::from_str(body).map_err(|e| {
                    ForgeError::Failed(format!("the release list is not JSON: {e}"))
                })?;
                Ok(Fetched::Fresh {
                    etag: header(headers, "etag").map(str::to_string),
                    releases: releases_from(kind, &value),
                })
            }
            403 | 429 => {
                let limited = header(headers, "x-ratelimit-remaining") == Some("0")
                    || header(headers, "retry-after").is_some()
                    || status == 429;
                let token = match kind {
                    Kind::GitHub => "GITHUB_TOKEN",
                    Kind::GitLab => "GITLAB_TOKEN",
                    Kind::Gitea => "",
                };
                if limited {
                    let hint = if token.is_empty() {
                        String::new()
                    } else {
                        format!("; set {token} to lift the limit")
                    };
                    Err(ForgeError::RateLimited(format!(
                        "the forge's rate limit is reached{hint}"
                    )))
                } else {
                    Err(ForgeError::Failed(format!(
                        "the forge refused the request (HTTP {status})"
                    )))
                }
            }
            404 => Err(ForgeError::Failed(
                "the forge has no such repository, or it is private".into(),
            )),
            other => Err(ForgeError::Failed(format!(
                "the forge answered HTTP {other}"
            ))),
        }
    }
}

/// The releases in a forge's JSON list; drafts and unpublished ones are left out.
pub fn releases_from(kind: Kind, value: &Value) -> Vec<Release> {
    let (tag_key, time_key) = match kind {
        Kind::GitHub | Kind::Gitea => ("tag_name", "published_at"),
        Kind::GitLab => ("tag_name", "released_at"),
    };
    value
        .as_array()
        .map(|list| {
            list.iter()
                .filter(|r| r.get("draft").and_then(Value::as_bool) != Some(true))
                .filter_map(|r| {
                    Some(Release {
                        tag: r.get(tag_key)?.as_str()?.to_string(),
                        published: parse_time(r.get(time_key)?.as_str()?)?,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

impl Forge for Http {
    fn releases(&self, kind: Kind, repo: &Repo, etag: Option<&str>) -> Result<Fetched, ForgeError> {
        let (url, token) = match kind {
            Kind::GitHub => (
                format!(
                    "{}/repos/{}/releases?per_page=100",
                    self.api("https://api.github.com"),
                    repo.path
                ),
                self.github_token
                    .as_deref()
                    .map(|t| ("Authorization", format!("Bearer {t}"))),
            ),
            Kind::GitLab => (
                format!(
                    "{}/api/v4/projects/{}/releases?per_page=100",
                    self.api("https://gitlab.com"),
                    repo.path.replace('/', "%2F")
                ),
                self.gitlab_token
                    .as_deref()
                    .map(|t| ("PRIVATE-TOKEN", t.to_string())),
            ),
            Kind::Gitea => (
                format!(
                    "{}/api/v1/repos/{}/releases?limit=50",
                    self.api(&format!("https://{}", repo.host)),
                    repo.path
                ),
                None,
            ),
        };
        let token = token.as_ref().map(|(h, v)| (*h, v.as_str()));
        let (status, headers, body) = self.request(&url, etag, token, None)?;
        self.finish(kind, status, &headers, &body)
    }

    fn releases_many(&self, repos: &[(Kind, Repo)]) -> Vec<(Repo, Result<Fetched, ForgeError>)> {
        let github: Vec<&Repo> = repos
            .iter()
            .filter(|(k, _)| *k == Kind::GitHub)
            .map(|(_, r)| r)
            .collect();
        let mut out = Vec::new();
        let batched = github.len() > 1 && self.github_token.is_some();
        if batched && let Some(done) = self.github_graphql(&github) {
            out.extend(done);
        }
        for (kind, repo) in repos {
            if batched && *kind == Kind::GitHub && out.iter().any(|(r, _)| r == repo) {
                continue;
            }
            out.push((repo.clone(), self.releases(*kind, repo, None)));
        }
        out
    }
}

impl Http {
    /// Every repository's releases in one GraphQL query (it needs a token).
    fn github_graphql(&self, repos: &[&Repo]) -> Option<Vec<(Repo, Result<Fetched, ForgeError>)>> {
        let token = self.github_token.as_deref()?;
        let mut query = String::from("query {");
        for (i, repo) in repos.iter().enumerate() {
            let (owner, name) = repo.path.split_once('/')?;
            query.push_str(&format!(
                " r{i}: repository(owner: {owner:?}, name: {name:?}) {{ releases(first: 100, orderBy: {{field: CREATED_AT, direction: DESC}}) {{ nodes {{ tagName publishedAt isDraft }} }} }}"
            ));
        }
        query.push_str(" }");
        let body = serde_json::json!({ "query": query }).to_string();
        let url = format!("{}/graphql", self.api("https://api.github.com"));
        let auth = format!("Bearer {token}");
        let (status, _, text) = self
            .request(&url, None, Some(("Authorization", &auth)), Some(&body))
            .ok()?;
        if status != 200 {
            return None;
        }
        let value: Value = serde_json::from_str(&text).ok()?;
        let data = value.get("data")?;
        let mut out = Vec::new();
        for (i, repo) in repos.iter().enumerate() {
            let nodes = data
                .get(format!("r{i}"))?
                .get("releases")?
                .get("nodes")?
                .as_array()?;
            let releases = nodes
                .iter()
                .filter(|n| n.get("isDraft").and_then(Value::as_bool) != Some(true))
                .filter_map(|n| {
                    Some(Release {
                        tag: n.get("tagName")?.as_str()?.to_string(),
                        published: parse_time(n.get("publishedAt")?.as_str()?)?,
                    })
                })
                .collect();
            out.push((
                (*repo).clone(),
                Ok(Fetched::Fresh {
                    etag: None,
                    releases,
                }),
            ));
        }
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct Fake {
        answers: RefCell<Vec<Result<Fetched, ForgeError>>>,
        asked: RefCell<Vec<Option<String>>>,
    }

    impl Forge for Fake {
        fn releases(&self, _: Kind, _: &Repo, etag: Option<&str>) -> Result<Fetched, ForgeError> {
            self.asked.borrow_mut().push(etag.map(str::to_string));
            self.answers.borrow_mut().remove(0)
        }
    }

    fn repo() -> Repo {
        Repo {
            host: "github.com".into(),
            path: "a/b".into(),
        }
    }

    fn fresh(tag: &str, time: i64, etag: &str) -> Result<Fetched, ForgeError> {
        Ok(Fetched::Fresh {
            etag: Some(etag.into()),
            releases: vec![Release {
                tag: tag.into(),
                published: time,
            }],
        })
    }

    #[test]
    fn times_are_cached_by_commit_and_lists_use_their_etag() {
        let forge = Fake {
            answers: RefCell::new(vec![fresh("v1", 100, "\"e1\""), Ok(Fetched::NotModified)]),
            asked: RefCell::new(vec![]),
        };
        let mut cache = Cache::in_memory();
        assert_eq!(published(&forge, &mut cache, &repo(), "v1", "c1"), Ok(100));
        // Known now: no request.
        assert_eq!(published(&forge, &mut cache, &repo(), "v1", "c1"), Ok(100));
        assert_eq!(forge.asked.borrow().len(), 1);
        // The tag moved: a miss, answered from the list through a conditional request.
        cache.fresh.clear();
        assert_eq!(published(&forge, &mut cache, &repo(), "v1", "c2"), Ok(100));
        assert_eq!(
            forge.asked.borrow().as_slice(),
            [None, Some("\"e1\"".to_string())]
        );
    }

    #[test]
    fn unknown_hosts_missing_releases_and_failures_are_distinct_errors() {
        let forge = Fake {
            answers: RefCell::new(vec![
                fresh("v1", 1, "e"),
                Err(ForgeError::RateLimited("slow down".into())),
            ]),
            asked: RefCell::new(vec![]),
        };
        let mut cache = Cache::in_memory();
        let other = Repo {
            host: "git.example.org".into(),
            path: "a/b".into(),
        };
        assert!(matches!(
            published(&forge, &mut cache, &other, "v1", "c"),
            Err(Unknown::NoRecord(_))
        ));
        assert!(matches!(
            published(&forge, &mut cache, &repo(), "v9", "c"),
            Err(Unknown::NoRecord(_))
        ));
        // No etag held and the list is cached, so the failure comes on the next fresh ask.
        let mut empty = Cache::in_memory();
        assert_eq!(
            published(&forge, &mut empty, &repo(), "v1", "c"),
            Err(Unknown::Failed("slow down".into()))
        );
    }

    #[test]
    fn release_lists_read_each_forges_shape_and_skip_drafts() {
        let github: Value = serde_json::from_str(
            r#"[{"tag_name":"v2","published_at":"2024-05-01T12:00:00Z"},{"tag_name":"v3","published_at":null,"draft":true},{"tag_name":"v1","published_at":"1970-01-02T00:00:00Z"}]"#,
        )
        .unwrap();
        assert_eq!(
            releases_from(Kind::GitHub, &github),
            [
                Release {
                    tag: "v2".into(),
                    published: 1_714_564_800
                },
                Release {
                    tag: "v1".into(),
                    published: 86_400
                },
            ]
        );
        let gitlab: Value =
            serde_json::from_str(r#"[{"tag_name":"v1","released_at":"1970-01-02T00:00:00Z"}]"#)
                .unwrap();
        assert_eq!(releases_from(Kind::GitLab, &gitlab).len(), 1);
    }

    #[test]
    fn curl_output_loses_its_interim_headers() {
        let text = "HTTP/1.1 100 Continue\r\n\r\nHTTP/1.1 200 OK\r\nETag: \"x\"\r\n\r\n[]";
        let (status, headers, body) = parse_response(text).unwrap();
        assert_eq!((status, body.as_str()), (200, "[]"));
        assert_eq!(header(&headers, "etag"), Some("\"x\""));
    }
}

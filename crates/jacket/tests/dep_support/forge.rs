//! A forge on localhost: release lists in GitHub's, GitLab's and Gitea's shapes, `ETag` and
//! `If-None-Match`, a rate limit, and GitHub's GraphQL batch. It records every request.
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub conditional: bool,
    pub authorization: Option<String>,
    pub private_token: Option<String>,
    pub status: u16,
}

#[derive(Default)]
struct State {
    /// `owner/name` to `(tag, published)` and the list's version.
    releases: BTreeMap<String, (Vec<(String, i64)>, u32)>,
    log: Vec<Request>,
    limited: bool,
}

pub struct Forge {
    pub base: String,
    state: Arc<Mutex<State>>,
}

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// `2024-05-01T12:00:00Z` for Unix seconds.
pub fn rfc3339(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let rest = secs.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 {
        mp + 3
    } else {
        mp - 9
    };
    let y = yoe + era * 400 + i64::from(m <= 2);

    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    )
}

impl Forge {
    pub fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let state = Arc::new(Mutex::new(State::default()));
        let shared = state.clone();

        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let state = shared.clone();

                std::thread::spawn(move || serve(stream, &state));
            }
        });
        Self {
            base,
            state,
        }
    }

    /// Publishes `tag` of `repo` (`owner/name`) `secs_ago` seconds ago.
    pub fn release(&self, repo: &str, tag: &str, secs_ago: i64) {
        let mut state = self.state.lock().unwrap();
        let entry = state.releases.entry(repo.to_string()).or_default();

        entry.0.retain(|(t, _)| t != tag);
        entry.0.insert(0, (tag.to_string(), now() - secs_ago));
        entry.1 += 1;
    }

    pub fn limit(&self, on: bool) {
        self.state.lock().unwrap().limited = on;
    }

    pub fn log(&self) -> Vec<Request> {
        self.state.lock().unwrap().log.clone()
    }

    pub fn clear(&self) {
        self.state.lock().unwrap().log.clear();
    }
}

fn serve(mut stream: TcpStream, state: &Mutex<State>) {
    let mut data = Vec::new();
    let mut chunk = [0u8; 4096];
    let (head, body) = loop {
        let n = stream.read(&mut chunk).unwrap_or(0);

        if n == 0 {
            return;
        }

        data.extend_from_slice(&chunk[..n]);
        let text = String::from_utf8_lossy(&data).into_owned();

        if let Some((head, rest)) = text.split_once("\r\n\r\n") {
            let length = head
                .lines()
                .find_map(|l| {
                    l.to_ascii_lowercase()
                        .strip_prefix("content-length:")
                        .map(|v| v.trim().parse::<usize>().unwrap_or(0))
                })
                .unwrap_or(0);

            if rest.len() >= length {
                break (head.to_string(), rest.to_string());
            }
        }
    };
    let mut lines = head.lines();
    let first = lines.next().unwrap_or_default().to_string();
    let mut parts = first.split_whitespace();
    let (method, path) = (
        parts.next().unwrap_or("").to_string(),
        parts.next().unwrap_or("").to_string(),
    );
    let header = |name: &str| {
        head.lines().skip(1).find_map(|l| {
            let (k, v) = l.split_once(':')?;
            k.eq_ignore_ascii_case(name).then(|| v.trim().to_string())
        })
    };
    let inbound_etag = header("if-none-match");
    let mut state = state.lock().unwrap();
    let route = path.split('?').next().unwrap_or("").to_string();
    let (status, etag, json) = if state.limited {
        (403, None, "{}".to_string())
    } else if method == "POST" && route == "/graphql" {
        (200, None, graphql(&state, &body))
    } else if let Some(repo) = list_repo(&route) {
        let (releases, version) = state.releases.get(&repo).cloned().unwrap_or_default();
        let etag = format!("\"{repo}-{version}\"");

        if inbound_etag.as_deref() == Some(etag.as_str()) {
            (304, Some(etag), String::new())
        } else {
            (200, Some(etag), list_json(&route, &releases))
        }
    } else {
        (404, None, "{}".to_string())
    };
    let request = Request {
        method,
        path,
        conditional: inbound_etag.is_some(),
        authorization: header("authorization"),
        private_token: header("private-token"),
        status,
    };

    state.log.push(request);
    drop(state);
    let mut out = format!(
        "HTTP/1.1 {status} X\r\nConnection: close\r\nContent-Length: {}\r\n",
        json.len()
    );

    if let Some(etag) = etag {
        out.push_str(&format!("ETag: {etag}\r\n"));
    }

    if status == 403 {
        out.push_str("x-ratelimit-remaining: 0\r\n");
    }

    out.push_str("\r\n");
    out.push_str(&json);
    let _ = stream.write_all(out.as_bytes());
}

/// The `owner/name` a release-list route names.
fn list_repo(route: &str) -> Option<String> {
    if let Some(rest) = route.strip_prefix("/api/v4/projects/") {
        return rest
            .strip_suffix("/releases")
            .map(|r| r.replace("%2F", "/"));
    }

    let rest = route
        .strip_prefix("/api/v1/repos/")
        .or_else(|| route.strip_prefix("/repos/"))?;
    rest.strip_suffix("/releases").map(str::to_string)
}

fn list_json(route: &str, releases: &[(String, i64)]) -> String {
    let time_key = if route.starts_with("/api/v4/") {
        "released_at"
    } else {
        "published_at"
    };
    let rows: Vec<String> = releases
        .iter()
        .map(|(tag, at)| {
            format!(
                "{{\"tag_name\":\"{tag}\",\"{time_key}\":\"{}\"}}",
                rfc3339(*at)
            )
        })
        .collect();

    format!("[{}]", rows.join(","))
}

/// Answers `r0: repository(owner: "a", name: "b")` aliases.
fn graphql(state: &State, body: &str) -> String {
    let query: String = serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v["query"].as_str().map(str::to_string))
        .unwrap_or_default();
    let mut fields = Vec::new();

    for (at, marker) in query.match_indices(": repository(owner: ") {
        let alias = query[..at].rsplit(' ').next().unwrap_or("");
        let rest = &query[at + marker.len()..];
        let owner = rest.split('"').nth(1).unwrap_or("");
        let name = rest
            .split("name: ")
            .nth(1)
            .and_then(|n| n.split('"').nth(1))
            .unwrap_or("");
        let releases = state
            .releases
            .get(&format!("{owner}/{name}"))
            .cloned()
            .unwrap_or_default()
            .0;
        let nodes: Vec<String> = releases
            .iter()
            .map(|(tag, at)| {
                format!(
                    "{{\"tagName\":\"{tag}\",\"publishedAt\":\"{}\",\"isDraft\":false}}",
                    rfc3339(*at)
                )
            })
            .collect();

        fields.push(format!(
            "\"{alias}\":{{\"releases\":{{\"nodes\":[{}]}}}}",
            nodes.join(",")
        ));
    }

    format!("{{\"data\":{{{}}}}}", fields.join(","))
}
